//! Physical HTTP capability.

use async_trait::async_trait;

use super::{HttpError, HttpRequest, HttpResponse};

/// Physical HTTP capability required by the sender. Implementations must
/// neither retry nor redirect requests, and must honor the supplied total
/// timeout and response limit. These obligations also apply to caller-supplied
/// implementations.
#[async_trait]
pub trait Http: Send + Sync {
  /// Perform exactly one request using the admitted configuration and payload.
  async fn post(
    &self,
    request: HttpRequest<'_>,
  ) -> Result<HttpResponse, HttpError>;
}
