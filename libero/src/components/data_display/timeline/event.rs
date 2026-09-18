use dioxus::prelude::*;

use crate::str_enum::str_enum;
use crate::{components::common::OptionLabel, sx::ThemeAwareValue};

str_enum! {
    /// How the connector *below* an event is drawn.
    ///
    /// Component-local, not in `theme/defaults/`: unlike `TimelineAlign` it
    /// has no themed default - `Solid` is a builder default - so
    /// `architecture.md`'s carve-out for enum props without a themed default
    /// applies.
    #[state_prefix = "line"]
    pub enum TimelineLine {
        #[default]
        Solid = "solid",
        Dashed = "dashed",
        Dotted = "dotted",
    }
}

/// One event on a [`Timeline`](super::Timeline).
///
/// A builder because the parts compose - a title alone is the common case, and
/// content, a custom bullet, a colour and a line style are each independently
/// optional. `TimelineEvent::new` rather than a free `event()`: too generic a
/// name to export at the crate root beside dioxus's `Event`.
#[derive(Clone, PartialEq)]
pub struct TimelineEvent {
    pub(super) title: OptionLabel,
    pub(super) content: Option<Element>,
    pub(super) bullet: Option<Element>,
    pub(super) color: Option<ThemeAwareValue>,
    pub(super) line: TimelineLine,
}

impl TimelineEvent {
    /// The event's name, optionally with its own rendering - rich content
    /// always *has* a plain-text name, which is what a screen reader reads.
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
    /// A bullet holding a child inverts when active: it *fills* with the
    /// accent, because a light glyph on a white ring vanishes.
    ///
    /// **Never put anything focusable in here.** The bullet is
    /// `aria-hidden="true"` - it is the rail's drawing, and the title is the
    /// text - but `aria-hidden` does not remove an element from the tab order.
    /// A `Button` or a link in a bullet stays tabbable and announces as
    /// nothing, which is a dead stop for a keyboard user. The type cannot
    /// prevent it and no `warn()` can detect focusability, so this note is the
    /// whole guard. Interactive content belongs in `.content(..)`.
    pub fn bullet(mut self, bullet: Element) -> Self {
        self.bullet = Some(bullet);
        self
    }

    /// This event's own accent, overriding the timeline's - an error step in
    /// an otherwise unremarkable run.
    pub fn color(mut self, color: impl Into<ThemeAwareValue>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// The connector drawn *below* this event. The last event has none.
    pub fn line(mut self, line: TimelineLine) -> Self {
        self.line = line;
        self
    }
}
