//! Cloudflare Email Sending transport.
//!
//! Posts to Cloudflare's `/email/sending/send` REST endpoint with a Bearer
//! token. Unlike most HTTP providers, Cloudflare layers an application-level
//! success flag *over* the HTTP status: a `200` carrying `"success": false`
//! is still a failure, and a `200` whose `result.permanent_bounces` is
//! non-empty means the message was rejected for those recipients. `send`
//! checks both layers.
//!
//! Errors flow through [`CloudflareError`] — manteau's standard kind + source
//! shape — and implement [`TransportFailure`] so retry middleware classifies
//! failures by category. See the [`crate::transport`] module docs for the
//! error pattern.

use std::borrow::Cow;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
  message::Message,
  models::{Address, MessageId},
  templating::attributes::urls::Url,
  transport::{Receipt, Transport, TransportFailure},
};

/// Tunable defaults for [`CloudflareTransport`]. Construct via
/// [`CloudflareConfig::new`] and chain setters to override individual fields.
///
/// ```
/// # use std::time::Duration;
/// # use manteau::CloudflareConfig;
/// let config = CloudflareConfig::new().timeout(Duration::from_secs(60));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CloudflareConfig {
  /// Per-request total timeout (DNS + connect + TLS + send + receive).
  pub timeout: Duration,
}

impl CloudflareConfig {
  pub fn new() -> Self {
    Self {
      timeout: Duration::from_secs(30),
    }
  }

  /// Override the per-request timeout. Default 30 seconds.
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
  }
}

impl Default for CloudflareConfig {
  fn default() -> Self { Self::new() }
}

/// Cloudflare Email Sending transport — posts to the account-scoped
/// `/email/sending/send` endpoint with a Bearer API token.
///
/// Requires a Cloudflare **account ID** (it is part of the request path) and
/// an **API token** with email-sending permission. The `from` domain must be
/// onboarded to Email Sending first (`wrangler email sending enable
/// yourdomain.com`). Defaults to `https://api.cloudflare.com`; override via
/// [`base_url`] for sandbox or proxy environments (and for wiremock in tests).
///
/// One transport per process is the lazy choice — the internal
/// `reqwest::Client` pools connections, so reusing a single transport across
/// many sends is significantly cheaper than constructing one per request.
///
/// ```no_run
/// # use std::time::Duration;
/// # use manteau::{CloudflareConfig, CloudflareTransport};
/// let transport = CloudflareTransport::new("account-id", "api-token")
///   .config(CloudflareConfig::new().timeout(Duration::from_secs(60)));
/// ```
///
/// [`base_url`]: CloudflareTransport::base_url
#[derive(Debug, Clone)]
pub struct CloudflareTransport {
  account_id: String,
  api_token:  String,
  base_url:   Url,
  config:     CloudflareConfig,
  client:     reqwest::Client,
}

impl CloudflareTransport {
  /// Build with the required Cloudflare credentials. Defaults applied:
  ///
  /// - `base_url`: `https://api.cloudflare.com`
  /// - `config`: [`CloudflareConfig::default`] (30s timeout)
  /// - `client`: a fresh [`reqwest::Client`]
  ///
  /// Override any of these by chaining the corresponding setter.
  pub fn new(
    account_id: impl Into<String>,
    api_token: impl Into<String>,
  ) -> Self {
    Self {
      account_id: account_id.into(),
      api_token:  api_token.into(),
      base_url:   Url::try_parse("https://api.cloudflare.com")
        .expect("hardcoded URL is valid"),
      config:     CloudflareConfig::default(),
      client:     reqwest::Client::default(),
    }
  }

  /// Override the API host. Useful for sandbox environments, proxies, or
  /// fakes (wiremock in integration tests).
  pub fn base_url(mut self, base_url: Url) -> Self {
    self.base_url = base_url;
    self
  }

  /// Override the tunable defaults (timeout, etc.). See [`CloudflareConfig`].
  pub fn config(mut self, config: CloudflareConfig) -> Self {
    self.config = config;
    self
  }

