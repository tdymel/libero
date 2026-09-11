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

pub(crate) mod backend;
mod click;
mod clipboard;
mod clock;
mod color_scheme;
mod document;
mod element;
mod error;
mod eye_dropper;
mod file_dialog;
mod focus;
mod form;
mod keyboard;
mod motion;
mod regex;
mod scroll;
mod select;
mod task;
mod timer;
mod transition;

pub(crate) use click::{follow_pointer, nested_interactive, padding_press};
pub(crate) use clipboard::clipboard;
pub use clock::{ClockApi, clock};
pub use color_scheme::{ColorSchemeApi, ColorSchemeSubscription, color_scheme};
pub use document::{DocumentApi, document};
pub use element::{Dimensions, ElementApi, Read};
pub use error::PlatformError;
pub(crate) use eye_dropper::eye_dropper;
pub(crate) use file_dialog::file_dialog;
pub(crate) use focus::{focus_entered_from, focus_pressed};
pub(crate) use form::{emulates_submit, implicit_submit, submit_click, submit_event};
pub(crate) use keyboard::arrow_target;
pub(crate) use keyboard::key_taken;
pub(crate) use keyboard::typing_target;
pub(crate) use keyboard::warn_reserved_chord;
pub use keyboard::{KeyChord, KeySubscription, KeyboardApi, keyboard};
pub(crate) use motion::prefers_reduced_motion;
pub(crate) use regex::{PreparedText, RegexMatch, regex_api};
pub use scroll::{ScrollApi, ScrollSubscription, scroll};
pub(crate) use select::select_picker;
pub(crate) use task::{next_task, when_free, when_laid_out};
pub use timer::{TimerApi, TimerSubscription, timer};
pub(crate) use transition::transition_property;
