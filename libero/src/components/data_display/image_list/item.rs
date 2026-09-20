use dioxus::prelude::*;

use crate::theme::BarPosition;

use crate::components::layout::GridSpan;

/// The caption strip over, or under, one cell, holding whatever the caller renders.
///
/// The default scrim is a gradient: a second text line over a bright photo can
/// fall to 1.8:1. A `text-shadow` rescues it.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::ImageBar;
/// let bar = ImageBar::new(rsx! { "Harbour" }).scrim(true);
/// ```
#[derive(Clone, PartialEq)]
pub struct ImageBar {
    pub(super) content: Element,
    pub(super) position: Option<BarPosition>,
    pub(super) scrim: bool,
}

impl ImageBar {
    /// The strip's content; never the image's accessible name. A `<button>`
    /// here stays clickable over a `to` link, but give it `sx().color("inherit")`.
    pub fn new(content: Element) -> Self {
        Self {
            content,
            position: None,
            scrim: true,
        }
    }

    /// Overrides the list's `bar_position` for this cell.
    pub fn position(mut self, position: BarPosition) -> Self {
        self.position = Some(position);
        self
    }

    /// The scrim and light text behind an overlay bar; on by default.
    pub fn scrim(mut self, scrim: bool) -> Self {
        self.scrim = scrim;
        self
    }
}

/// One cell of an [`ImageList`](super::ImageList), usually an `Image` with `fit: "cover"`.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Image, ImageItem};
/// let item = ImageItem::new(rsx! { Image { src: "/a.jpg", alt: "A harbour", fit: "cover" } })
///     .span("half");
/// ```
#[derive(Clone, PartialEq)]
pub struct ImageItem {
    pub(super) content: Element,
    pub(super) span: Option<GridSpan>,
    pub(super) rows: Option<u8>,
    pub(super) bar: Option<ImageBar>,
    pub(super) to: Option<NavigationTarget>,
}

impl ImageItem {
    pub fn new(content: Element) -> Self {
        Self {
            content,
            span: None,
            rows: None,
            bar: None,
            to: None,
        }
    }

    /// This cell's width in twelfths, as a `GridItem` takes it; overrides `cols`.
    pub fn span(mut self, span: impl Into<GridSpan>) -> Self {
        self.span = Some(span.into());
        self
    }

    /// This cell's height in rows. `quilted` only; other variants warn and ignore it.
    pub fn rows(mut self, rows: u8) -> Self {
        self.rows = Some(rows.max(1));
        self
    }

    pub fn bar(mut self, bar: ImageBar) -> Self {
        self.bar = Some(bar);
        self
    }

    /// Makes the cell a link, named by the image's `alt`: a decorative image
    /// leaves it nameless. Wins over a `zoomable` `Image`.
    pub fn to(mut self, to: impl Into<NavigationTarget>) -> Self {
        self.to = Some(to.into());
        self
    }
}
