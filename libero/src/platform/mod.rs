//! Everything that reaches the machine underneath dioxus: the DOM, the
//! clipboard, a regex engine. One trait per capability, one implementation per
//! platform behind it, and one accessor to reach it.
//!
//! The layer sits below `hooks` and `components`, so both can use it - and
//! nothing here may reach back up.
//!
//! **Elements are the exception to "call an accessor"**: there is no portable
//! way to name an element, so the way in is
//! [`use_element`](crate::hooks::use_element), whose handle picks the richest
//! backend the renderer offers. See [`backend`].

mod backend;
mod click;
mod clipboard;
mod clock;
mod color_scheme;
mod direction;
mod document;
mod element;
mod error;
mod eye_dropper;
mod file_dialog;
mod focus;
mod form;
mod http;
mod keyboard;
mod motion;
mod paint;
mod placeholder;
mod regex;
mod resize;
mod scroll;
mod select;
mod session;
mod table;
mod task;
mod timer;
mod transition;

// The renderer seams everything above `platform` reaches `backend` through (todo 820).
pub(crate) use backend::{Listener, Outlet, PortalEntry, PortalRoot, SheetWatch, element};
pub(crate) use click::{
    DoublePress, follow_pointer, hits_inline_boxes, nested_interactive, padding_press,
};
pub(crate) use clipboard::clipboard;
pub use clock::{ClockApi, clock};
pub use color_scheme::{ColorSchemeApi, ColorSchemeSubscription, color_scheme};
pub(crate) use direction::{
    apply_direction, clear_root_direction, forget_direction, set_root_direction, store_direction,
    stored_direction,
};
pub use document::{DocumentApi, document};
pub(crate) use element::{
    ContentSubscription, SCROLL_MARGIN_VAR, is_rtl, on_content_change, on_form_reset,
};
pub use element::{Dimensions, ElementApi, Read};
pub use error::PlatformError;
pub(crate) use eye_dropper::eye_dropper;
pub(crate) use file_dialog::pick_files;
pub(crate) use focus::{
    FocusMove, SilentFocusApi, SilentFocusSubscription, blur_counts, focus_entered_from,
    focus_pressed, focus_selectors, focus_visible, silent_focus,
};
pub(crate) use form::{submit_event, submit_listeners};
pub(crate) use http::fetch_text;
pub(crate) use keyboard::arrow_target;
pub(crate) use keyboard::key_taken;
pub(crate) use keyboard::logical_key;
pub(crate) use keyboard::typing_target;
pub(crate) use keyboard::warn_reserved_chord;
pub use keyboard::{KeyChord, KeySubscription, KeyboardApi, keyboard};
pub(crate) use motion::prefers_reduced_motion;
pub(crate) use paint::{
    aligns_logical_text, draws_backdrop_filter, fits_svg_images, paints_outer_inline_backgrounds,
};
pub(crate) use placeholder::{
    PLACEHOLDER_ATTR, PLACEHOLDER_CELL_ATTR, PLACEHOLDER_SHOWN_ATTR, draws_placeholders,
    placeholder_drawn,
};
pub(crate) use regex::{PreparedText, RegexMatch, regex_api};
pub(crate) use resize::{is_measured_resize, on_resize};
pub use scroll::{ScrollApi, ScrollSubscription, scroll};
pub(crate) use scroll::{clips_z_indexed, scroll_range, wheel_travel_y};
pub(crate) use select::opens_select_picker;
pub(crate) use session::{session_get, session_set};
pub(crate) use table::lays_out_captions;
pub(crate) use task::{next_task, when_free, when_laid_out};
pub use timer::{TimerApi, TimerSubscription, timer};
pub(crate) use transition::transition_property;
