//! Reusable submission behavior composed exclusively from provider and HTTP
//! ports.

use std::time::Duration;

use async_trait::async_trait;

use crate::{
  Acceptance, IdempotentTransport, PreparedMessage, Receipt, ReplayError,
  Submission, Transport, TransportFailure,
  http::{Credentials, Http, HttpConfig, HttpError, HttpRequest, HttpResponse},
};

/// Provider-specific protocol. Implementations describe their inputs and
/// results; the sender owns dispatch order, shared HTTP failures, and replay
/// checks.
pub trait HttpProvider: Send + Sync {
  /// Payload representation, borrowing immutable prepared content where
  /// possible.
  type Payload<'a>: serde::Serialize + Send
  where
    Self: 'a;

  /// Evidence meaningful for this provider.
  type Receipt: Receipt;

  /// Provider-specific rejection or ambiguous response.
  type Error: TransportFailure;

  /// Validated fixed destination and request bounds.
  fn config(&self) -> &HttpConfig;

  /// Validated authentication material bound to this provider instance.
  fn credentials(&self) -> &Credentials;

  /// Stable protocol identity and serialization version. A change invalidates
  /// old keyed submissions before dispatch rather than risking a changed body.
  fn protocol_scope(&self) -> &'static str;

  /// Admit provider-specific inputs and construct their payload as one
  /// operation.
  fn request<'a>(
    &'a self,
    message: &'a PreparedMessage,
  ) -> Result<Self::Payload<'a>, Self::Error>;

  /// Declare status evidence documented by this provider. Unknown statuses do
  /// not establish non-acceptance. Provider-specific statuses reach `receipt`.
  fn status_policy(&self) -> StatusPolicy;

  /// Interpret a response that the provider declares it can handle. Missing or
  /// contradictory evidence must remain uncertain, never become fabricated
  /// success.
  fn receipt(
    &self,
    response: HttpResponse,
    message: &PreparedMessage,
  ) -> Result<Self::Receipt, Self::Error>;
}

/// A provider whose documented protocol deduplicates identical requests.
pub trait IdempotentProvider: HttpProvider {
  /// Minimum retention of the original identity from the first request.
  fn retention(&self) -> Duration;
}

/// Configured submission capability. Owns its provider and physical HTTP port.
/// Every provider uses this same dispatch and recovery procedure.
pub struct Sender<P, H> {
  provider: P,
  http:     H,
  scope:    String,
}

impl<P: HttpProvider, H: Http> Sender<P, H> {
  /// Bind the protocol to an HTTP implementation, preserving endpoint and
  /// credentials.
  pub fn new(provider: P, http: H) -> Self {
    let scope = format!(
      "{}:{}",
      provider.protocol_scope(),
      provider.credentials().scope(provider.config().endpoint())
    );
    Self {
      provider,
      http,
      scope,
    }
  }

  async fn submit(
    &self,
    message: &PreparedMessage,
    key: Option<&crate::IdempotencyKey>,
  ) -> Result<P::Receipt, SendError<P::Error>> {
    let payload = self
      .provider
      .request(message)
      .map_err(SendError::Provider)?;
    let request = HttpRequest::new(
      self.provider.config(),
      self.provider.credentials(),
      key,
      &payload,
    )
    .map_err(SendError::Http)?;
    let response = self.http.post(request).await.map_err(SendError::Http)?;

    let policy = self.provider.status_policy();
    if !policy.handles(response.status()) {
      let acceptance = if policy.proves_not_accepted(response.status()) {
        Acceptance::NotAccepted
      } else {
        Acceptance::Unknown
      };
      return Err(SendError::Status {
        status: response.status(),
        retry_after: response.retry_after(),
        acceptance,
      });
    }

    self
      .provider
      .receipt(response, message)
      .map_err(SendError::Provider)
  }
}

impl<P, H> std::fmt::Debug for Sender<P, H> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Sender([redacted])")
  }
}

#[async_trait]
impl<P: HttpProvider, H: Http> Transport for Sender<P, H> {
  type Error = SendError<P::Error>;
  type Receipt = P::Receipt;

  #[tracing::instrument(skip_all)]
  async fn send(
    &self,
    message: &PreparedMessage,
  ) -> Result<P::Receipt, Self::Error> {
    self.submit(message, None).await
  }
}

