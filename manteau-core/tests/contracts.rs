use manteau_core::prelude::*;

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
