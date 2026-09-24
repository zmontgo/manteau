//! Opt-in local measurement of rendering versus prepared-message reuse.
//! Run with `cargo test -p manteau --test preparation_cost -- --ignored
//! --nocapture`.

use std::{hint::black_box, time::Instant};

use manteau::{
  Address, Envelope, HeaderText, Message, MrmlRenderer, Recipients,
  templating::{Body, Column, Push, Section, Template, Text},
};

#[test]
#[ignore = "local measurement; results depend on the host"]
fn preparation_and_reuse_cost() {
  let paragraph = "A representative transactional update. ".repeat(32);
  let template = Template::new(
    Body::new().push(
      Section::new().push(
        Column::new()
          .push(Text::new("Account update"))
          .push(Text::new(paragraph)),
      ),
    ),
  );
  let message = Message::new(
    Envelope::new(
      Address::new("sender@example.com".parse().unwrap()),
      Recipients::to(Address::new("reader@example.com".parse().unwrap())),
      HeaderText::new("Account update").unwrap(),
    ),
    template,
  );
  let renderer = MrmlRenderer::new();
  let iterations = 500;

  let render_start = Instant::now();
  for _ in 0..iterations {
    black_box(message.prepare(&renderer).unwrap());
  }
  let render_elapsed = render_start.elapsed();

  let prepared = message.prepare(&renderer).unwrap();
  let clone_start = Instant::now();
  for _ in 0..iterations {
    black_box(prepared.clone());
  }
  let clone_elapsed = clone_start.elapsed();

  eprintln!(
    "{iterations} preparations: {render_elapsed:?}; {iterations} prepared \
     clones: {clone_elapsed:?}"
  );
}
