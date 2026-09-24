use manteau_core::templating::{Body, Column, Push, Section, Template, Text};
use manteau_render::MrmlRenderer;

#[test]
fn typed_content_is_escaped_before_mjml_rendering() {
  let template = Template::new(
    Body::new().push(
      Section::new()
        .push(Column::new().push(Text::new("<script>alert(1)</script>"))),
    ),
  );
  let rendered = template.render(&MrmlRenderer::new()).unwrap();

  assert!(!rendered.html().contains("<script>alert(1)</script>"));
  assert!(rendered.html().contains("&lt;script&gt;"));
  assert!(rendered.text().contains("<script>"));
}

#[test]
fn plaintext_width_is_checked() {
  assert!(MrmlRenderer::new().text_width(2).is_err());
  assert!(MrmlRenderer::new().text_width(3).is_ok());
}
