use manteau_cloudflare::{Cloudflare, CloudflareErrorKind};
use manteau_core::{
  Acceptance, Address, Envelope, HeaderText, PreparedMessage, Receipt,
  Recipients, Rendered, Sender, Transport, TransportFailure, http::HttpConfig,
};
use manteau_http::HttpClient;
use wiremock::{
  Mock, MockServer, ResponseTemplate,
  matchers::{body_partial_json, header, method, path},
};

const PATH: &str = "/client/v4/accounts/test-account/email/sending/send";
struct Fixture<'a>(&'a MockServer);

impl Fixture<'_> {
  fn message(
    &self,
    subject: &str,
    extra: bool,
  ) -> manteau_core::PreparedMessage {
    let recipients =
      Recipients::to(Address::new("to@example.com".parse().unwrap()));
    let recipients = if extra {
      recipients.push_cc(Address::new("cc@example.com".parse().unwrap()))
    } else {
      recipients
    };
    PreparedMessage::new(
      Envelope::new(
        Address::new("from@example.com".parse().unwrap()),
        recipients,
        HeaderText::new(subject).unwrap(),
      ),
      Rendered::new("<p>Hi!</p>", "Hi!").unwrap(),
    )
  }

  fn sender(&self) -> Sender<Cloudflare, HttpClient> {
    Sender::new(
      Cloudflare::with_config(
        "test-token",
        HttpConfig::new(&format!("{}{}", self.0.uri(), PATH)).unwrap(),
      )
      .unwrap(),
      HttpClient::new().unwrap(),
    )
  }
}

#[tokio::test]
async fn wire_request_and_real_outcome() {
  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  Mock::given(method("POST")).and(path(PATH))
    .and(header("authorization", "Bearer test-token"))
    .and(body_partial_json(serde_json::json!({"from":{"address":"from@example.com"},"to":[{"address":"to@example.com"}],"subject":"Hello"})))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":true,"result":{"message_id":"provider-1","delivered":["to@example.com"],"queued":[],"permanent_bounces":[],"suppressed_recipients":[]}
    }))).expect(1).mount(&server).await;
  let receipt = fixture
    .sender()
    .send(&fixture.message("Hello", false))
    .await
    .unwrap();
  assert_eq!(receipt.ids()[0].as_str(), "provider-1");
  assert_eq!(receipt.delivered()[0].as_str(), "to@example.com");
  assert!(!format!("{receipt:?}").contains("to@example.com"));
}

#[tokio::test]
async fn named_recipients_keep_their_display_names() {
  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  Mock::given(method("POST"))
    .and(body_partial_json(serde_json::json!({
      "to":[{"address":"to@example.com","name":"Recipient"}]
    })))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":true,"result":{"message_id":"id","delivered":["to@example.com"],"queued":[],"permanent_bounces":[]}
    })))
    .expect(1)
    .mount(&server)
    .await;

  let named = Address::new("to@example.com".parse().unwrap())
    .name(HeaderText::new("Recipient").unwrap());
  let prepared = manteau_core::PreparedMessage::new(
    Envelope::new(
      Address::new("from@example.com".parse().unwrap()),
      Recipients::to(named),
      HeaderText::new("Hello").unwrap(),
    ),
    manteau_core::Rendered::new("<p>Hello</p>", "Hello").unwrap(),
  );
  fixture.sender().send(&prepared).await.unwrap();
}

#[tokio::test]
async fn duplicate_recipients_are_rejected_before_dispatch() {
  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  let prepared = manteau_core::PreparedMessage::new(
    Envelope::new(
      Address::new("from@example.com".parse().unwrap()),
      Recipients::to(Address::new("same@example.com".parse().unwrap()))
        .push_cc(Address::new("same@example.com".parse().unwrap())),
      HeaderText::new("Hello").unwrap(),
    ),
    manteau_core::Rendered::new("<p>Hello</p>", "Hello").unwrap(),
  );
  let error = fixture.sender().send(&prepared).await.unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::NotAccepted);
  assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn unicode_subject_and_mixed_outcomes() {
  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  Mock::given(method("POST")).and(path(PATH))
    .and(body_partial_json(serde_json::json!({"subject":"=?UTF-8?Q?Welcome_=E2=80=94_test?="})))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":true,"result":{"message_id":"provider-2","delivered":["to@example.com"],"queued":[],"permanent_bounces":["cc@example.com"]}
    }))).expect(1).mount(&server).await;
  let receipt = fixture
    .sender()
    .send(&fixture.message("Welcome — test", true))
    .await
    .unwrap();
  assert_eq!(receipt.delivered().len(), 1);
  assert_eq!(receipt.permanent_bounces()[0].as_str(), "cc@example.com");
}

#[tokio::test]
async fn suppression_is_reported_alongside_accepted_recipients() {
  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  Mock::given(method("POST"))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":true,
      "result":{
        "message_id":"provider-3",
        "delivered":[],
        "queued":["to@example.com"],
        "permanent_bounces":[],
        "suppressed_recipients":["cc@example.com"]
      }
    })))
    .expect(1)
    .mount(&server)
    .await;

  let receipt = fixture
    .sender()
    .send(&fixture.message("Hello", true))
    .await
    .unwrap();
  assert_eq!(receipt.ids()[0].as_str(), "provider-3");
  assert_eq!(receipt.queued()[0].as_str(), "to@example.com");
  assert_eq!(receipt.suppressed()[0].as_str(), "cc@example.com");
}

#[tokio::test]
async fn contradictory_success_is_uncertain() {
  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
    "success":true,"result":{"message_id":"id","delivered":["someone@example.com"],"queued":[],"permanent_bounces":[]}
  }))).expect(1).mount(&server).await;
  let error = fixture
    .sender()
    .send(&fixture.message("Hello", false))
    .await
    .unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::Unknown);
  assert!(
    matches!(error, manteau_core::SendError::Provider(ref source) if source.kind() == CloudflareErrorKind::Response)
  );
}

#[tokio::test]
async fn error_flag_with_result_is_uncertain() {
  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  Mock::given(method("POST"))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":false,"errors":[{"code":10202}],
      "result":{"message_id":"id","delivered":["to@example.com"],"queued":[],"permanent_bounces":[]}
    })))
    .expect(1)
    .mount(&server)
    .await;
  let error = fixture
    .sender()
    .send(&fixture.message("Hello", false))
    .await
    .unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::Unknown);
}

#[tokio::test]
async fn application_rejection_and_status_evidence() {
  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  Mock::given(method("POST"))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":false,"errors":[{"code":10202}],"result":null
    })))
    .expect(1)
    .mount(&server)
    .await;
  let error = fixture
    .sender()
    .send(&fixture.message("Hello", false))
    .await
    .unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::NotAccepted);
  assert!(
    matches!(error, manteau_core::SendError::Provider(ref source) if source.provider_code() == Some(10202))
  );

  let server = MockServer::start().await;
  let fixture = Fixture(&server);
  Mock::given(method("POST"))
    .respond_with(ResponseTemplate::new(503))
    .expect(1)
    .mount(&server)
    .await;
  let error = fixture
    .sender()
    .send(&fixture.message("Hello", false))
    .await
    .unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::Unknown);
  assert!(error.is_transient());
}
