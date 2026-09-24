use manteau_core::{
  Acceptance, Address, Envelope, HeaderText, Message, Receipt, Recipients,
  Sender, Transport, TransportFailure, http::HttpConfig, prelude::*,
};
use manteau_http::HttpClient;
use manteau_mailjet::{Mailjet, MailjetErrorKind};
use wiremock::{
  Mock, MockServer, ResponseTemplate,
  matchers::{body_partial_json, header, method, path},
};

fn message() -> manteau_core::PreparedMessage {
  let template = Template::new(
    Body::new().push(Section::new().push(Column::new().push(Text::new("Hi!")))),
  );
  Message::new(
    Envelope::new(
      Address::new("from@example.com".parse().unwrap()),
      Recipients::to(Address::new("to@example.com".parse().unwrap())),
      HeaderText::new("Hello").unwrap(),
    ),
    template,
  )
  .prepare()
  .unwrap()
}

fn sender(server: &MockServer) -> Sender<Mailjet, HttpClient> {
  Sender::new(
    Mailjet::with_config(
      "test-key",
      "test-secret",
      HttpConfig::new(&format!("{}/v3.1/send", server.uri())).unwrap(),
    )
    .unwrap(),
    HttpClient::new().unwrap(),
  )
}

#[tokio::test]
async fn wire_request_and_real_recipient_id() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/v3.1/send"))
    .and(header("content-type", "application/json"))
    .and(body_partial_json(serde_json::json!({"Messages":[{"From":{"Email":"from@example.com"},"Subject":"Hello"}]})))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
      "Messages":[{"Status":"success","To":[{"Email":"to@example.com","MessageID":18014398509481984u64}]}]
    })))
    .expect(1).mount(&server).await;
  let receipt = sender(&server).send(&message()).await.unwrap();
  assert_eq!(receipt.ids()[0].as_str(), "18014398509481984");
  assert_eq!(receipt.recipients()[0].as_str(), "to@example.com");
}

#[tokio::test]
async fn malformed_success_remains_uncertain() {
  let server = MockServer::start().await;
  Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
    "Messages":[{"Status":"success","To":[{"Email":"someone-else@example.com","MessageID":"id"}]}]
  }))).expect(1).mount(&server).await;
  let error = sender(&server).send(&message()).await.unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::Unknown);
  assert!(
    matches!(error, manteau_core::SendError::Provider(ref source) if source.kind() == MailjetErrorKind::Response)
  );
}

#[tokio::test]
async fn status_classification_keeps_acceptance_separate() {
  for (status, acceptance, transient) in [
    (400, Acceptance::NotAccepted, false),
    (401, Acceptance::NotAccepted, false),
    (429, Acceptance::NotAccepted, true),
    (503, Acceptance::Unknown, true),
  ] {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
      .respond_with(ResponseTemplate::new(status))
      .expect(1)
      .mount(&server)
      .await;
    let error = sender(&server).send(&message()).await.unwrap_err();
    assert_eq!(error.acceptance(), acceptance);
    assert_eq!(error.is_transient(), transient);
    assert_eq!(error.is_auth(), status == 401);
  }
}
