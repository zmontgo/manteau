use std::{
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use async_trait::async_trait;
use manteau_core::{
  Acceptance, Address, Envelope, HeaderText, HttpProvider, IdempotencyKey,
  IdempotentProvider, IdempotentTransport, MessageId, PreparedMessage, Receipt,
  Recipients, Rendered, SendError, Sender, StatusPolicy, Submission, Transport,
  TransportFailure,
  http::{Credentials, Http, HttpConfig, HttpError, HttpRequest, HttpResponse},
};

struct CountingHttp {
  calls:  Arc<AtomicUsize>,
  status: u16,
}

#[async_trait]
impl Http for CountingHttp {
  async fn post(
    &self,
    request: HttpRequest<'_>,
  ) -> Result<HttpResponse, HttpError> {
    let payload: serde_json::Value =
      serde_json::from_slice(request.body()).unwrap();
    assert_eq!(payload["subject"], "A subject");
    self.calls.fetch_add(1, Ordering::SeqCst);
    request.response(self.status, None, b"{}".to_vec())
  }
}

struct AdmissionProtocol {
  config:         HttpConfig,
  credentials:    Credentials,
  admit:          bool,
  protocol_scope: &'static str,
}

#[derive(Debug)]
struct Accepted;

impl Receipt for Accepted {
  fn ids(&self) -> &[MessageId] {
    &[]
  }
}

#[derive(Debug, thiserror::Error)]
#[error("message was not admitted")]
struct NotAdmitted;

impl TransportFailure for NotAdmitted {
  fn is_transient(&self) -> bool {
    false
  }

  fn is_auth(&self) -> bool {
    false
  }

  fn is_message_rejected(&self) -> bool {
    true
  }

  fn acceptance(&self) -> Acceptance {
    Acceptance::NotAccepted
  }
}

#[derive(serde::Serialize)]
struct WireRequest<'a> {
  subject: &'a str,
}

impl HttpProvider for AdmissionProtocol {
  type Error = NotAdmitted;
  type Payload<'a> = WireRequest<'a>;
  type Receipt = Accepted;

  fn config(&self) -> &HttpConfig {
    &self.config
  }

  fn credentials(&self) -> &Credentials {
    &self.credentials
  }

  fn protocol_scope(&self) -> &'static str {
    self.protocol_scope
  }

  fn status_policy(&self) -> StatusPolicy {
    const POLICY: StatusPolicy = StatusPolicy::new(&[201], &[400]);
    POLICY
  }

  fn request<'a>(
    &'a self,
    message: &'a PreparedMessage,
  ) -> Result<WireRequest<'a>, NotAdmitted> {
    if !self.admit {
      return Err(NotAdmitted);
    }
    Ok(WireRequest {
      subject: message.envelope().subject(),
    })
  }

  fn receipt(
    &self,
    _: HttpResponse,
    _: &PreparedMessage,
  ) -> Result<Accepted, NotAdmitted> {
    Ok(Accepted)
  }
}

impl IdempotentProvider for AdmissionProtocol {
  fn retention(&self) -> Duration {
    Duration::from_secs(86400)
  }
}

fn message() -> PreparedMessage {
  PreparedMessage::new(
    Envelope::new(
      Address::new("from@example.com".parse().unwrap()),
      Recipients::to(Address::new("to@example.com".parse().unwrap())),
      HeaderText::new("A subject").unwrap(),
    ),
    Rendered::new("<p>Body</p>", "Body").unwrap(),
  )
}

fn sender(
  calls: Arc<AtomicUsize>,
  status: u16,
  admit: bool,
  token: &str,
  protocol_scope: &'static str,
) -> Sender<AdmissionProtocol, CountingHttp> {
  Sender::new(
    AdmissionProtocol {
      config: HttpConfig::new("https://example.com/send").unwrap(),
      credentials: Credentials::bearer(token).unwrap(),
      admit,
      protocol_scope,
    },
    CountingHttp { calls, status },
  )
}

#[tokio::test]
async fn admission_precedes_physical_dispatch() {
  let calls = Arc::new(AtomicUsize::new(0));
  let blocked = sender(calls.clone(), 201, false, "token", "test-protocol/1");
  let error = blocked.send(&message()).await.unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::NotAccepted);
  assert_eq!(calls.load(Ordering::SeqCst), 0);

  let allowed = sender(calls.clone(), 201, true, "token", "test-protocol/1");
  allowed.send(&message()).await.unwrap();
  assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn status_evidence_and_replay_scope_prevent_blind_dispatch() {
  let calls = Arc::new(AtomicUsize::new(0));
  let original = sender(
    calls.clone(),
    503,
    true,
    "original-token",
    "test-protocol/1",
  );
  let submission = Submission::new(
    IdempotencyKey::try_from("key".to_owned()).unwrap(),
    message(),
    &original,
  );
  let error = original.send_idempotent(&submission).await.unwrap_err();
  assert_eq!(error.acceptance(), Acceptance::Unknown);
  assert_eq!(calls.load(Ordering::SeqCst), 1);

  let changed = sender(
    calls.clone(),
    201,
    true,
    "different-token",
    "test-protocol/1",
  );
  let error = changed.send_idempotent(&submission).await.unwrap_err();
  assert!(matches!(error, SendError::Replay(_)));
  assert_eq!(calls.load(Ordering::SeqCst), 1);

  let upgraded = sender(
    calls.clone(),
    201,
    true,
    "original-token",
    "test-protocol/2",
  );
  let error = upgraded.send_idempotent(&submission).await.unwrap_err();
  assert!(matches!(error, SendError::Replay(_)));
  assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn expired_persisted_submission_never_dispatches() {
  let calls = Arc::new(AtomicUsize::new(0));
  let mail = sender(calls.clone(), 201, true, "token", "test-protocol/1");
  let submission = Submission::new(
    IdempotencyKey::try_from("key".to_owned()).unwrap(),
    message(),
    &mail,
  );
  let mut persisted = serde_json::to_value(&submission).unwrap();
  persisted["started"] = serde_json::json!({
    "secs_since_epoch": 0,
    "nanos_since_epoch": 0
  });
  let expired: Submission = serde_json::from_value(persisted).unwrap();

  let error = mail.send_idempotent(&expired).await.unwrap_err();
  assert!(matches!(error, SendError::Replay(_)));
  assert_eq!(calls.load(Ordering::SeqCst), 0);
}
