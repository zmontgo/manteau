use std::borrow::Cow;

// Cloudflare requires encoded words for non-ASCII subject and sender headers.
pub(crate) struct Header<'a>(pub(crate) &'a str);
impl<'a> Header<'a> {
  pub(crate) fn encode(self) -> Cow<'a, str> {
    let value = self.0;
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
      // multibyte sequence — each word must decode to valid UTF-8 (RFC 2047
      // §5).
      let mut piece = String::new();
      match ch {
        ' ' => piece.push('_'),
        // Printable ASCII may appear literally in Q-text, except the three
        // characters that always carry meaning there: '=', '?', '_'.
        c if (0x20..=0x7e).contains(&(c as u32))
          && !matches!(c, '=' | '?' | '_') =>
        {
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
}

#[cfg(test)]
mod tests {
  use super::Header;

  #[test]
  fn unicode_is_encoded_without_splitting_a_character() {
    let value = "🦊".repeat(40);
    let encoded = Header(&value).encode();

    for word in encoded.split(' ') {
      assert!(word.starts_with("=?UTF-8?Q?"));
      assert!(word.ends_with("?="));
      assert!(word.len() <= 75);
      assert!(word.contains("=F0=9F=A6=8A"));
    }
  }

  #[test]
  fn punctuation_and_spaces_follow_q_encoding() {
    assert_eq!(Header("é ?_=").encode(), "=?UTF-8?Q?=C3=A9_=3F=5F=3D?=");
  }
}
