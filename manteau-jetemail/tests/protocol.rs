use manteau_core::{
  Acceptance, Address, Envelope, HeaderText, IdempotencyKey,
  IdempotentTransport, PreparedMessage, Receipt, Recipients, Rendered, Sender,
  Submission, Transport, TransportFailure, http::HttpConfig,
};
use manteau_http::HttpClient;
use manteau_jetemail::{JetEmail, JetEmailErrorKind};
use wiremock::{
  Mock, MockServer, ResponseTemplate,
  matchers::{header, method, path},
};

struct Fixture(PreparedMessage);

impl Fixture {
  fn new() -> Self {
    Self(PreparedMessage::new(
      Envelope::new(
        Address::new("from@example.com".parse().unwrap())
          .name(HeaderText::new("Sender").unwrap()),
        Recipients::to(Address::new("to@example.com".parse().unwrap())),
        HeaderText::new("Hello").unwrap(),
      ),
      Rendered::new("<p>Private</p>", "Private").unwrap(),
    ))
  }

  fn message(&self) -> &PreparedMessage {
    &self.0
  }

  fn sender(&self, server: &MockServer) -> Sender<JetEmail, HttpClient> {
    Sender::new(
      JetEmail::with_config(
        "test-token",
        HttpConfig::new(&format!("{}/email", server.uri())).unwrap(),
      )
      .unwrap(),
      HttpClient::new().unwrap(),
    )
  }
}

#[tokio::test]
async fn replay_preserves_key_and_body() {
  let fixture = Fixture::new();
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/email"))
    .and(header("Authorization", "Bearer test-token"))
    .and(header("Idempotency-Key", "notification-1"))
    .respond_with(
      ResponseTemplate::new(201)
        .set_body_json(serde_json::json!({"id":"original"})),
    )
    .expect(2)
    .mount(&server)
    .await;

  let mail = fixture.sender(&server);
  let key = IdempotencyKey::try_from("notification-1".to_owned()).unwrap();
  let submission = Submission::new(key, fixture.message().clone(), &mail);
  let persisted = serde_json::to_vec(&submission).unwrap();
  let restored: Submission = serde_json::from_slice(&persisted).unwrap();

  for attempt in [&submission, &restored] {
    let receipt = mail.send_idempotent(attempt).await.unwrap();
    assert_eq!(receipt.ids()[0].as_str(), "original");
    assert!(!format!("{receipt:?}").contains("original"));
  }

  let requests = server.received_requests().await.unwrap();
  assert_eq!(requests[0].body, requests[1].body);
  let body: serde_json::Value =
    serde_json::from_slice(&requests[0].body).unwrap();
  assert_eq!(body["to"], serde_json::json!(["to@example.com"]));
  assert_eq!(body["html"], "<p>Private</p>");
}

#[tokio::test]
async fn rejection_and_uncertainty_are_distinct() {
  let fixture = Fixture::new();
  for (status, body, acceptance, provider_kind) in [
    (
      409,
      serde_json::json!({"code":"IDEMPOTENCY_BODY_MISMATCH"}),
      Acceptance::Unknown,
      Some(JetEmailErrorKind::Conflict),
    ),
    (
      409,
      serde_json::json!({"code":"IDEMPOTENCY_IN_FLIGHT"}),
      Acceptance::Unknown,
      Some(JetEmailErrorKind::InFlight),
    ),
    (429, serde_json::json!({}), Acceptance::Unknown, None),
    (401, serde_json::json!({}), Acceptance::NotAccepted, None),
    (
      500,
      serde_json::json!({"private":"secret"}),
      Acceptance::Unknown,
      None,
    ),
    (
      201,
      serde_json::json!({"id":""}),
      Acceptance::Unknown,
      Some(JetEmailErrorKind::Response),
    ),
    (
      201,
      serde_json::json!({}),
      Acceptance::Unknown,
      Some(JetEmailErrorKind::Response),
    ),
    (302, serde_json::json!({}), Acceptance::Unknown, None),
  ] {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
      .respond_with(
        ResponseTemplate::new(status)
          .insert_header("Retry-After", "12")
          .set_body_json(body),
      )
      .expect(1)
      .mount(&server)
      .await;

    let mail = fixture.sender(&server);
    let submission = Submission::new(
      IdempotencyKey::try_from("one".to_owned()).unwrap(),
      fixture.message().clone(),
      &mail,
    );
    let error = mail.send_idempotent(&submission).await.unwrap_err();
    assert_eq!(error.acceptance(), acceptance);
    assert!(!error.to_string().contains("secret"));
    if let Some(kind) = provider_kind {
      assert!(
        matches!(error, manteau_core::SendError::Provider(ref source) if source.kind() == kind)
      );
    }
    if status == 429 {
      assert_eq!(
        error.retry_after(),
        Some(std::time::Duration::from_secs(12))
      );
    }
  }
}

