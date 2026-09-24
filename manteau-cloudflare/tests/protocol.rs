use manteau_cloudflare::{Cloudflare, CloudflareErrorKind};
use manteau_core::{
  Acceptance, Address, Envelope, HeaderText, Message, Receipt, Recipients,
  Sender, Transport, TransportFailure, http::HttpConfig, prelude::*,
};
use manteau_http::HttpClient;
use wiremock::{
  Mock, MockServer, ResponseTemplate,
  matchers::{body_partial_json, header, method, path},
};

const PATH: &str = "/client/v4/accounts/test-account/email/sending/send";
fn message(subject: &str, extra: bool) -> manteau_core::PreparedMessage {
  let recipients =
    Recipients::to(Address::new("to@example.com".parse().unwrap()));
  let recipients = if extra {
    recipients.push_cc(Address::new("cc@example.com".parse().unwrap()))
  } else {
    recipients
  };
  Message::new(
    Envelope::new(
      Address::new("from@example.com".parse().unwrap()),
      recipients,
      HeaderText::new(subject).unwrap(),
    ),
    Template::new(
      Body::new()
        .push(Section::new().push(Column::new().push(Text::new("Hi!")))),
    ),
  )
  .prepare()
  .unwrap()
}
fn sender(server: &MockServer) -> Sender<Cloudflare, HttpClient> {
  Sender::new(
    Cloudflare::with_config(
      "test-token",
      HttpConfig::new(&format!("{}{}", server.uri(), PATH)).unwrap(),
    )
    .unwrap(),
    HttpClient::new().unwrap(),
  )
}

#[tokio::test]
async fn wire_request_and_real_outcome() {
  let server = MockServer::start().await;
  Mock::given(method("POST")).and(path(PATH))
    .and(header("authorization", "Bearer test-token"))
    .and(body_partial_json(serde_json::json!({"from":{"address":"from@example.com"},"to":["to@example.com"],"subject":"Hello"})))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":true,"result":{"message_id":"provider-1","delivered":["to@example.com"],"queued":[],"permanent_bounces":[],"suppressed_recipients":[]}
    }))).expect(1).mount(&server).await;
  let receipt = sender(&server)
    .send(&message("Hello", false))
    .await
    .unwrap();
  assert_eq!(receipt.ids()[0].as_str(), "provider-1");
  assert_eq!(receipt.delivered()[0].as_str(), "to@example.com");
}

#[tokio::test]
async fn unicode_subject_and_mixed_outcomes() {
  let server = MockServer::start().await;
  Mock::given(method("POST")).and(path(PATH))
    .and(body_partial_json(serde_json::json!({"subject":"=?UTF-8?Q?Welcome_=E2=80=94_test?="})))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":true,"result":{"message_id":"provider-2","delivered":["to@example.com"],"queued":[],"permanent_bounces":["cc@example.com"]}
    }))).expect(1).mount(&server).await;
  let receipt = sender(&server)
    .send(&message("Welcome — test", true))
    .await
    .unwrap();
  assert_eq!(receipt.delivered().len(), 1);
  assert_eq!(receipt.permanent_bounces()[0].as_str(), "cc@example.com");
}

#[tokio::test]
async fn contradictory_success_is_uncertain() {
  let server = MockServer::start().await;
  Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
    "success":true,"result":{"message_id":"id","delivered":["someone@example.com"],"queued":[],"permanent_bounces":[]}
  }))).expect(1).mount(&server).await;
  let error = sender(&server)
    .send(&message("Hello", false))
    .await
    .unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::Unknown);
  assert!(
    matches!(error, manteau_core::SendError::Provider(ref source) if source.kind() == CloudflareErrorKind::Response)
  );
}

#[tokio::test]
async fn application_rejection_and_status_evidence() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "success":false,"errors":[{"code":10202}],"result":null
    })))
    .expect(1)
    .mount(&server)
    .await;
  let error = sender(&server)
    .send(&message("Hello", false))
    .await
    .unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::NotAccepted);
  assert!(
    matches!(error, manteau_core::SendError::Provider(ref source) if source.provider_code() == Some(10202))
  );

  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .respond_with(ResponseTemplate::new(503))
    .expect(1)
    .mount(&server)
    .await;
  let error = sender(&server)
    .send(&message("Hello", false))
    .await
    .unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::Unknown);
  assert!(error.is_transient());
}
