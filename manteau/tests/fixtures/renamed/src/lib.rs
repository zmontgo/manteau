//! A consumer that imports Manteau under a different dependency name.

use mailcoat::prelude::*;

pub struct Example(Body);

impl Example {
  pub fn new() -> Self {
    Self(mailcoat::mjml! {
      <Body>
        <Section>
          <Column>
            <Text color="#fff">"Hello"</Text>
            <Image src="https://example.com/logo.png" />
          </Column>
        </Section>
      </Body>
    })
  }

  pub fn body(&self) -> &Body {
    &self.0
  }
}
