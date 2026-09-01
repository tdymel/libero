use dioxus::prelude::*;

use crate::{components::OptionLabel, theme::BarPosition};

use super::super::grid::GridSpan;

/// The caption strip over - or under - one cell.
///
/// A builder, like [`ImageItem`]: a title alone is the common case, and the
/// subtitle, the action and the position are each independently optional.
#[derive(Clone, PartialEq)]
pub struct ImageBar {
    pub(super) title: OptionLabel,
    pub(super) subtitle: Option<OptionLabel>,
    pub(super) action: Option<Element>,
    pub(super) position: Option<BarPosition>,
}

impl ImageBar {
    /// The caption. It is **not** a label for the image: the two are siblings,
    /// so nothing here becomes the picture's accessible name - that is the
    /// image's own `alt`, and the title may repeat it or not.
    pub fn new(title: impl Into<OptionLabel>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            action: None,
            position: None,
        }
    }

    /// A second, dimmer line under the title.
    pub fn subtitle(mut self, subtitle: impl Into<OptionLabel>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// A control at the end of the bar - an `ActionIcon`, usually.
    ///
    /// It stays clickable on a cell with a `to`, because it is a sibling of
    /// the link rather than inside it: a button inside an anchor is invalid
    /// HTML, and the cell's hit area is a stretched pseudo-element the action
    /// sits above.
    pub fn action(mut self, action: Element) -> Self {
        self.action = Some(action);
        self
    }

    /// Overrides the list's `bar_position` for this cell.
    pub fn position(mut self, position: BarPosition) -> Self {
        self.position = Some(position);
        self
    }
}

/// One cell of an [`ImageList`](super::ImageList).
///
/// The content is whatever the caller renders - an `Image`, usually, with
/// `fit: "cover"`. `ImageList` never invents an `alt`.
#[derive(Clone, PartialEq)]
pub struct ImageItem {
    pub(super) content: Element,
    pub(super) span: Option<GridSpan>,
    pub(super) bar: Option<ImageBar>,
    pub(super) to: Option<NavigationTarget>,
}

impl ImageItem {
    pub fn new(content: Element) -> Self {
        Self {
            content,
            span: None,
            bar: None,
            to: None,
        }
    }

    /// This cell's width, overriding the one the list's `cols` derives.
    ///
    /// The zone's twelfths, the same vocabulary a `GridItem` takes - a cell is
    /// a `GridItem`, so a span means here exactly what it means there.
    pub fn span(mut self, span: impl Into<GridSpan>) -> Self {
        self.span = Some(span.into());
        self
    }

    pub fn bar(mut self, bar: ImageBar) -> Self {
        self.bar = Some(bar);
        self
    }

    /// Makes the cell a link.
    ///
    /// The anchor wraps the bar's title, or the image when there is no bar,
    /// and a stretched `::after` extends the hit area over the whole tile.
    /// So the accessible name is the title's text, or the image's `alt` - a
    /// cell with no bar and a decorative image (`alt: ""`) gives the link no
    /// name at all, which is the one arrangement to avoid.
    pub fn to(mut self, to: impl Into<NavigationTarget>) -> Self {
        self.to = Some(to.into());
        self
    }
}
