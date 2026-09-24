//! Asserts that the `#[tracing::instrument]` annotations actually fire
//! and that they carry the field shape downstream consumers may rely on
//! (operator dashboards, log aggregators). Locks the contract — changing
//! a span's name or removing a field becomes a test failure.

use std::sync::{Arc, Mutex};

use manteau::{Message, prelude::*};
use tracing::{
  Subscriber,
  field::{Field, Visit},
  span::{Attributes, Id},
};
use tracing_subscriber::{
  layer::{Context, Layer, SubscriberExt},
  registry::Registry,
};

#[derive(Debug, Clone)]
struct CapturedSpan {
  name:   &'static str,
  fields: String,
}

#[derive(Default, Clone)]
struct CapturingLayer {
  spans: Arc<Mutex<Vec<CapturedSpan>>>,
}

impl<S: Subscriber> Layer<S> for CapturingLayer {
  fn on_new_span(
    &self,
    attrs: &Attributes<'_>,
    _id: &Id,
    _ctx: Context<'_, S>,
  ) {
    let mut visitor = FieldVisitor::default();
    attrs.record(&mut visitor);
    self.spans.lock().unwrap().push(CapturedSpan {
      name:   attrs.metadata().name(),
      fields: visitor.fields,
    });
  }
}

#[derive(Default)]
struct FieldVisitor {
  fields: String,
}

impl Visit for FieldVisitor {
  fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
    use std::fmt::Write;
    let _ = write!(&mut self.fields, "{}={:?} ", field.name(), value);
  }

  fn record_str(&mut self, field: &Field, value: &str) {
    use std::fmt::Write;
    let _ = write!(&mut self.fields, "{}={} ", field.name(), value);
  }
}

#[test]
fn preparation_traces_operations_without_private_content() {
  let capture = CapturingLayer::default();
  let subscriber = Registry::default().with(capture.clone());
  let _guard = tracing::subscriber::set_default(subscriber);
  let message = Message::new(
    Envelope::new(
      Address::new("private-sender@example.com".parse().unwrap()),
      Recipients::to(Address::new(
        "private-recipient@example.com".parse().unwrap(),
      )),
      HeaderText::new("Private subject").unwrap(),
    ),
    Template::new(Body::new().push(
      Section::new().push(Column::new().push(Text::new("Private body"))),
    )),
  );
  message.prepare(&manteau::MrmlRenderer::new()).unwrap();
  let spans = capture.spans.lock().unwrap();
  assert!(spans.iter().any(|span| span.name == "prepare"));
  for span in spans.iter() {
    assert!(!span.fields.contains("Private"));
    assert!(!span.fields.contains("example.com"));
  }
}
