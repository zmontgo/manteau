//! Integration test for `CloudflareTransport` against a wiremock-backed fake
//! of the Cloudflare Email Sending `/email/sending/send` endpoint.
//!
//! Gated via `required-features = ["cloudflare"]` in `Cargo.toml`, so this
//! whole test binary is skipped without the feature.
//!
//! These assert on `err.kind()` (the variant), not the `TransportFailure`
//! predicates — `send`'s request shaping and two-layer status handling are
//! what an integration test can verify, while the transient/auth/rejected
//! mapping is covered by the in-module unit tests.

use std::time::Duration;

use manteau::{
  CloudflareErrorKind, CloudflareTransport, Message, Transport, prelude::*,
};
use wiremock::{
  Mock, MockServer, ResponseTemplate,
  matchers::{body_partial_json, header, method, path},
};

const ACCOUNT_ID: &str = "test-account";
const SEND_PATH: &str = "/client/v4/accounts/test-account/email/sending/send";

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

fn transport(server_uri: &str) -> CloudflareTransport {
  CloudflareTransport::new(ACCOUNT_ID, "test-token")
    .base_url(server_uri.parse().expect("wiremock URL"))
}

#[tokio::test]
async fn success_populates_buckets_and_synthesizes_ids() {
  let server = MockServer::start().await;

  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .and(header("authorization", "Bearer test-token"))
    .and(header("content-type", "application/json"))
    // Wire-shape assertions: object `from` keyed by `address` (not `email`),
    // bare-string `to`, and the flat lowercase field names.
    .and(body_partial_json(serde_json::json!({
        "from": { "address": "from@example.com" },
        "to": ["to@example.com"],
        "subject": "Hello",
    })))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "success": true,
        "errors": [],
        "messages": [],
        "result": {
            "delivered": ["alice@example.com"],
            "queued": ["bob@example.com"],
            "permanent_bounces": [],
        }
    })))
    .mount(&server)
    .await;

  let receipt = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap();

  assert_eq!(receipt.delivered, ["alice@example.com"]);
  assert_eq!(receipt.queued, ["bob@example.com"]);
  assert!(receipt.permanent_bounces.is_empty());
  // ids() is synthesized from delivered + queued, in that order.
  let ids: Vec<&str> = receipt.ids.iter().map(|id| id.as_str()).collect();
  assert_eq!(ids, ["alice@example.com", "bob@example.com"]);
}

#[tokio::test]
async fn non_ascii_subject_is_rfc2047_encoded_on_the_wire() {
  // Regression for Cloudflare 10202 (email.sending.error.email.invalid): a
  // non-ASCII Subject must reach the wire RFC 2047-encoded so the resulting
  // header is valid ASCII. The matcher below only matches the encoded form —
  // a raw-UTF-8 subject would miss every mock, yield a 404, and fail the send.
  let server = MockServer::start().await;

  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .and(body_partial_json(serde_json::json!({
        "subject": "=?UTF-8?Q?Welcome_=E2=80=94_test?=",
    })))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "success": true,
        "errors": [],
        "messages": [],
        "result": {
            "delivered": ["to@example.com"],
            "queued": [],
            "permanent_bounces": [],
        }
    })))
    .mount(&server)
    .await;

  let template = Template::new(
    Body::new().push(Section::new().push(Column::new().push(Text::new("Hi!")))),
  );
  let msg = Message::new(
    Address::new("from@example.com".parse().unwrap()),
    vec![Address::new("to@example.com".parse().unwrap())],
    "Welcome — test",
    template,
  );

  transport(&server.uri()).send(&msg).await.unwrap();
}

#[tokio::test]
async fn http_200_with_success_false_is_api_error() {
  // The case with no Mailjet analog: HTTP is 200, but the body reports an
  // application-level failure. Must surface as Err, not a silent Ok.
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "success": false,
        "errors": [{
            "code": 10001,
            "message": "email.sending.error.invalid_request_schema",
        }],
        "result": null
    })))
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(matches!(err.kind(), CloudflareErrorKind::Api {
    code: 10001,
  }));
  // Display delegates to the source chain, which carries the provider message.
  assert!(
    err
      .to_string()
      .contains("email.sending.error.invalid_request_schema")
  );
}

#[tokio::test]
async fn sending_disabled_is_forbidden() {
  // HTTP 403 (code 10203, sending disabled) maps to its own kind — not the
  // generic Validation bucket, and not Auth (a credential refresh won't fix
  // a disabled account).
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
        "success": false,
        "errors": [{
            "code": 10203,
            "message": "email.sending.error.email.sending_disabled",
        }],
        "result": null
    })))
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(matches!(err.kind(), CloudflareErrorKind::Forbidden));
}

#[tokio::test]
async fn permanent_bounce_fails_the_send() {
  // Configured policy: any permanent bounce on an otherwise-200 response
  // turns the whole send into an error.
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "success": true,
        "errors": [],
        "result": {
            "delivered": ["ok@example.com"],
            "queued": [],
            "permanent_bounces": ["bounced@example.com"],
        }
    })))
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(matches!(err.kind(), CloudflareErrorKind::Bounced));
  assert!(err.to_string().contains("bounced@example.com"));

  // The delivered recipient must survive the error path, not just the bounce.
  let report = err.bounce_report().expect("Bounced carries a report");
  assert_eq!(report.delivered, ["ok@example.com"]);
  assert_eq!(report.permanent_bounces, ["bounced@example.com"]);
  assert!(report.queued.is_empty());
}

#[tokio::test]
async fn auth_failure_is_classified() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(matches!(err.kind(), CloudflareErrorKind::Auth));
}

#[tokio::test]
async fn rate_limited_parses_retry_after() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .respond_with(
      ResponseTemplate::new(429)
        .insert_header("Retry-After", "30")
        .set_body_string("Too Many Requests"),
    )
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(matches!(
    err.kind(),
    CloudflareErrorKind::RateLimited { retry_after } if *retry_after == Some(Duration::from_secs(30))
  ));
}

#[tokio::test]
async fn server_error_is_classified() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .respond_with(
      ResponseTemplate::new(503).set_body_string("Service Unavailable"),
    )
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(matches!(err.kind(), CloudflareErrorKind::Server {
    status: 503,
  }));
}

#[tokio::test]
async fn bad_request_is_validation() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(matches!(err.kind(), CloudflareErrorKind::Validation {
    status: 400,
  }));
  // A plain-text body carries no structured provider code.
  assert_eq!(err.provider_code(), None);
}

#[tokio::test]
async fn validation_surfaces_provider_code_and_message() {
  // Reproduces the real-world failure: HTTP 400 with a Cloudflare error
  // envelope. Retry classification stays status-based (Validation), but the
  // numeric code and machine message are now recoverable instead of buried in
  // a stringified body.
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path(SEND_PATH))
    .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
        "success": false,
        "messages": [],
        "errors": [{
            "code": 10202,
            "message": "email.sending.error.email.invalid",
        }],
        "result": null
    })))
    .mount(&server)
    .await;

  let err = transport(&server.uri())
    .send(&make_message())
    .await
    .unwrap_err();

  assert!(matches!(err.kind(), CloudflareErrorKind::Validation {
    status: 400,
  }));
  assert_eq!(err.provider_code(), Some(10202));
  assert_eq!(
    err.provider_message(),
    Some("email.sending.error.email.invalid")
  );
}
