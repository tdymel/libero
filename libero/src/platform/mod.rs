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
mod clipboard;
mod document;
mod element;
mod error;
mod eye_dropper;
mod regex;
mod scroll;

pub use clipboard::ClipboardApi;
pub(crate) use clipboard::clipboard;
pub use document::{DocumentApi, document};
pub use element::{Dimensions, ElementApi, Read};
pub use error::PlatformError;
pub use eye_dropper::EyeDropperApi;
pub(crate) use eye_dropper::eye_dropper;
pub(crate) use regex::{PreparedText, RegexMatch, regex_api};
pub use scroll::{ScrollApi, ScrollSubscription, scroll};
