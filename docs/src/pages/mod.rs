mod a11y;
mod about;
mod data_display;
mod feedback;
mod form;
mod inputs;
mod layout;
mod navigation;
mod overlay;
mod surface;
mod typography;

// One glob per category, so a new page touches only its own category `mod.rs`.
pub use a11y::*;
pub use about::*;
pub use data_display::*;
pub use feedback::*;
pub use form::*;
pub use inputs::*;
pub use layout::*;
pub use navigation::*;
pub use overlay::*;
pub use surface::*;
pub use typography::*;
