mod color;
mod control;
mod demo;
mod field;
mod gradient;
mod placement;

pub use control::{Control, ControlKind, generate_code};
#[cfg(test)]
pub use demo::DemoCode;
pub use demo::{Child, Demo, DemoFile, DemoValues, UNSET, Wrap, indent, or_unset};
pub use field::{FieldCopy, field_controls, field_props};
pub use gradient::{gradient_controls, gradient_value, not_gradient_variant};
pub use placement::{align_of, delay_of, side_of};