  /// Override the reqwest client (for shared connection pools across many
  /// transports, custom middleware, etc.).
  pub fn client(mut self, client: reqwest::Client) -> Self {
    self.client = client;
    self
  }
}

/// Acknowledgement returned by [`CloudflareTransport::send`] for an accepted
/// post.
///
/// Cloudflare assigns no per-message IDs; it reports each recipient's outcome
/// instead. `ids` is therefore synthesized from the `delivered` and `queued`
/// recipients so cross-transport observability still has something to key on;
/// the raw buckets are exposed as fields for callers that need the
/// distinction. (A receipt only exists on success — any `permanent_bounces`
/// turns the send into an error — so `permanent_bounces` here is always the
/// empty vec, retained for forward-compatibility and `raw` parity.)
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CloudflareReceipt {
  /// Synthesized from `delivered` + `queued` recipient addresses — Cloudflare
  /// returns no provider-assigned message IDs.
  pub ids:               Vec<MessageId>,
  /// Recipients Cloudflare accepted and delivered immediately.
  pub delivered:         Vec<String>,
  /// Recipients Cloudflare accepted and queued for later delivery.
  pub queued:            Vec<String>,
  /// Recipients that hard-bounced. Always empty on a successful receipt (see
  /// the type docs); present for `raw` parity.
  pub permanent_bounces: Vec<String>,
  /// Full JSON response from Cloudflare. Escape hatch for fields not surfaced
  /// above; treat its schema as unstable and provider-controlled.
  pub raw:               serde_json::Value,
}

impl Receipt for CloudflareReceipt {
  fn ids(&self) -> &[MessageId] { &self.ids }
}

// ---------- Error machinery ----------

/// Error returned by [`CloudflareTransport::send`].
///
/// Follows manteau's standard kind + source pattern (see the
/// [`crate::transport`] module docs). Branch on [`CloudflareError::kind`] for
/// programmatic decisions; `Display` carries the upstream's message; the
/// original error is preserved in the `#[source]` chain for downcasting.
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub struct CloudflareError {
  kind:   CloudflareErrorKind,
  #[source]
  source: Box<dyn std::error::Error + Send + Sync>,
}

impl CloudflareError {
  /// The category of failure. Callers branch on this for retry policy; the
  /// human-readable message comes from the source chain.
  pub fn kind(&self) -> &CloudflareErrorKind { &self.kind }

  /// The per-recipient breakdown when this is a
  /// [`Bounced`](CloudflareErrorKind::Bounced) error — which recipients still
  /// delivered/queued alongside the ones that hard-bounced. `None` for every
  /// other kind. Use it to update suppression lists and avoid re-sending to
  /// recipients that already went out.
  pub fn bounce_report(&self) -> Option<&BounceReport> {
    self.source.downcast_ref::<BounceReport>()
  }

  /// Cloudflare's numeric error code, when the failure carried one — a non-2xx
  /// HTTP response whose body parsed, or a 200 with `success:false`. `None` for
  /// transport-level failures (network, render) that never reached Cloudflare
  /// or produced no code.
  ///
  /// This is a *diagnostic*: retry classification keys off the HTTP status (see
  /// [`TransportFailure`]), not this code. The documented code table is not
  /// exhaustive, so treat unrecognized codes as opaque.
  pub fn provider_code(&self) -> Option<i64> {
    if let Some(api) = self.source.downcast_ref::<ApiRejection>() {
      return Some(api.code);
    }
    self
      .source
      .downcast_ref::<HttpRejection>()
      .and_then(|h| h.code)
  }

