//! What reaches below dioxus (DOM, clipboard, timers, regex): one trait per
//! capability, one accessor each, one implementation per renderer.
//!
//! Sits below `hooks` and `components`; nothing here reaches back up. Elements
//! come through [`use_element`](crate::hooks::use_element), not an accessor.

mod a11y_media;
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
mod intersection;
mod keyboard;
mod max_length;
mod media_query;
mod motion;
mod paint;
mod placeholder;
mod press;
mod regex;
mod resize;
mod scroll;
mod select;
mod session;
mod table;
mod task;
mod timer;
mod transition;

pub(crate) use a11y_media::{
    A11yAnswers, A11yMediaApi, a11y_media, answer_a11y_media, answers_a11y_media,
    current_a11y_answers, set_current_a11y_answers,
};
// Renderer seams that everything above reaches `backend` through (820).
pub(crate) use backend::{Listener, Outlet, PortalEntry, PortalRoot, SheetWatch, element};
pub(crate) use click::{
    DoublePress, follow_pointer, hits_inline_boxes, nested_interactive, padding_press,
    reads_click_targets,
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
    ContentSubscription, FocusStep, SCROLL_MARGIN_VAR, focus_among, focus_kept, focus_selector,
    focused_attribute, is_rtl, keep_focused, on_content_change, on_form_reset, set_value_by_id,
};
pub use element::{Dimensions, ElementApi, Read};
pub use error::PlatformError;
pub(crate) use eye_dropper::eye_dropper;
pub(crate) use file_dialog::pick_files;
pub(crate) use focus::{
    FocusMove, SilentFocusApi, SilentFocusSubscription, blur_counts, element_contains,
    focus_entered_from, focus_is_in, focus_pressed, focus_selectors, focus_visible, silent_focus,
};
pub(crate) use form::{lifts_legends, submit_event, submit_listeners};
pub(crate) use http::fetch_text;
pub(crate) use intersection::{
    OBSERVE_ATTR, computed_px_by_tag, next_observe_tag, observes_by_tag, on_intersection,
};
pub(crate) use keyboard::arrow_target;
pub(crate) use keyboard::caret_edges;
pub(crate) use keyboard::key_taken;
pub(crate) use keyboard::logical_key;
pub(crate) use keyboard::mod_is_meta;
pub(crate) use keyboard::soft_keyboard_app;
pub(crate) use keyboard::typing_target;
pub(crate) use keyboard::warn_reserved_chord;
pub use keyboard::{KeyChord, KeySubscription, KeyboardApi, keyboard};
pub(crate) use max_length::fit_max_length;
pub use media_query::{MediaQueryApi, MediaQuerySubscription, media_query};
pub(crate) use motion::prefers_reduced_motion;
pub(crate) use paint::{
    aligns_logical_text, clips_background_to_text, colors_form_controls, draws_backdrop_filter,
    fires_image_errors, fits_svg_images, paints_outer_inline_backgrounds,
};
pub(crate) use placeholder::{
    PLACEHOLDER_ATTR, PLACEHOLDER_CELL_ATTR, PLACEHOLDER_SHOWN_ATTR, draws_placeholders,
    placeholder_drawn,
};
pub(crate) use press::{PRESS_MARKER_ATTR, PressApi, PressSubscription, press};
pub(crate) use regex::{PreparedText, RegexMatch, regex_api};
pub(crate) use resize::{is_measured_resize, on_resize};
pub(crate) use scroll::{
    SCROLL_QUIET, clips_z_indexed, draws_own_scrollbars, fires_scroll_end, scroll_range,
    snaps_scroll, wheel_travel_y,
};
pub use scroll::{ScrollApi, ScrollSubscription, scroll};
pub(crate) use select::opens_select_picker;
pub(crate) use session::{session_get, session_set};
pub(crate) use table::{lays_out_captions, widens_sized_tables};
pub(crate) use task::{next_task, when_free, when_laid_out};
pub use timer::{TimerApi, TimerSubscription, timer};
pub(crate) use transition::transition_property;