#[async_trait]
impl<P: IdempotentProvider, H: Http> IdempotentTransport for Sender<P, H> {
  fn retention(&self) -> Duration {
    self.provider.retention()
  }

  fn scope(&self) -> &str {
    &self.scope
  }

  fn request_timeout(&self) -> Duration {
    self.provider.config().request_timeout()
  }

  async fn send_idempotent(
    &self,
    submission: &Submission,
  ) -> Result<P::Receipt, Self::Error> {
    submission.check(self).map_err(SendError::Replay)?;
    self
      .submit(submission.message(), Some(submission.key()))
      .await
  }
}

/// Failures retain their layer and acceptance evidence without disclosing mail.
#[derive(thiserror::Error)]
pub enum SendError<E: TransportFailure> {
  /// Physical request or response failure.
  #[error("HTTP submission failed")]
  Http(#[source] HttpError),

  /// Shared HTTP status classification. Unknown status outcomes stay uncertain.
  #[error("provider returned HTTP {status}")]
  Status {
    /// Observed status.
    status:      u16,
    /// Requested minimum delay.
    retry_after: Option<Duration>,
    /// Provider-specific acceptance evidence for this status.
    acceptance:  Acceptance,
  },

  /// Provider-specific protocol evidence; inspect the source's typed
  /// classification.
  #[error("provider submission failed")]
  Provider(#[source] E),

  /// Scope or retention no longer permits another keyed request.
  #[error("submission cannot be replayed safely")]
  Replay(#[source] ReplayError),
}

impl<E: TransportFailure> std::fmt::Debug for SendError<E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Display::fmt(self, f)
  }
}

impl<E: TransportFailure> TransportFailure for SendError<E> {
  fn is_transient(&self) -> bool {
    match self {
      Self::Provider(error) => error.is_transient(),
      Self::Status {
        status: 429 | 500..=599,
        ..
      }
      | Self::Http(HttpError::Network(_)) => true,
      _ => false,
    }
  }

  fn is_auth(&self) -> bool {
    match self {
      Self::Provider(error) => error.is_auth(),
      Self::Status {
        status: 401 | 403, ..
      } => true,
      _ => false,
    }
  }

  fn is_message_rejected(&self) -> bool {
    match self {
      Self::Provider(error) => error.is_message_rejected(),
      Self::Status {
        status: 400 | 413 | 422,
        acceptance: Acceptance::NotAccepted,
        ..
      } => true,
      _ => false,
    }
  }

  fn acceptance(&self) -> Acceptance {
    match self {
      Self::Provider(error) => error.acceptance(),
      Self::Http(HttpError::Configuration | HttpError::Encode(_)) => {
        Acceptance::NotAccepted
      }
      Self::Status { acceptance, .. } => *acceptance,
      _ => Acceptance::Unknown,
    }
  }

  fn retry_after(&self) -> Option<Duration> {
    match self {
      Self::Provider(error) => error.retry_after(),
      Self::Status { retry_after, .. } => *retry_after,
      _ => None,
    }
  }
}

/// Provider-declared interpretation boundaries. Status handling is shared;
/// whether a status proves rejection comes from the provider's documented
/// protocol.
#[derive(Debug, Clone, Copy)]
pub struct StatusPolicy {
  handled:      &'static [u16],
  not_accepted: &'static [u16],
}

impl StatusPolicy {
  /// Declare disjoint statuses requiring provider interpretation and statuses
  /// that prove rejection. Invalid policy literals fail during const evaluation.
  pub const fn new(handled: &'static [u16], not_accepted: &'static [u16]) -> Self {
    let mut i = 0;
    while i < handled.len() {
      assert!(handled[i] >= 100 && handled[i] <= 599, "invalid handled HTTP status");
      let mut j = 0;
      while j < not_accepted.len() {
        assert!(handled[i] != not_accepted[j], "overlapping status policy");
        j += 1;
      }
      i += 1;
    }

    let mut i = 0;
    while i < not_accepted.len() {
      assert!(not_accepted[i] >= 100 && not_accepted[i] <= 599, "invalid rejection HTTP status");
      i += 1;
    }

    Self { handled, not_accepted }
  }

  /// Whether this status requires provider-specific body interpretation.
  pub fn handles(&self, status: u16) -> bool {
    self.handled.contains(&status)
  }

  /// Whether this status proves that no recipient was accepted.
  pub fn proves_not_accepted(&self, status: u16) -> bool {
    self.not_accepted.contains(&status)
  }
}