  /// Cloudflare's machine-readable error message (e.g.
  /// `email.sending.error.email.invalid`), paired with [`provider_code`]. See
  /// that method for when it is present.
  ///
  /// [`provider_code`]: CloudflareError::provider_code
  pub fn provider_message(&self) -> Option<&str> {
    if let Some(api) = self.source.downcast_ref::<ApiRejection>() {
      return Some(&api.message);
    }
    self
      .source
      .downcast_ref::<HttpRejection>()
      .and_then(|h| h.message.as_deref())
  }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CloudflareErrorKind {
  /// Network-level failure (DNS, connection refused, TLS, timeout).
  #[error("network error")]
  Network,
  /// Cloudflare rejected the API token (HTTP 401).
  #[error("authentication rejected")]
  Auth,
  /// Cloudflare returned HTTP 403 — email sending is disabled for the
  /// account/zone (code `10203`), or the token lacks email-sending
  /// permission. Distinct from [`Auth`](Self::Auth) (a bad token) because a
  /// credential *refresh* generally won't fix it: a human must enable sending
  /// or widen the token's scope. Treated as neither transient nor auth nor
  /// message-rejected — branch on this kind directly to surface it.
  #[error("forbidden — sending disabled or insufficient permission")]
  Forbidden,
  /// Cloudflare rate-limited the request (HTTP 429). Carries the parsed
  /// `Retry-After` value when the response supplied one.
  #[error("rate limited")]
  RateLimited { retry_after: Option<Duration> },
  /// Cloudflare returned a server error (HTTP 5xx).
  #[error("provider server error ({status})")]
  Server { status: u16 },
  /// Request was malformed / failed validation (HTTP 400).
  #[error("request rejected ({status})")]
  Validation { status: u16 },
  /// HTTP 200 but the body carried `"success": false`. Defensive residual:
  /// Cloudflare normally signals failure with a real HTTP status (400/403/
  /// 429/500), so this path is rare. The numeric Cloudflare code (5-digit,
  /// e.g. `10001`) is carried for diagnostics/logging, not retry policy —
  /// the documented code table is "common", not exhaustive, so we do not
  /// branch retry decisions on it.
  #[error("provider reported failure (code {code})")]
  Api { code: i64 },
  /// HTTP 200 with `"success": true` but one or more recipients hard-bounced.
  /// Per manteau's configured policy, any permanent bounce fails the send.
  #[error("message rejected for one or more recipients")]
  Bounced,
  /// The 2xx response body did not parse as the expected JSON shape.
  #[error("failed to decode provider response")]
  Decode,
  /// The upstream message could not be rendered. Source chain holds the
  /// underlying [`RenderError`](crate::render::RenderError).
  #[error("failed to render message")]
  Render,
}

impl CloudflareErrorKind {
  /// Internal constructor — paired with a source error. `pub(crate)` so
  /// consumers cannot fabricate manteau errors.
  pub(crate) fn err(
    self,
    source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
  ) -> CloudflareError {
    CloudflareError {
      kind:   self,
      source: source.into(),
    }
  }
}

impl TransportFailure for CloudflareError {
  /// Retry-worthy failures: network blips, rate limits, and provider 5xx.
  /// Everything else is either caller-actionable or a bug.
  fn is_transient(&self) -> bool {
    matches!(
      self.kind,
      CloudflareErrorKind::Network
        | CloudflareErrorKind::RateLimited { .. }
        | CloudflareErrorKind::Server { .. }
    )
  }

  /// Only a rejected token (401) is an auth failure a credential refresh
  /// fixes. [`Forbidden`](CloudflareErrorKind::Forbidden) (403) is deliberately
  /// excluded — see its docs.
  fn is_auth(&self) -> bool { matches!(self.kind, CloudflareErrorKind::Auth) }

  /// The message/envelope itself is the problem: malformed request (400),
  /// a per-recipient hard bounce, or an application-level `success:false`.
  fn is_message_rejected(&self) -> bool {
    matches!(
      self.kind,
      CloudflareErrorKind::Validation { .. }
        | CloudflareErrorKind::Bounced
        | CloudflareErrorKind::Api { .. }
    )
  }

