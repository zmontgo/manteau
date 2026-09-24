//! Frequently used template and email construction types.
pub use crate::{
  Address, EmailAddress, Envelope, HeaderText, PreparedMessage, Receipt,
  Recipients, Rendered, Renderer, Transport, TransportFailure,
  templating::{
    Block, Body, BodyChild, Button, Column, Element, Image, Push, Section,
    Template, Text, Wrapper, attributes::prelude::*,
  },
};
