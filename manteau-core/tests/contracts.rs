use manteau_core::prelude::*;
use manteau_core::{RenderedBodyError, templating::attributes::{measurements::{Em, Rem, PositiveFinite}, urls::ImageUrl}};

#[test]
fn persistence_cannot_bypass_envelope_and_body_validation() {
  let message = PreparedMessage::new(
    Envelope::new(
      Address::new("from@example.com".parse().unwrap()),
      Recipients::bcc(Address::new("hidden@example.com".parse().unwrap())),
      HeaderText::new("Private subject").unwrap(),
    ),
    Rendered::new("<p>Private body</p>", "Private body").unwrap(),
  );
  let encoded = serde_json::to_value(&message).unwrap();
  let restored: PreparedMessage =
    serde_json::from_value(encoded.clone()).unwrap();
  assert_eq!(serde_json::to_value(restored).unwrap(), encoded);
  for pointer in ["/envelope/from/email", "/envelope/subject"] {
    let mut corrupt = encoded.clone();
    *corrupt.pointer_mut(pointer).unwrap() =
      serde_json::json!("injected\r\nHeader: value");
    assert!(serde_json::from_value::<PreparedMessage>(corrupt).is_err());
  }
  let mut empty = encoded.clone();
  empty["envelope"]["recipients"]["bcc"] = serde_json::json!([]);
  assert!(serde_json::from_value::<PreparedMessage>(empty).is_err());
  let mut empty = encoded;
  empty["body"] = serde_json::json!({"html":"", "text":""});
  assert!(serde_json::from_value::<PreparedMessage>(empty).is_err());
  for diagnostic in [
    format!("{message:?}"),
    format!("{:?}", message.envelope()),
    format!("{:?}", message.body()),
  ] {
    assert!(!diagnostic.contains("Private"));
    assert!(!diagnostic.contains("example.com"));
  }
}

#[test]
fn prepared_clones_share_the_exact_rendered_bodies() {
  let message = PreparedMessage::new(
    Envelope::new(
      Address::new("from@example.com".parse().unwrap()),
      Recipients::to(Address::new("to@example.com".parse().unwrap())),
      HeaderText::new("").unwrap(),
    ),
    Rendered::new("", "Text-only email").unwrap(),
  );
  let cloned = message.clone();
  assert!(std::ptr::eq(message.body(), cloned.body()));
}

#[test]
fn imported_bodies_preserve_the_renderer_character_contract() {
  assert!(matches!(Rendered::new("", ""), Err(RenderedBodyError::Empty)));
  assert!(matches!(Rendered::new("<p>hello</p>\0", "hello"), Err(RenderedBodyError::Html(_))));
  assert!(matches!(Rendered::new("<p>hello</p>", "hello\0"), Err(RenderedBodyError::Plaintext(_))));
  assert!(serde_json::from_str::<Rendered>(r#"{"html":"<p>hello</p>","text":"hello\u0000"}"#).is_err());
}

#[test]
fn markup_values_reject_unsupported_links_and_nonfinite_dimensions() {
  assert!(Url::try_parse("javascript:alert(1)").is_err());
  assert!(Url::try_parse("data:text/html,<script>alert(1)</script>").is_err());
  assert!(Url::try_parse("https://user:secret@example.com").is_err());
  assert!(Url::try_parse("mailto:person@example.com").is_ok());
  assert!(ImageUrl::try_parse("mailto:person@example.com").is_err());
  assert!(ImageUrl::try_parse("https://example.com/image.png").is_ok());
  assert!(Color::try_parse("imaginarycolor").is_err());
  assert!(Color::try_parse("rebeccapurple").is_ok());
  assert!(Em::new(f32::NAN).is_err());
  assert!(Rem::new(f32::INFINITY).is_err());
  assert!(PositiveFinite::new(0.0).is_err());
  let private = "https://example.com/reset?token=private-token";
  assert!(!format!("{:?}", Url::try_parse(private).unwrap()).contains("private-token"));
  assert!(!format!("{:?}", Url::try_parse("javascript:private-token").unwrap_err()).contains("private-token"));
  assert!(manteau_core::MessageId::new("").is_err());
  assert!(manteau_core::MessageId::new("line\rbreak").is_err());
}