  /// Surface the parsed `Retry-After` from a 429, when Cloudflare supplied it.
  fn retry_after(&self) -> Option<Duration> {
    match &self.kind {
      CloudflareErrorKind::RateLimited { retry_after } => *retry_after,
      _ => None,
    }
  }
}

// Source error capturing the HTTP status and response body — held behind
// CloudflareError's `source` so the body survives in the Display chain. When
// the body parsed as Cloudflare's error envelope, the first error's `code` and
// `message` are split out so `provider_code`/`provider_message` can surface
// them without re-parsing.
#[derive(Debug, thiserror::Error)]
#[error("status {status}: {body}")]
struct HttpRejection {
  status:  u16,
  code:    Option<i64>,
  message: Option<String>,
  body:    String,
}

// Pull the first `{code, message}` out of a Cloudflare error envelope body.
// Returns `(None, None)` when the body isn't the expected JSON shape (e.g. a
// plain-text gateway error), so a non-JSON body degrades gracefully.
fn parse_error_envelope(body: &str) -> (Option<i64>, Option<String>) {
  match serde_json::from_str::<ApiResponse>(body) {
    Ok(parsed) => match parsed.errors.into_iter().next() {
      Some(e) => (Some(e.code), Some(e.message)),
      None => (None, None),
    },
    Err(_) => (None, None),
  }
}

// Source error for a 200 response that reported application-level failure.
#[derive(Debug, thiserror::Error)]
#[error("cloudflare error {code}: {message}")]
struct ApiRejection {
  code:    i64,
  message: String,
}

/// Per-recipient outcome for a send Cloudflare accepted at the HTTP layer but
/// that hard-bounced for one or more recipients. Attached as the source of a
/// [`CloudflareErrorKind::Bounced`] error and recoverable via
/// [`CloudflareError::bounce_report`], so a caller can see which recipients
/// still went out alongside the ones that bounced.
#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
#[error(
  "permanent bounces: {permanent_bounces:?} (delivered: {delivered:?}, \
   queued: {queued:?})"
)]
pub struct BounceReport {
  /// Recipients Cloudflare accepted and delivered immediately.
  pub delivered:         Vec<String>,
  /// Recipients Cloudflare accepted and queued for later delivery.
  pub queued:            Vec<String>,
  /// Recipients that hard-bounced — the reason this send is an error.
  pub permanent_bounces: Vec<String>,
}

// ---------- Header encoding ----------

/// Encode an RFC 5322 header value (subject, address display name) as one or
/// more RFC 2047 "Q" encoded-words when it contains anything outside printable
/// US-ASCII. Pure printable-ASCII values pass through borrowed (no allocation).
///
/// Cloudflare's `/email/sending/send` endpoint places `subject` verbatim into
/// the Subject header, which is ASCII-only unless RFC 2047-encoded — a raw
/// non-ASCII byte (e.g. an em-dash) is rejected with `10202
/// email.sending.error.email.invalid`. The HTML/text *bodies* are MIME parts
/// with their own charset and are deliberately left untouched.
///
/// The single "not printable ASCII" predicate also closes CR/LF header
/// injection: any control byte forces encoding, so a newline in a caller-
/// supplied subject becomes `=0D=0A` rather than reaching the raw header.
fn encode_header_word(value: &str) -> Cow<'_, str> {
  // Fast path: emit verbatim when every byte is already printable ASCII.
  if value.bytes().all(|b| (0x20..=0x7e).contains(&b)) {
    return Cow::Borrowed(value);
  }

  const PREFIX: &str = "=?UTF-8?Q?";
  const SUFFIX: &str = "?=";
  // RFC 2047 §2: an encoded-word is at most 75 characters in total.
  const MAX_TEXT: usize = 75 - PREFIX.len() - SUFFIX.len(); // 63

  let mut words: Vec<String> = Vec::new();
  let mut cur = String::new();

  for ch in value.chars() {
    // Encode one whole character atomically so no encoded-word ever splits a
    // multibyte sequence — each word must decode to valid UTF-8 (RFC 2047 §5).
    let mut piece = String::new();
    match ch {
      ' ' => piece.push('_'),
      // Printable ASCII may appear literally in Q-text, except the three
      // characters that always carry meaning there: '=', '?', '_'.
      c if (0x20..=0x7e).contains(&(c as u32)) && !matches!(c, '=' | '?' | '_') => {
        piece.push(c);
      }
      c => {
        let mut buf = [0u8; 4];
        for b in c.encode_utf8(&mut buf).as_bytes() {
          piece.push_str(&format!("={b:02X}"));
        }
      }
    }
    if cur.len() + piece.len() > MAX_TEXT {
      words.push(std::mem::take(&mut cur));
    }
    cur.push_str(&piece);
  }
  if !cur.is_empty() {
    words.push(cur);
  }

  // Adjacent encoded-words are separated by a single space; decoders drop the
  // whitespace between them (RFC 2047 §6.2). Cloudflare folds the header.
  Cow::Owned(
    words
      .into_iter()
      .map(|w| format!("{PREFIX}{w}{SUFFIX}"))
      .collect::<Vec<_>>()
      .join(" "),
  )
}

