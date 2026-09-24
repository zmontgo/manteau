//! Frequently used template and email construction types.
pub use crate::{
  Address, EmailAddress, Envelope, HeaderText, Message, PreparedMessage,
  Receipt, Recipients, Rendered, Transport, TransportFailure,
  templating::{
    Block, Body, BodyChild, Button, Column, Element, Image, Push, Section,
    Template, Text, Wrapper, attributes::prelude::*,
  },
};
