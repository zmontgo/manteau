use std::sync::{
  Arc,
  atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use manteau::{
  Acceptance, Address, Envelope, HeaderText, HtmlBody, Mailer, MailerError,
  Message, MessageId, MjmlDocument, PlaintextBody, PreparedMessage, Receipt,
  Recipients, Renderer, Transport, TransportFailure,
  templating::{Body, Column, Push, Section, Template, Text},
};
use manteau_mock::MockTransport;

struct CountingRenderer(Arc<AtomicUsize>);

impl Renderer for CountingRenderer {
  type Error = std::convert::Infallible;

  fn html(&self, _: &MjmlDocument) -> Result<HtmlBody, Self::Error> {
    self.0.fetch_add(1, Ordering::SeqCst);
    Ok(HtmlBody::new("<p>Hello</p>").unwrap())
  }

  fn plaintext(&self, _: &HtmlBody) -> Result<PlaintextBody, Self::Error> {
    Ok(PlaintextBody::new("Hello").unwrap())
  }
}

struct RejectedTransport;

#[derive(Debug)]
struct RejectedReceipt;

impl Receipt for RejectedReceipt {
  fn ids(&self) -> &[MessageId] {
    &[]
  }
}

#[derive(Debug, thiserror::Error)]
#[error("submission rejected")]
struct Rejected;

impl TransportFailure for Rejected {
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

#[async_trait]
impl Transport for RejectedTransport {
  type Error = Rejected;
  type Receipt = RejectedReceipt;

  async fn send(
    &self,
    _: &PreparedMessage,
  ) -> Result<Self::Receipt, Self::Error> {
    Err(Rejected)
  }
}

struct Welcome(Message);

impl Welcome {
  fn new() -> Self {
    Self(Message::new(
      Envelope::new(
        Address::new("from@example.com".parse().unwrap()),
        Recipients::to(Address::new("to@example.com".parse().unwrap())),
        HeaderText::new("Welcome").unwrap(),
      ),
      Template::new(
        Body::new()
          .push(Section::new().push(Column::new().push(Text::new("Hello")))),
      ),
    ))
  }

  fn into_message(self) -> Message {
    self.0
  }
}

#[tokio::test]
async fn configured_mailer_prepares_and_captures_once() {
  let renders = Arc::new(AtomicUsize::new(0));
  let mailer =
    Mailer::new(CountingRenderer(Arc::clone(&renders)), MockTransport::new());
  mailer.send(Welcome::new().into_message()).await.unwrap();

  let sent = mailer.transport().sent();
  assert_eq!(sent.len(), 1);
  assert_eq!(sent[0].body().html(), "<p>Hello</p>");
  assert_eq!(renders.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn failed_submission_retains_exact_prepared_content() {
  let mailer = Mailer::new(
    CountingRenderer(Arc::new(AtomicUsize::new(0))),
    RejectedTransport,
  );
  let error = mailer
    .send(Welcome::new().into_message())
    .await
    .unwrap_err();

  match error {
    MailerError::Submission { prepared, error } => {
      assert_eq!(prepared.body().html(), "<p>Hello</p>");
      assert_eq!(error.acceptance(), Acceptance::NotAccepted);
    }
    MailerError::Preparation { .. } => {
      panic!("preparation unexpectedly failed")
    }
  }
}