// ---------- Serialization wire shape (private to this module) ----------

#[derive(Serialize)]
struct Payload<'a> {
  from:    AddrObj<'a>,
  // Cloudflare's REST API takes bare address strings for to/cc/bcc — recipient
  // display names cannot be expressed and are dropped.
  to:      Vec<&'a str>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  cc:      Vec<&'a str>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  bcc:     Vec<&'a str>,
  // RFC 2047-encoded when non-ASCII (see `encode_header_word`); borrowed
  // otherwise. The bodies below are MIME parts and stay raw UTF-8.
  subject: Cow<'a, str>,
  html:    &'a str,
  text:    &'a str,
}

// `from` (and reply_to) take an object with `address` + `name` — note the
// `address` key, which differs from the Workers binding's `email`.
#[derive(Serialize)]
struct AddrObj<'a> {
  address: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  name:    Option<Cow<'a, str>>,
}

impl<'a> From<&'a Address> for AddrObj<'a> {
  fn from(a: &'a Address) -> Self {
    AddrObj {
      address: a.email.as_str(),
      // The display name is an RFC 5322 phrase — same header-encoding rule
      // as the subject.
      name:    a.name.as_deref().map(encode_header_word),
    }
  }
}

// ---------- Response wire shape (private to this module) ----------

#[derive(Deserialize)]
struct ApiResponse {
  success: bool,
  #[serde(default)]
  errors:  Vec<ApiError>,
  result:  Option<SendResult>,
}

#[derive(Deserialize)]
struct ApiError {
  code:    i64,
  message: String,
}

#[derive(Deserialize, Default)]
struct SendResult {
  #[serde(default)]
  delivered:         Vec<String>,
  #[serde(default)]
  queued:            Vec<String>,
  #[serde(default)]
  permanent_bounces: Vec<String>,
}

// ---------- Implementation ----------

/// Parse a `Retry-After` header expressed as a number of seconds. Cloudflare
/// uses the delay-seconds form on 429s; the HTTP-date form is ignored.
fn parse_retry_after(resp: &reqwest::Response) -> Option<Duration> {
  resp
    .headers()
    .get(reqwest::header::RETRY_AFTER)
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.trim().parse::<u64>().ok())
    .map(Duration::from_secs)
}

#[async_trait]
impl Transport for CloudflareTransport {
  type Error = CloudflareError;
  type Receipt = CloudflareReceipt;

