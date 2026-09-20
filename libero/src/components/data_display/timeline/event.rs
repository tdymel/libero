use dioxus::prelude::*;

use crate::str_enum::str_enum;
use crate::{components::common::OptionLabel, sx::ThemeAwareValue};

str_enum! {
    /// How the connector below an event is drawn. Component-local: no themed default.
    #[state_prefix = "line"]
    pub enum TimelineLine {
        #[default]
        Solid = "solid",
        Dashed = "dashed",
        Dotted = "dotted",
    }
}

/// One event on a [`Timeline`](super::Timeline), built from its title.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{TimelineEvent, TimelineLine};
/// let event = TimelineEvent::new("Shipped")
///     .content(rsx! { "Out to all users." })
///     .line(TimelineLine::Dashed);
/// ```
#[derive(Clone, PartialEq)]
pub struct TimelineEvent {
    pub(super) title: OptionLabel,
    pub(super) content: Option<Element>,
    pub(super) bullet: Option<Element>,
    pub(super) color: Option<ThemeAwareValue>,
    pub(super) line: TimelineLine,
}

impl TimelineEvent {
    /// The event's name; rich content still carries the plain-text name a screen reader reads.
    pub fn new(title: impl Into<OptionLabel>) -> Self {
        Self {
            title: title.into(),
            content: None,
            bullet: None,
            color: None,
            line: TimelineLine::default(),
        }
    }

    /// The body below the title.
    pub fn content(mut self, content: Element) -> Self {
        self.content = Some(content);
        self
    }

    /// An icon or avatar inside the bullet, instead of the dot.
    ///
    /// **Nothing focusable**: the bullet is `aria-hidden`, so a link in it is a
    /// nameless tab stop. Interactive content belongs in `.content(..)`.
    pub fn bullet(mut self, bullet: Element) -> Self {
        self.bullet = Some(bullet);
        self
    }

    /// This event's own accent, overriding the timeline's.
    pub fn color(mut self, color: impl Into<ThemeAwareValue>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// The connector below this event. The last event has none.
    pub fn line(mut self, line: TimelineLine) -> Self {
        self.line = line;
        self
    }
}
