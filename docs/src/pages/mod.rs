mod about;
mod accessibility;
mod buttons;
mod data_display;
mod feedback;
mod form;
mod home;
mod hooks;
mod layout;
mod navigation;
mod not_found;
mod overlay;
mod typography;

// One glob per category, so a new page touches only its own category `mod.rs`.
pub use about::*;
pub use accessibility::*;
pub use buttons::*;
pub use data_display::*;
pub use feedback::*;
pub use form::*;
pub use home::Home;
#[cfg(test)]
pub use home::TITLE as HOME_TITLE;
pub use hooks::*;
pub use layout::*;
pub use navigation::*;
pub use not_found::NotFound;
pub use overlay::*;
pub use typography::*;
