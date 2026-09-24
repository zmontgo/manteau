//! End-to-end integration tests for the `mjml!` proc-macro.
//!
//! These tests live in the consumer-facing `manteau` crate (not in
//! `manteau-macros`) because that's how a real user encounters the
//! macro — through `manteau::mjml`. A separate fixture checks renamed imports.

use manteau::{mjml, prelude::*};

struct MacroFixture(Body);

impl MacroFixture {
  fn body(&self) -> &Body {
    &self.0
  }

  fn basic() -> Self {
    Self(mjml!(<Body></Body>))
  }

  fn one_section() -> Self {
    Self(mjml!(
      <Body>
        <Section></Section>
      </Body>
    ))
  }

  fn text_simple() -> Self {
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            <Text>"Hello, world!"</Text>
          </Column>
        </Section>
      </Body>
    ))
  }

  fn text_with_interp() -> Self {
    let name = "Alice";
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            <Text>"Hello, " {name} "! Welcome."</Text>
          </Column>
        </Section>
      </Body>
    ))
  }

  fn attrs_string_lit() -> Self {
    Self(mjml!(
      <Body background-color="#f00">
        <Section background-color="#ff0000">
          <Column width="50%">
            <Text color="#abc" font-size="20px">"Styled"</Text>
          </Column>
        </Section>
      </Body>
    ))
  }

  fn checked_literals() -> Self {
    Self(mjml!(
      <Body background-color="rebeccapurple">
        <Section>
          <Column>
            <Text color="#abcd" align="justify" font-weight="700" line-height="1.5" text-transform="uppercase">"Styled"</Text>
            <Button href="mailto:hello@example.com" align="center">"Write"</Button>
          </Column>
        </Section>
      </Body>
    ))
  }

  fn attrs_expr() -> Self {
    let c = Color::hex(0x123456).unwrap();
    Self(mjml!(
      <Body background-color={c}>
        <Section>
          <Column width={Percentage::new(75).unwrap()}>
            <Text font-size={Pixels::new(14)}>"Expr-attr"</Text>
          </Column>
        </Section>
      </Body>
    ))
  }

  fn control_flow_if() -> Self {
    let show = true;
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            @if show {
              <Text>"Visible"</Text>
            }
          </Column>
        </Section>
      </Body>
    ))
  }

  fn control_flow_if_else() -> Self {
    let show = false;
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            @if show {
              <Text>"Yes"</Text>
            } @else {
              <Text>"No"</Text>
            }
          </Column>
        </Section>
      </Body>
    ))
  }

  fn control_flow_for() -> Self {
    let items = vec!["a", "b", "c"];
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            @for item in &items {
              <Text>{item}</Text>
            }
          </Column>
        </Section>
      </Body>
    ))
  }

  fn control_flow_while() -> Self {
    let mut n = 3;
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            @while n > 0 {
              <Text>{ { let s = n.to_string(); n -= 1; s } }</Text>
            }
          </Column>
        </Section>
      </Body>
    ))
  }

  fn control_flow_while_let() -> Self {
    let mut iter = vec!["x", "y", "z"].into_iter();
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            @while let Some(item) = iter.next() {
              <Text>{item}</Text>
            }
          </Column>
        </Section>
      </Body>
    ))
  }

  fn control_flow_match() -> Self {
    let kind = 2;
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            @match kind {
              0 => {
                <Text>"zero"</Text>
              }
              1 | 2 => {
                <Text>"one or two"</Text>
              }
              n if n > 10 => {
                <Text>"big"</Text>
              }
              _ => {
                <Text>"other"</Text>
              }
            }
          </Column>
        </Section>
      </Body>
    ))
  }

  fn control_flow_match_multi_child() -> Self {
    let kind = "premium";
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            @match kind {
              "premium" => {
                <Text>"Premium tier"</Text>
                <Text>"Thanks for subscribing"</Text>
              }
              _ => {
                <Text>"Free tier"</Text>
              }
            }
          </Column>
        </Section>
      </Body>
    ))
  }

  fn image_self_closing() -> Self {
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            <Image src="https://example.com/logo.png" />
          </Column>
        </Section>
      </Body>
    ))
  }

  fn button_with_required_attr() -> Self {
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            <Button href="https://example.com">"Click me"</Button>
          </Column>
        </Section>
      </Body>
    ))
  }

  fn interp_block() -> Self {
    let extra: Text = Text::new("extra");
    Self(mjml!(
      <Body>
        <Section>
          <Column>
            <Text>"Before"</Text>
            {extra}
            <Text>"After"</Text>
          </Column>
        </Section>
      </Body>
    ))
  }

  fn text_with_if() -> Self {
    let logged_in = true;
    Self(mjml!(
      <Body><Section><Column>
        <Text>
          "Hello, "
          @if logged_in {
            "friend"
          } @else {
            "stranger"
          }
          "!"
        </Text>
      </Column></Section></Body>
    ))
  }

  fn text_with_for() -> Self {
    let names = vec!["Alice", "Bob", "Carol"];
    Self(mjml!(
      <Body><Section><Column>
        <Text>
          "Members: "
          @for name in &names {
            {name} " "
          }
        </Text>
      </Column></Section></Body>
    ))
  }

  fn text_with_match() -> Self {
    let kind = 2;
    Self(mjml!(
      <Body><Section><Column>
        <Text>
          "Status: "
          @match kind {
            0 => { "zero" }
            1 | 2 => { "one or two" }
            n if n > 10 => { "big" }
            _ => { "other" }
          }
        </Text>
      </Column></Section></Body>
    ))
  }

  fn text_with_while() -> Self {
    let mut n = 3;
    Self(mjml!(
      <Body><Section><Column>
        <Text>
          "Countdown:"
          @while n > 0 {
            " " {n}
            {{ n -= 1; "" }}
          }
        </Text>
      </Column></Section></Body>
    ))
  }

  fn wrapper_with_sections() -> Self {
    Self(mjml!(
      <Body>
        <Wrapper background-color="#fff" border-radius="8px">
          <Section>
            <Column>
              <Text>"Inside wrapper"</Text>
            </Column>
          </Section>
        </Wrapper>
      </Body>
    ))
  }
}

