use manteau_core::{
  Address, Envelope, HeaderText, IdempotencyKey, IdempotentTransport,
  PreparedMessage, Receipt, Recipients, Rendered, TransportFailure,
};
use manteau_jetemail::{JetEmailConfig, JetEmailErrorKind, JetEmailTransport};
use wiremock::{
  Mock, MockServer, ResponseTemplate,
  matchers::{header, method, path},
};

struct Fixture;
impl Fixture {
  fn message() -> PreparedMessage {
    PreparedMessage::new(
      Envelope::new(
        Address::new("from@example.com".parse().unwrap())
          .name(HeaderText::new("Sender").unwrap()),
        Recipients::to(Address::new("to@example.com".parse().unwrap())),
        HeaderText::new("Hello").unwrap(),
      ),
      Rendered::new("<p>Private</p>", "Private").unwrap(),
    )
  }

  fn transport(server: &MockServer) -> JetEmailTransport {
    JetEmailTransport::new("test-token", JetEmailConfig {
      endpoint: format!("{}/email", server.uri()),
      ..Default::default()
    })
    .unwrap()
  }
}

#[tokio::test]
async fn retries_preserve_key_body_and_acceptance() {
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
  let mail = Fixture::transport(&server);
  let key = IdempotencyKey::try_from("notification-1".to_owned()).unwrap();
  let message = Fixture::message();
  let persisted = serde_json::to_vec(&message).unwrap();
  let restored: PreparedMessage = serde_json::from_slice(&persisted).unwrap();
  for message in [&message, &restored] {
    assert_eq!(
      mail.send_idempotent(&key, message).await.unwrap().ids()[0].as_str(),
      "original"
    );
  }
  let requests = server.received_requests().await.unwrap();
  assert_eq!(requests[0].body, requests[1].body);
  let body: serde_json::Value =
    serde_json::from_slice(&requests[0].body).unwrap();
  assert_eq!(body["to"], serde_json::json!(["to@example.com"]));
  assert_eq!(body["html"], "<p>Private</p>");
  assert_eq!(mail.retention(), std::time::Duration::from_secs(86400));
}

#[tokio::test]
async fn failure_classification_does_not_hide_uncertain_acceptance() {
  for (status, body, expected) in [
    (
      409,
      serde_json::json!({"code":"IDEMPOTENCY_BODY_MISMATCH"}),
      JetEmailErrorKind::Conflict,
    ),
    (
      409,
      serde_json::json!({"code":"IDEMPOTENCY_IN_FLIGHT"}),
      JetEmailErrorKind::InFlight,
    ),
    (429, serde_json::json!({}), JetEmailErrorKind::RateLimited),
    (
      401,
      serde_json::json!({}),
      JetEmailErrorKind::Authentication,
    ),
    (
      500,
      serde_json::json!({"private":"secret"}),
      JetEmailErrorKind::Uncertain,
    ),
    (
      201,
      serde_json::json!({"id":""}),
      JetEmailErrorKind::Uncertain,
    ),
    (201, serde_json::json!({}), JetEmailErrorKind::Uncertain),
    (302, serde_json::json!({}), JetEmailErrorKind::Uncertain),
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
    let mail = Fixture::transport(&server);
    let key = IdempotencyKey::try_from("one".to_owned()).unwrap();
    let error = mail
      .send_idempotent(&key, &Fixture::message())
      .await
      .unwrap_err();
    assert_eq!(error.kind(), &expected);
    assert!(!error.to_string().contains("secret"));
    if expected == JetEmailErrorKind::Uncertain {
      assert!(!error.is_transient());
    }
    if expected == JetEmailErrorKind::RateLimited {
      assert_eq!(
        error.retry_after(),
        Some(std::time::Duration::from_secs(12))
      );
    }
  }
}

#[test]
fn deserialization_preserves_input_contracts() {
  for key in ["".to_owned(), "x".repeat(257), "key\r\ninjected".to_owned()] {
    assert!(IdempotencyKey::try_from(key.clone()).is_err());
    assert!(
      serde_json::from_value::<IdempotencyKey>(serde_json::json!(key)).is_err()
    );
  }
  let key = IdempotencyKey::try_from("event/1".to_owned()).unwrap();
  assert_eq!(serde_json::to_string(&key).unwrap(), "\"event/1\"");
  for (field, value) in [
    ("from", serde_json::json!("bad")),
    ("to", serde_json::json!([])),
    ("subject", serde_json::json!("x\nBcc: y")),
  ] {
    let mut content = serde_json::to_value(Fixture::message()).unwrap();
    match field {
      "from" => content["envelope"]["from"]["email"] = value,
      "to" => content["envelope"]["recipients"]["to"] = value,
      _ => content["envelope"][field] = value,
    }
    assert!(serde_json::from_value::<PreparedMessage>(content).is_err());
  }
}
