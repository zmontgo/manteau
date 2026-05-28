//! Integration test for `MailjetTransport` against a wiremock-backed fake of
//! the Mailjet `/v3.1/send` endpoint.
//!
//! Gated via `required-features = ["mailjet"]` in `Cargo.toml`, so this
//! whole test binary is skipped without the feature.

use manteau::{
  MailjetTransport, Message, Transport, prelude::*, transport::TransportFailure,
};
use wiremock::{
  Mock, MockServer, ResponseTemplate,
  matchers::{body_partial_json, header, method, path},
};

fn make_message() -> Message {
  let template = Template::new(
    Body::new().push(Section::new().push(Column::new().push(Text::new("Hi!")))),
  );

  Message::new(
    Address::new("from@example.com".parse().unwrap()),
    vec![Address::new("to@example.com".parse().unwrap())],
    "Hello",
    template,
  )
}

fn transport(server_uri: &str) -> MailjetTransport {
  MailjetTransport::new("test-key", "test-secret")
    .base_url(server_uri.parse().expect("wiremock URL"))
}

#[tokio::test]
async fn success_returns_message_ids() {
  let server = MockServer::start().await;

  Mock::given(method("POST"))
    .and(path("/v3.1/send"))
    .and(header("content-type", "application/json"))
    .and(body_partial_json(serde_json::json!({
        "Messages": [{
            "From": {"Email": "from@example.com"},
            "Subject": "Hello",
        }]
    })))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "Messages": [{
            "Status": "success",
            "To": [{
                "Email": "to@example.com",
                "MessageID": 18014398509481984u64,
            }]
        }]
    })))
    .mount(&server)
    .await;

  let receipt = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap();

  assert_eq!(receipt.ids.len(), 1);
  assert_eq!(receipt.ids[0].as_str(), "18014398509481984");
}

#[tokio::test]
async fn string_message_id_unquoted() {
  // Edge case: some Mailjet response paths return MessageID as a JSON string
  // instead of a number. The id must come back without surrounding quotes.
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/v3.1/send"))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "Messages": [{
            "To": [{"MessageID": "abc-123"}]
        }]
    })))
    .mount(&server)
    .await;

  let receipt = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap();
  assert_eq!(receipt.ids[0].as_str(), "abc-123");
}

#[tokio::test]
async fn auth_failure_is_classified() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/v3.1/send"))
    .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(err.is_auth());
  assert!(!err.is_transient());
}

#[tokio::test]
async fn server_error_is_transient() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/v3.1/send"))
    .respond_with(
      ResponseTemplate::new(503).set_body_string("Service Unavailable"),
    )
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(err.is_transient());
  assert!(!err.is_auth());
}

#[tokio::test]
async fn bad_request_is_message_rejected() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/v3.1/send"))
    .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(err.is_message_rejected());
  assert!(!err.is_transient());
}
