//! Asserts that the `#[tracing::instrument]` annotations actually fire
//! and that they carry the field shape downstream consumers may rely on
//! (operator dashboards, log aggregators). Locks the contract — changing
//! a span's name or removing a field becomes a test failure.

use std::sync::{Arc, Mutex};

use manteau::{
  Address, Message,
  templating::{Body, Column, Section, Template, Text},
};
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

fn make_message(subject: &str) -> Message {
  let template = Template::new(
    Body::new().push_section(
      Section::new().push_column(Column::new().push(Text::new("Hi"))),
    ),
  );

  Message::new(
    Address::new("from@example.com".parse().unwrap()),
    vec![Address::new("to@example.com".parse().unwrap())],
    subject,
    template,
  )
}

#[test]
fn message_render_emits_span_with_subject_field() {
  let capture = CapturingLayer::default();
  let subscriber = Registry::default().with(capture.clone());
  let _guard = tracing::subscriber::set_default(subscriber);

  make_message("Quarterly Update").render().unwrap();

  let spans = capture.spans.lock().unwrap();
  let render = spans
    .iter()
    .find(|s| s.name == "render")
    .expect("Message::render should emit a 'render' span");
  assert!(
    render.fields.contains("Quarterly Update"),
    "expected subject field present, got fields: {}",
    render.fields,
  );
}

#[test]
fn render_pipeline_emits_html_and_plaintext_spans() {
  let capture = CapturingLayer::default();
  let subscriber = Registry::default().with(capture.clone());
  let _guard = tracing::subscriber::set_default(subscriber);

  make_message("anything").render().unwrap();

  let spans = capture.spans.lock().unwrap();
  let names: Vec<&str> = spans.iter().map(|s| s.name).collect();
  assert!(
    names.contains(&"render_html"),
    "expected render_html span, got {:?}",
    names,
  );
  assert!(
    names.contains(&"render_plaintext"),
    "expected render_plaintext span, got {:?}",
    names,
  );
}