  #[tracing::instrument(skip_all, fields(base_url = %self.base_url.as_str()))]
  async fn send(
    &self,
    message: &Message,
  ) -> Result<CloudflareReceipt, CloudflareError> {
    let rendered = message
      .render()
      .map_err(|e| CloudflareErrorKind::Render.err(e))?;

    let payload = Payload {
      from:    (&message.from).into(),
      to:      message.to.iter().map(|a| a.email.as_str()).collect(),
      cc:      message.cc.iter().map(|a| a.email.as_str()).collect(),
      bcc:     message.bcc.iter().map(|a| a.email.as_str()).collect(),
      subject: encode_header_word(&message.subject),
      html:    &rendered.html,
      text:    &rendered.text,
    };

    let url = format!(
      "{}/client/v4/accounts/{}/email/sending/send",
      self.base_url.as_str(),
      self.account_id,
    );
    let resp = self
      .client
      .post(&url)
      .timeout(self.config.timeout)
      .bearer_auth(&self.api_token)
      .json(&payload)
      .send()
      .await
      .map_err(|e| CloudflareErrorKind::Network.err(e))?;

    // ---- Layer 1: HTTP status ----
    let status = resp.status();
    if !status.is_success() {
      let status_u16 = status.as_u16();
      let retry_after = parse_retry_after(&resp);
      let body = resp.text().await.unwrap_or_default();
      let (code, message) = parse_error_envelope(&body);
      let rejection = HttpRejection {
        status: status_u16,
        code,
        message,
        body,
      };
      let kind = match status_u16 {
        401 => CloudflareErrorKind::Auth,
        403 => CloudflareErrorKind::Forbidden,
        429 => CloudflareErrorKind::RateLimited { retry_after },
        500..=599 => CloudflareErrorKind::Server { status: status_u16 },
        // 400 and any other 4xx: the request is the problem, no retry.
        _ => CloudflareErrorKind::Validation { status: status_u16 },
      };
      tracing::error!(kind = ?kind, "cloudflare send failed (http)");
      return Err(kind.err(rejection));
    }

    // ---- Layer 2: application-level success flag in the JSON body ----
    let raw: serde_json::Value = resp
      .json()
      .await
      .map_err(|e| CloudflareErrorKind::Decode.err(e))?;
    let parsed: ApiResponse = serde_json::from_value(raw.clone())
      .map_err(|e| CloudflareErrorKind::Decode.err(e))?;

    if !parsed.success {
      let first = parsed.errors.into_iter().next().unwrap_or(ApiError {
        code:    0,
        message: "unknown error".into(),
      });
      let kind = CloudflareErrorKind::Api { code: first.code };
      let rejection = ApiRejection {
        code:    first.code,
        message: first.message,
      };
      tracing::error!(kind = ?kind, "cloudflare send failed (api)");
      return Err(kind.err(rejection));
    }

    let result = parsed.result.unwrap_or_default();

    // ---- Configured policy: any permanent bounce fails the send ----
    if !result.permanent_bounces.is_empty() {
      tracing::error!(
        bounces = ?result.permanent_bounces,
        "cloudflare reported permanent bounces"
      );
      // Carry the full per-recipient breakdown on the error so the caller can
      // see who still went out; recover it via
      // `CloudflareError::bounce_report`.
      return Err(CloudflareErrorKind::Bounced.err(BounceReport {
        delivered:         result.delivered,
        queued:            result.queued,
        permanent_bounces: result.permanent_bounces,
      }));
    }

    // ids() has no real provider IDs to expose; synthesize from accepted
    // recipients so cross-transport observability still has a handle.
    let ids = result
      .delivered
      .iter()
      .chain(result.queued.iter())
      .map(|addr| MessageId::new(addr.clone()))
      .collect();

    Ok(CloudflareReceipt {
      ids,
      delivered: result.delivered,
      queued: result.queued,
      permanent_bounces: result.permanent_bounces,
      raw,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn err(kind: CloudflareErrorKind) -> CloudflareError {
    CloudflareError {
      kind,
      source: "test".into(),
    }
  }

  // ---- Header encoding (RFC 2047) ----

  #[test]
  fn ascii_subject_passes_through_borrowed() {
    let out = encode_header_word("Plain ASCII subject");
    assert!(matches!(out, Cow::Borrowed(_)));
    assert_eq!(out, "Plain ASCII subject");
  }

  #[test]
  fn em_dash_subject_is_q_encoded() {
    // The xodus welcome-email case: U+2014 em-dash → =E2=80=94, spaces → '_'.
    let out = encode_header_word("A — B");
    assert_eq!(out, "=?UTF-8?Q?A_=E2=80=94_B?=");
  }

  #[test]
  fn non_ascii_output_is_pure_ascii() {
    // Whatever the input, the wire value must be ASCII so the Subject header
    // is valid — this is the property Cloudflare rejected (10202) without.
    let out = encode_header_word("Café — Zürich — São Paulo");
    assert!(out.is_ascii(), "encoded header must be ASCII: {out}");
  }

  #[test]
  fn q_specials_are_escaped_when_encoding() {
    // '=', '?', '_' must be escaped inside an encoded-word; they only appear
    // here because the non-ASCII 'é' forces the whole value to encode.
    let out = encode_header_word("=?_é");
    assert_eq!(out, "=?UTF-8?Q?=3D=3F=5F=C3=A9?=");
  }

  #[test]
  fn control_bytes_force_encoding_no_raw_crlf() {
    // Header-injection guard: a CR/LF in a caller-supplied subject becomes
    // =0D=0A rather than reaching the raw header.
    let out = encode_header_word("Subject\r\nBcc: evil@example.com");
    assert!(out.is_ascii());
    assert!(!out.contains('\r') && !out.contains('\n'));
    assert!(out.contains("=0D=0A"));
  }

  #[test]
  fn long_value_splits_into_words_within_75_chars() {
    // A long non-ASCII value must split into multiple encoded-words, each
    // ≤ 75 chars, none splitting a multibyte character.
    let input = "é".repeat(200);
    let out = encode_header_word(&input);
    let words: Vec<&str> = out.split(' ').collect();
    assert!(words.len() > 1, "expected multiple encoded-words");
    for w in words {
      assert!(w.len() <= 75, "encoded-word exceeds 75 chars: {} ({})", w.len(), w);
      assert!(w.starts_with("=?UTF-8?Q?") && w.ends_with("?="));
    }
  }

  // Classification spec for `impl TransportFailure` — one test per kind.

  #[test]
  fn network_is_transient() {
    let e = err(CloudflareErrorKind::Network);
    assert!(e.is_transient());
    assert!(!e.is_auth());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn auth_is_only_auth() {
    let e = err(CloudflareErrorKind::Auth);
    assert!(!e.is_transient());
    assert!(e.is_auth());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn rate_limited_is_transient_and_surfaces_retry_after() {
    let e = err(CloudflareErrorKind::RateLimited {
      retry_after: Some(Duration::from_secs(30)),
    });
    assert!(e.is_transient());
    assert_eq!(e.retry_after(), Some(Duration::from_secs(30)));
  }

  #[test]
  fn server_5xx_is_transient() {
    let e = err(CloudflareErrorKind::Server { status: 503 });
    assert!(e.is_transient());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn validation_is_rejected_not_transient() {
    let e = err(CloudflareErrorKind::Validation { status: 400 });
    assert!(!e.is_transient());
    assert!(e.is_message_rejected());
  }

  #[test]
  fn bounced_is_message_rejected() {
    let e = err(CloudflareErrorKind::Bounced);
    assert!(!e.is_transient());
    assert!(e.is_message_rejected());
  }

  #[test]
  fn bounce_report_recovers_full_breakdown() {
    let report = BounceReport {
      delivered:         vec!["ok@example.com".into()],
      queued:            vec!["later@example.com".into()],
      permanent_bounces: vec!["bad@example.com".into()],
    };
    let e = CloudflareErrorKind::Bounced.err(report);

    let recovered = e.bounce_report().expect("Bounced carries a report");
    assert_eq!(recovered.delivered, ["ok@example.com"]);
    assert_eq!(recovered.queued, ["later@example.com"]);
    assert_eq!(recovered.permanent_bounces, ["bad@example.com"]);
  }

  #[test]
  fn bounce_report_is_source_keyed_not_kind_keyed() {
    // The accessor downcasts the source, so it returns None whenever the
    // source isn't a BounceReport — including a Bounced kind whose source is
    // something else. In production only `send`'s bounce path pairs the two,
    // so the distinction is invisible to callers; this pins the mechanism.
    assert!(err(CloudflareErrorKind::Auth).bounce_report().is_none());
    assert!(err(CloudflareErrorKind::Bounced).bounce_report().is_none());
  }

  #[test]
  fn forbidden_is_neither() {
    // 403: sending disabled / insufficient permission — not transient, not an
    // auth-refresh, not a message problem. Surfaced via kind() only.
    let e = err(CloudflareErrorKind::Forbidden);
    assert!(!e.is_transient());
    assert!(!e.is_auth());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn api_failure_is_message_rejected() {
    let e = err(CloudflareErrorKind::Api { code: 10001 });
    assert!(!e.is_transient());
    assert!(!e.is_auth());
    assert!(e.is_message_rejected());
  }

  #[test]
  fn decode_failure_is_neither() {
    // An unparseable 2xx body is a bug to surface, not a retry/auth/rejection.
    let e = err(CloudflareErrorKind::Decode);
    assert!(!e.is_transient());
    assert!(!e.is_auth());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn non_rate_limit_has_no_retry_after() {
    assert_eq!(err(CloudflareErrorKind::Network).retry_after(), None);
    assert_eq!(
      err(CloudflareErrorKind::Server { status: 500 }).retry_after(),
      None
    );
  }

  #[test]
  fn render_failure_is_neither() {
    let e = err(CloudflareErrorKind::Render);
    assert!(!e.is_transient());
    assert!(!e.is_auth());
    assert!(!e.is_message_rejected());
  }

  #[test]
  fn provider_code_and_message_from_http_rejection() {
    // Reproduces the real-world 400: code 10202 / email.invalid.
    let e =
      CloudflareErrorKind::Validation { status: 400 }.err(HttpRejection {
        status:  400,
        code:    Some(10202),
        message: Some("email.sending.error.email.invalid".into()),
        body:    "{\"errors\":[{\"code\":10202}]}".into(),
      });
    assert_eq!(e.provider_code(), Some(10202));
    assert_eq!(
      e.provider_message(),
      Some("email.sending.error.email.invalid")
    );
  }

  #[test]
  fn provider_code_from_api_rejection() {
    let e = CloudflareErrorKind::Api { code: 10001 }.err(ApiRejection {
      code:    10001,
      message: "email.sending.error.invalid_request_schema".into(),
    });
    assert_eq!(e.provider_code(), Some(10001));
    assert_eq!(
      e.provider_message(),
      Some("email.sending.error.invalid_request_schema")
    );
  }

  #[test]
  fn provider_code_none_without_structured_error() {
    // A non-JSON body (e.g. a gateway error page) yields no structured code,
    // and transport-level failures have no provider code at all.
    let e =
      CloudflareErrorKind::Validation { status: 400 }.err(HttpRejection {
        status:  400,
        code:    None,
        message: None,
        body:    "Bad Request".into(),
      });
    assert_eq!(e.provider_code(), None);
    assert_eq!(e.provider_message(), None);
    assert_eq!(err(CloudflareErrorKind::Network).provider_code(), None);
  }

  #[test]
  fn parse_error_envelope_extracts_first_code() {
    let body = r#"{"success":false,"messages":[],"errors":[{"code":10202,"message":"email.sending.error.email.invalid"}],"result":null}"#;
    let (code, msg) = parse_error_envelope(body);
    assert_eq!(code, Some(10202));
    assert_eq!(msg.as_deref(), Some("email.sending.error.email.invalid"));
    // A non-JSON body degrades gracefully to no code/message.
    assert_eq!(parse_error_envelope("Bad Request"), (None, None));
  }
}
