#[cfg(feature = "visually-hidden")]
mod announcer;
#[cfg(feature = "focus-trap")]
mod focus_trap;
#[cfg(any(
    feature = "focus-trap",
    feature = "hover-card",
    feature = "color",
    feature = "date-field",
    feature = "number-field"
))]
mod tab_stops;
#[cfg(feature = "visually-hidden")]
mod visually_hidden;

#[cfg(feature = "visually-hidden")]
pub(crate) use announcer::{Announcer, use_announcer};
#[cfg(feature = "focus-trap")]
pub use focus_trap::{FocusTrap, FocusTrapInitialFocus, FocusTrapProps};
#[cfg(any(feature = "focus-trap", feature = "hover-card"))]
pub(crate) use tab_stops::focus_edge;
#[cfg(any(
    feature = "focus-trap",
    feature = "hover-card",
    feature = "color",
    feature = "date-field",
    feature = "number-field"
))]
pub(crate) use tab_stops::{focus_first_stop, tab_order};
#[cfg(feature = "visually-hidden")]
pub(crate) use visually_hidden::{
    VISUALLY_HIDDEN_FIXED_SX, VISUALLY_HIDDEN_SX, hidden_input_centred_sx,
    visually_hidden_absolute_sx,
};
#[cfg(feature = "visually-hidden")]
pub use visually_hidden::{VisuallyHidden, VisuallyHiddenProps, visually_hidden_sx};
