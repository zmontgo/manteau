//! Reqwest implementation of Manteau's HTTP port. Protocol decisions and send
//! orchestration live in core; this driver owns connection pooling and I/O
//! bounds.

use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use manteau_core::http::{Http, HttpError, HttpRequest, HttpResponse};

/// Pooled HTTP driver with redirects and automatic retries disabled.
#[derive(Clone)]
pub struct HttpClient {
  client: reqwest::Client,
}

impl HttpClient {
  /// Build reusable I/O capability. Each request supplies its validated
  /// endpoint and timeout; external clients cannot bypass this driver's fixed
  /// policy.
  pub fn new() -> Result<Self, HttpError> {
    let client = reqwest::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .retry(reqwest::retry::never())
      .build()
      .map_err(|_| HttpError::Configuration)?;

    Ok(Self { client })
  }
}

impl std::fmt::Debug for HttpClient {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("HttpClient")
  }
}

#[async_trait]
impl Http for HttpClient {
  async fn post(
    &self,
    request: HttpRequest<'_>,
  ) -> Result<HttpResponse, HttpError> {
    let (config, credentials, key, body) = request.into_parts();
    let mut outgoing = self
      .client
      .post(config.endpoint())
      .timeout(config.request_timeout())
      .header(reqwest::header::CONTENT_TYPE, "application/json")
      .body(body);

    if let Some(token) = credentials.bearer_token() {
      outgoing = outgoing.bearer_auth(token);
    }
    if let Some((key, secret)) = credentials.basic_pair() {
      outgoing = outgoing.basic_auth(key, Some(secret));
    }
    if let Some(key) = key {
      let mut value = reqwest::header::HeaderValue::from_str(key.as_str())
        .expect("validated idempotency key");
      value.set_sensitive(true);
      outgoing = outgoing.header("Idempotency-Key", value);
    }

    let mut response = outgoing
      .send()
      .await
      .map_err(|error| HttpError::Network(Box::new(error.without_url())))?;
    let status = response.status().as_u16();
    let retry_after = response
      .headers()
      .get(reqwest::header::RETRY_AFTER)
      .and_then(|header| header.to_str().ok())
      .and_then(|value| {
        value
          .trim()
          .parse::<u64>()
          .ok()
          .map(Duration::from_secs)
          .or_else(|| {
            httpdate::parse_http_date(value).ok().map(|date| {
              date.duration_since(SystemTime::now()).unwrap_or_default()
            })
          })
      });

    let limit = config.max_response_bytes();
    if response
      .content_length()
      .is_some_and(|size| size > limit as u64)
    {
      return Err(HttpError::ResponseLimit);
    }

    let mut body = Vec::new();
    while let Some(chunk) = response
      .chunk()
      .await
      .map_err(|error| HttpError::Network(Box::new(error.without_url())))?
    {
      if chunk.len() > limit - body.len() {
        return Err(HttpError::ResponseLimit);
      }
      body.extend_from_slice(&chunk);
    }

    Ok(HttpResponse::new(status, retry_after, body))
  }
}