#[test]
fn persisted_input_remains_checked() {
  let fixture = Fixture::new();
  for key in ["".to_owned(), "x".repeat(257), "key\r\ninjected".to_owned()] {
    assert!(IdempotencyKey::try_from(key.clone()).is_err());
    assert!(
      serde_json::from_value::<IdempotencyKey>(serde_json::json!(key)).is_err()
    );
  }

  for (field, value) in [
    ("from", serde_json::json!("bad")),
    ("to", serde_json::json!([])),
    ("subject", serde_json::json!("x\nBcc: y")),
  ] {
    let mut content = serde_json::to_value(fixture.message()).unwrap();
    match field {
      "from" => content["envelope"]["from"]["email"] = value,
      "to" => content["envelope"]["recipients"]["to"] = value,
      _ => content["envelope"][field] = value,
    }
    assert!(serde_json::from_value::<PreparedMessage>(content).is_err());
  }
}

#[tokio::test]
async fn response_limit_is_enforced_after_dispatch() {
  let fixture = Fixture::new();
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .respond_with(
      ResponseTemplate::new(201)
        .set_body_string("a response larger than eight bytes"),
    )
    .expect(1)
    .mount(&server)
    .await;

  let config = HttpConfig::new(&format!("{}/email", server.uri()))
    .unwrap()
    .response_limit(std::num::NonZeroUsize::new(8).unwrap());
  let mail = Sender::new(
    JetEmail::with_config("token", config).unwrap(),
    HttpClient::new().unwrap(),
  );
  let error = mail.send(fixture.message()).await.unwrap_err();
  assert!(matches!(
    error,
    manteau_core::SendError::Http(manteau_core::http::HttpError::ResponseLimit)
  ));
  assert_eq!(error.acceptance(), Acceptance::Unknown);
}

#[tokio::test]
async fn total_request_timeout_keeps_acceptance_unknown() {
  let fixture = Fixture::new();
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .respond_with(
      ResponseTemplate::new(201)
        .set_delay(std::time::Duration::from_millis(200))
        .set_body_json(serde_json::json!({"id":"late"})),
    )
    .mount(&server)
    .await;

  let config = HttpConfig::new(&format!("{}/email", server.uri()))
    .unwrap()
    .timeout(std::time::Duration::from_millis(20))
    .unwrap();
  let mail = Sender::new(
    JetEmail::with_config("token", config).unwrap(),
    HttpClient::new().unwrap(),
  );
  let error = mail.send(fixture.message()).await.unwrap_err();
  assert!(matches!(
    error,
    manteau_core::SendError::Http(manteau_core::http::HttpError::Network(_))
  ));
  assert_eq!(error.acceptance(), Acceptance::Unknown);
}

#[tokio::test]
async fn redirect_does_not_send_credentials_to_another_endpoint() {
  let fixture = Fixture::new();
  let first = MockServer::start().await;
  let target = MockServer::start().await;
  Mock::given(method("POST"))
    .respond_with(
      ResponseTemplate::new(302)
        .insert_header("Location", format!("{}/other", target.uri()).as_str()),
    )
    .expect(1)
    .mount(&first)
    .await;
  let error = fixture
    .sender(&first)
    .send(fixture.message())
    .await
    .unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::Unknown);
  assert!(target.received_requests().await.unwrap().is_empty());
}
