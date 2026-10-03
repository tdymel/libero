mod color;
mod control;
mod demo;
mod gradient;
mod placement;

pub use control::{Control, ControlKind, generate_code};
#[cfg(test)]
pub use demo::DemoCode;
pub use demo::{Child, Demo, DemoValues, UNSET, Wrap, indent, or_unset};
pub use gradient::{gradient_controls, gradient_value, not_gradient_variant};
pub use placement::{align_of, delay_of, side_of};