// ─── Assertions ──────────────────────────────────────────────────────

#[test]
fn smoke_all_examples_compile_and_construct() {
  let _ = MacroFixture::basic();
  let _ = MacroFixture::one_section();
  let _ = MacroFixture::text_simple();
  let _ = MacroFixture::text_with_interp();
  let _ = MacroFixture::attrs_string_lit();
  let _ = MacroFixture::checked_literals();
  let _ = MacroFixture::attrs_expr();
  let _ = MacroFixture::control_flow_if();
  let _ = MacroFixture::control_flow_if_else();
  let _ = MacroFixture::control_flow_for();
  let _ = MacroFixture::control_flow_while();
  let _ = MacroFixture::control_flow_while_let();
  let _ = MacroFixture::control_flow_match();
  let _ = MacroFixture::control_flow_match_multi_child();
  let _ = MacroFixture::image_self_closing();
  let _ = MacroFixture::button_with_required_attr();
  let _ = MacroFixture::interp_block();
  let _ = MacroFixture::wrapper_with_sections();
  let _ = MacroFixture::text_with_if();
  let _ = MacroFixture::text_with_for();
  let _ = MacroFixture::text_with_match();
  let _ = MacroFixture::text_with_while();
}

#[test]
fn text_body_if_picks_branch() {
  let b = MacroFixture::text_with_if();
  assert_eq!(b.text_at_path(0).content(), "Hello, friend!");
}

#[test]
fn text_body_for_concatenates_iterations() {
  let b = MacroFixture::text_with_for();
  assert_eq!(b.text_at_path(0).content(), "Members: Alice Bob Carol ");
}

#[test]
fn text_body_match_picks_arm() {
  let b = MacroFixture::text_with_match();
  assert_eq!(b.text_at_path(0).content(), "Status: one or two");
}

#[test]
fn text_body_while_loops() {
  let b = MacroFixture::text_with_while();
  assert_eq!(b.text_at_path(0).content(), "Countdown: 3 2 1");
}

impl MacroFixture {
  fn text_at_path(&self, depth: usize) -> &Text {
    let b = self.body();
    assert!(depth < b.children().len());
    let s = match &b.children()[depth] {
      BodyChild::Section(s) => s,
      _ => panic!("expected Section as body child"),
    };
    match &s.column_items()[0].blocks()[0] {
      Block::Text(t) => t,
      _ => panic!("expected Text at column[0].children[0]"),
    }
  }
}

#[test]
fn text_simple_carries_literal_content() {
  let b = MacroFixture::text_simple();
  assert_eq!(b.text_at_path(0).content(), "Hello, world!");
}

