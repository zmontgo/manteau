//! End-to-end integration tests for the `mjml!` proc-macro.
//!
//! These tests live in the consumer-facing `manteau` crate (not in
//! `manteau-macros`) because that's how a real user encounters the
//! macro — through `manteau::mjml`. The macro's emitted paths
//! (`::manteau::prelude::*`) resolve in this binary the same way they
//! would in any downstream crate.

use manteau::mjml;
use manteau::prelude::*;

fn basic() -> Body { mjml!(<Body></Body>) }

fn one_section() -> Body {
  mjml!(
    <Body>
      <Section></Section>
    </Body>
  )
}

fn text_simple() -> Body {
  mjml!(
    <Body>
      <Section>
        <Column>
          <Text>"Hello, world!"</Text>
        </Column>
      </Section>
    </Body>
  )
}

fn text_with_interp() -> Body {
  let name = "Alice";
  mjml!(
    <Body>
      <Section>
        <Column>
          <Text>"Hello, " {name} "! Welcome."</Text>
        </Column>
      </Section>
    </Body>
  )
}

fn attrs_string_lit() -> Body {
  mjml!(
    <Body background-color="#f00">
      <Section background-color="#ff0000">
        <Column width="50%">
          <Text color="#abc" font-size="20px">"Styled"</Text>
        </Column>
      </Section>
    </Body>
  )
}

fn attrs_expr() -> Body {
  let c = Color::hex(0x123456);
  mjml!(
    <Body background-color={c}>
      <Section>
        <Column width={Percentage::new(75).unwrap()}>
          <Text font-size={Pixels::new(14)}>"Expr-attr"</Text>
        </Column>
      </Section>
    </Body>
  )
}

fn control_flow_if() -> Body {
  let show = true;
  mjml!(
    <Body>
      <Section>
        <Column>
          @if show {
            <Text>"Visible"</Text>
          }
        </Column>
      </Section>
    </Body>
  )
}

fn control_flow_if_else() -> Body {
  let show = false;
  mjml!(
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
  )
}

fn control_flow_for() -> Body {
  let items = vec!["a", "b", "c"];
  mjml!(
    <Body>
      <Section>
        <Column>
          @for item in &items {
            <Text>{item}</Text>
          }
        </Column>
      </Section>
    </Body>
  )
}

fn control_flow_while() -> Body {
  let mut n = 3;
  mjml!(
    <Body>
      <Section>
        <Column>
          @while n > 0 {
            <Text>{ { let s = n.to_string(); n -= 1; s } }</Text>
          }
        </Column>
      </Section>
    </Body>
  )
}

fn control_flow_while_let() -> Body {
  let mut iter = vec!["x", "y", "z"].into_iter();
  mjml!(
    <Body>
      <Section>
        <Column>
          @while let Some(item) = iter.next() {
            <Text>{item}</Text>
          }
        </Column>
      </Section>
    </Body>
  )
}

fn control_flow_match() -> Body {
  let kind = 2;
  mjml!(
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
  )
}

fn control_flow_match_multi_child() -> Body {
  let kind = "premium";
  mjml!(
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
  )
}

fn image_self_closing() -> Body {
  mjml!(
    <Body>
      <Section>
        <Column>
          <Image src="https://example.com/logo.png" />
        </Column>
      </Section>
    </Body>
  )
}

fn button_with_required_attr() -> Body {
  mjml!(
    <Body>
      <Section>
        <Column>
          <Button href="https://example.com">"Click me"</Button>
        </Column>
      </Section>
    </Body>
  )
}

fn interp_block() -> Body {
  let extra: Text = Text::new("extra");
  mjml!(
    <Body>
      <Section>
        <Column>
          <Text>"Before"</Text>
          {extra}
          <Text>"After"</Text>
        </Column>
      </Section>
    </Body>
  )
}

fn text_with_if() -> Body {
  let logged_in = true;
  mjml!(
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
  )
}

fn text_with_for() -> Body {
  let names = vec!["Alice", "Bob", "Carol"];
  mjml!(
    <Body><Section><Column>
      <Text>
        "Members: "
        @for name in &names {
          {name} " "
        }
      </Text>
    </Column></Section></Body>
  )
}

fn text_with_match() -> Body {
  let kind = 2;
  mjml!(
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
  )
}

fn text_with_while() -> Body {
  let mut n = 3;
  mjml!(
    <Body><Section><Column>
      <Text>
        "Countdown:"
        @while n > 0 {
          " " {n}
          {{ n -= 1; "" }}
        }
      </Text>
    </Column></Section></Body>
  )
}

fn wrapper_with_sections() -> Body {
  mjml!(
    <Body>
      <Wrapper background-color="#fff" border-radius="8px">
        <Section>
          <Column>
            <Text>"Inside wrapper"</Text>
          </Column>
        </Section>
      </Wrapper>
    </Body>
  )
}

// ─── Assertions ──────────────────────────────────────────────────────

#[test]
fn smoke_all_examples_compile_and_construct() {
  let _ = basic();
  let _ = one_section();
  let _ = text_simple();
  let _ = text_with_interp();
  let _ = attrs_string_lit();
  let _ = attrs_expr();
  let _ = control_flow_if();
  let _ = control_flow_if_else();
  let _ = control_flow_for();
  let _ = control_flow_while();
  let _ = control_flow_while_let();
  let _ = control_flow_match();
  let _ = control_flow_match_multi_child();
  let _ = image_self_closing();
  let _ = button_with_required_attr();
  let _ = interp_block();
  let _ = wrapper_with_sections();
  let _ = text_with_if();
  let _ = text_with_for();
  let _ = text_with_match();
  let _ = text_with_while();
}

