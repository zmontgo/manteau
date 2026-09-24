use manteau_core::{
  Renderer,
  templating::{Body, Column, Push, Section, Template, Text},
};
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
fn width_is_checked_and_remote_includes_are_disabled() {
  assert!(MrmlRenderer::new().text_width(2).is_err());
  assert!(MrmlRenderer::new().text_width(3).is_ok());

  let renderer = MrmlRenderer::new();
  let error = renderer
    .html(
      "<mjml><mj-head><mj-include path=\"https://example.com/template\" \
       /></mj-head><mj-body /></mjml>",
    )
    .unwrap_err();
  assert!(error.to_string().contains("parse"));
}