#[test]
fn text_interp_is_format_concatenated() {
  let b = MacroFixture::text_with_interp();
  assert_eq!(b.text_at_path(0).content(), "Hello, Alice! Welcome.");
}

#[test]
fn string_lit_attrs_dispatch_typed() {
  let b = MacroFixture::attrs_string_lit();
  assert_eq!(
    b.body().configured_background_color(),
    Some(&Color::try_parse("#f00").unwrap())
  );
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  assert_eq!(
    s.configured_background_color(),
    Some(&Color::hex(0xff0000).unwrap())
  );
  assert_eq!(
    s.column_items()[0].configured_width(),
    Some(Percentage::new(50).unwrap())
  );
  match &s.column_items()[0].blocks()[0] {
    Block::Text(t) => {
      assert_eq!(
        t.configured_color(),
        Some(&Color::try_parse("#abc").unwrap())
      );
      assert_eq!(t.configured_font_size(), Some(Pixels::new(20)));
      assert_eq!(t.content(), "Styled");
    }
    _ => panic!(),
  }
}

#[test]
fn for_loop_produces_one_child_per_iteration() {
  let b = MacroFixture::control_flow_for();
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  let children = &s.column_items()[0].blocks();
  assert_eq!(children.len(), 3);
  let texts: Vec<&str> = children
    .iter()
    .map(|c| match c {
      Block::Text(t) => t.content(),
      _ => panic!(),
    })
    .collect();
  assert_eq!(texts, vec!["a", "b", "c"]);
}

#[test]
fn if_else_picks_correct_branch() {
  let b = MacroFixture::control_flow_if_else();
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  assert_eq!(s.column_items()[0].blocks().len(), 1);
  match &s.column_items()[0].blocks()[0] {
    Block::Text(t) => assert_eq!(t.content(), "No"),
    _ => panic!(),
  }
}

#[test]
fn button_has_required_href() {
  let b = MacroFixture::button_with_required_attr();
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  match &s.column_items()[0].blocks()[0] {
    Block::Button(btn) => {
      assert_eq!(btn.content(), "Click me");
      assert_eq!(btn.href().as_str(), "https://example.com/");
    }
    _ => panic!("expected button"),
  }
}

#[test]
fn image_has_required_src() {
  let b = MacroFixture::image_self_closing();
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  match &s.column_items()[0].blocks()[0] {
    Block::Image(img) => {
      assert_eq!(img.src().as_str(), "https://example.com/logo.png");
    }
    _ => panic!("expected image"),
  }
}

#[test]
fn while_loop_runs_until_condition_fails() {
  let b = MacroFixture::control_flow_while();
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  let texts: Vec<&str> = s.column_items()[0]
    .blocks()
    .iter()
    .map(|c| match c {
      Block::Text(t) => t.content(),
      _ => panic!(),
    })
    .collect();
  assert_eq!(texts, vec!["3", "2", "1"]);
}

#[test]
fn while_let_loop_drains_iterator() {
  let b = MacroFixture::control_flow_while_let();
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  let texts: Vec<&str> = s.column_items()[0]
    .blocks()
    .iter()
    .map(|c| match c {
      Block::Text(t) => t.content(),
      _ => panic!(),
    })
    .collect();
  assert_eq!(texts, vec!["x", "y", "z"]);
}

#[test]
fn match_arm_with_or_pattern_fires() {
  let b = MacroFixture::control_flow_match();
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  assert_eq!(s.column_items()[0].blocks().len(), 1);
  match &s.column_items()[0].blocks()[0] {
    Block::Text(t) => assert_eq!(t.content(), "one or two"),
    _ => panic!(),
  }
}

#[test]
fn match_arm_with_multiple_children() {
  let b = MacroFixture::control_flow_match_multi_child();
  let s = match &b.body().children()[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  let texts: Vec<&str> = s.column_items()[0]
    .blocks()
    .iter()
    .map(|c| match c {
      Block::Text(t) => t.content(),
      _ => panic!(),
    })
    .collect();
  assert_eq!(texts, vec!["Premium tier", "Thanks for subscribing"]);
}

#[test]
fn wrapper_appears_as_body_child() {
  let b = MacroFixture::wrapper_with_sections();
  match &b.body().children()[0] {
    BodyChild::Wrapper(w) => {
      assert_eq!(
        w.configured_background_color(),
        Some(&Color::try_parse("#fff").unwrap())
      );
      assert_eq!(w.section_items().len(), 1);
    }
    _ => panic!("expected Wrapper"),
  }
}