#[test]
fn text_body_if_picks_branch() {
  let b = text_with_if();
  assert_eq!(text_at_path(&b, 0).content, "Hello, friend!");
}

#[test]
fn text_body_for_concatenates_iterations() {
  let b = text_with_for();
  assert_eq!(text_at_path(&b, 0).content, "Members: Alice Bob Carol ");
}

#[test]
fn text_body_match_picks_arm() {
  let b = text_with_match();
  assert_eq!(text_at_path(&b, 0).content, "Status: one or two");
}

#[test]
fn text_body_while_loops() {
  let b = text_with_while();
  assert_eq!(text_at_path(&b, 0).content, "Countdown: 3 2 1");
}

fn text_at_path(b: &Body, depth: usize) -> &Text {
  assert!(depth < b.children.len());
  let s = match &b.children[depth] {
    BodyChild::Section(s) => s,
    _ => panic!("expected Section as body child"),
  };
  match &s.columns[0].children[0] {
    Block::Text(t) => t,
    _ => panic!("expected Text at column[0].children[0]"),
  }
}

#[test]
fn text_simple_carries_literal_content() {
  let b = text_simple();
  assert_eq!(text_at_path(&b, 0).content, "Hello, world!");
}

#[test]
fn text_interp_is_format_concatenated() {
  let b = text_with_interp();
  assert_eq!(text_at_path(&b, 0).content, "Hello, Alice! Welcome.");
}

#[test]
fn string_lit_attrs_dispatch_typed() {
  let b = attrs_string_lit();
  assert_eq!(b.background_color, Some(Color::hex(0xff0000)));
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  assert_eq!(s.background_color, Some(Color::hex(0xff0000)));
  assert_eq!(s.columns[0].width, Some(Percentage::new(50).unwrap()));
  match &s.columns[0].children[0] {
    Block::Text(t) => {
      assert_eq!(t.color, Some(Color::hex(0xaabbcc)));
      assert_eq!(t.font_size, Some(Pixels::new(20)));
      assert_eq!(t.content, "Styled");
    }
    _ => panic!(),
  }
}

#[test]
fn for_loop_produces_one_child_per_iteration() {
  let b = control_flow_for();
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  let children = &s.columns[0].children;
  assert_eq!(children.len(), 3);
  let texts: Vec<&str> = children
    .iter()
    .map(|c| match c {
      Block::Text(t) => t.content.as_str(),
      _ => panic!(),
    })
    .collect();
  assert_eq!(texts, vec!["a", "b", "c"]);
}

#[test]
fn if_else_picks_correct_branch() {
  let b = control_flow_if_else();
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  assert_eq!(s.columns[0].children.len(), 1);
  match &s.columns[0].children[0] {
    Block::Text(t) => assert_eq!(t.content, "No"),
    _ => panic!(),
  }
}

#[test]
fn button_has_required_href() {
  let b = button_with_required_attr();
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  match &s.columns[0].children[0] {
    Block::Button(btn) => {
      assert_eq!(btn.content, "Click me");
      assert_eq!(btn.href.as_str(), "https://example.com");
    }
    _ => panic!("expected button"),
  }
}

#[test]
fn image_has_required_src() {
  let b = image_self_closing();
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  match &s.columns[0].children[0] {
    Block::Image(img) => {
      assert_eq!(img.src.as_str(), "https://example.com/logo.png");
    }
    _ => panic!("expected image"),
  }
}

#[test]
fn while_loop_runs_until_condition_fails() {
  let b = control_flow_while();
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  let texts: Vec<&str> = s.columns[0]
    .children
    .iter()
    .map(|c| match c {
      Block::Text(t) => t.content.as_str(),
      _ => panic!(),
    })
    .collect();
  assert_eq!(texts, vec!["3", "2", "1"]);
}

#[test]
fn while_let_loop_drains_iterator() {
  let b = control_flow_while_let();
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  let texts: Vec<&str> = s.columns[0]
    .children
    .iter()
    .map(|c| match c {
      Block::Text(t) => t.content.as_str(),
      _ => panic!(),
    })
    .collect();
  assert_eq!(texts, vec!["x", "y", "z"]);
}

#[test]
fn match_arm_with_or_pattern_fires() {
  let b = control_flow_match();
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  assert_eq!(s.columns[0].children.len(), 1);
  match &s.columns[0].children[0] {
    Block::Text(t) => assert_eq!(t.content, "one or two"),
    _ => panic!(),
  }
}

#[test]
fn match_arm_with_multiple_children() {
  let b = control_flow_match_multi_child();
  let s = match &b.children[0] {
    BodyChild::Section(s) => s,
    _ => panic!(),
  };
  let texts: Vec<&str> = s.columns[0]
    .children
    .iter()
    .map(|c| match c {
      Block::Text(t) => t.content.as_str(),
      _ => panic!(),
    })
    .collect();
  assert_eq!(texts, vec!["Premium tier", "Thanks for subscribing"]);
}

#[test]
fn wrapper_appears_as_body_child() {
  let b = wrapper_with_sections();
  match &b.children[0] {
    BodyChild::Wrapper(w) => {
      assert_eq!(w.background_color, Some(Color::hex(0xffffff)));
      assert_eq!(w.sections.len(), 1);
    }
    _ => panic!("expected Wrapper"),
  }
}
