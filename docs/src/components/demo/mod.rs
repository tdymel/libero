mod color;
mod control;
mod demo;

pub use control::{Control, ControlKind, generate_code};
#[cfg(test)]
pub use demo::DemoCode;
pub use demo::{Child, Demo, DemoValues, UNSET, Wrap, indent, or_unset};
