use dioxus::prelude::*;

use crate::theme::BarPosition;

use super::super::grid::GridSpan;

/// The caption strip over - or under - one cell.
///
/// It holds **whatever the caller renders**: `ImageList` owns where the strip
/// sits in the cell and, by default, the scrim behind it, and nothing else.
/// There is no title, subtitle or action slot - a caption is content, and a
/// component that shapes it can only ever be in the way of the next design.
///
/// The one thing to carry across from the old title/subtitle pair: **the
/// default scrim is a gradient, so its contrast has a range and not a number.**
/// Over a pure-white picture, white text is 9.3:1 at the strip's bottom edge
/// and 1.8:1 near its top, so a *second line* of a two-line bar over a *bright*
/// photograph is the case that fails. A `text-shadow` rescues it more cheaply
/// than a heavier gradient, which reads as a solid black band.
#[derive(Clone, PartialEq)]
pub struct ImageBar {
    pub(super) content: Element,
    pub(super) position: Option<BarPosition>,
    pub(super) scrim: bool,
}

impl ImageBar {
    /// The strip's content. It is **not** a label for the image: the two are
    /// siblings, so nothing here becomes the picture's accessible name - that
    /// is the image's own `alt`.
    ///
    /// A control in here stays clickable on a cell with a `to`: the bar takes
    /// a `z-index` above the stretched link. It is not inside the anchor, so a
    /// `<button>` here is valid HTML - but it inherits no colour, and an
    /// overlay bar's colour comes from the scrim, so give it
    /// `sx().color("inherit")`.
    pub fn new(content: Element) -> Self {
        Self {
            content,
            position: None,
            scrim: true,
        }
    }

    /// Overrides the list's `bar_position` for this cell.
    ///
    /// Where the strip sits is the cell's own layout - an overlay shares the
    /// picture's grid cell and a `Below` bar takes the implicit second row -
    /// so it stays a prop rather than something the content can express.
    pub fn position(mut self, position: BarPosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Turns the scrim behind an overlay bar off, along with the light text
    /// colour that goes with it - so the strip is a bare, transparent box in
    /// the right place and the design is entirely the caller's.
    ///
    /// On by default, because an overlay caption over a photograph is
    /// unreadable without one. `Below` never had a scrim.
    pub fn scrim(mut self, scrim: bool) -> Self {
        self.scrim = scrim;
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

    /// This cell's width, overriding the one the list's `cols` derives.
    ///
    /// The zone's twelfths, the same vocabulary a `GridItem` takes - a cell is
    /// a `GridItem`, so a span means here exactly what it means there.
    pub fn span(mut self, span: impl Into<GridSpan>) -> Self {
        self.span = Some(span.into());
        self
    }

    /// This cell's height, in rows - the `quilted` variant's whole vocabulary.
    ///
    /// Ignored by every other variant, with a warn: `standard` has one row per
    /// cell by definition, and `masonry` derives the row span from the
    /// measured height.
    pub fn rows(mut self, rows: u8) -> Self {
        self.rows = Some(rows.max(1));
        self
    }

    pub fn bar(mut self, bar: ImageBar) -> Self {
        self.bar = Some(bar);
        self
    }

    /// Makes the cell a link.
    ///
    /// The anchor is the **picture**, and a stretched `::after` extends the hit
    /// area over the whole tile - so the accessible name is the image's `alt`,
    /// and a cell with a decorative image (`alt: ""`) gives the link no name at
    /// all. Name the image, or put a link of your own in the bar.
    ///
    /// The bar is not part of the hit area: it sits above the stretched link so
    /// that a control the caller put in it still works.
    pub fn to(mut self, to: impl Into<NavigationTarget>) -> Self {
        self.to = Some(to.into());
        self
    }
}
