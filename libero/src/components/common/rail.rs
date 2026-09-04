//! The marker-and-connector geometry shared by a vertical rail: `Timeline`
//! today, `Stepper`'s vertical arm next.
//!
//! **This is a shape contract, not a convenience.** The two components must
//! not drift apart geometrically - a stepper and a timeline on the same page
//! with different rail insets reads as a bug - so the formulae and the
//! `::before` recipe live here once and both call in.
//!
//! Everything is taken as a **CSS length expression**, never a baked variable
//! name: `Timeline` resolves `--lsx-timeline-*` and `Stepper` will resolve
//! `--lsx-stepper-*`. Sharing geometry must not mean sharing a theme
//! namespace, which would couple two components' theming for nothing.
//!
//! **On the formula, and why ours differs.** The E4 plan quotes Mantine's
//! `--offset: calc(bullet/2 + line/2)` as the single inset expression. Both
//! are correct; they measure different things from different origins.
//!
//! Mantine draws the connector as the **item's own `border-left`**, so its
//! origin is the *line's* leading edge. With the bullet centred on the line's
//! centreline (at `line / 2`), the bullet spans
//! `[line/2 - bullet/2, line/2 + bullet/2]`, so its trailing edge - where
//! content has to begin - is at `bullet/2 + line/2`. `--offset` is therefore a
//! **content inset measured from a line origin**. It is not a centreline, and
//! it does not transfer to a box model where the connector is an absolutely
//! positioned `::before` and the marker's leading edge sits at the item's
//! edge.
//!
//! In our origin that same quantity is `marker` plus the space term, which is
//! what [`Rail::content_inset`] returns. Hence three named methods rather than
//! one `offset()`: three different measurements were being carried by one
//! name, and the name is what got copied without its origin.
//!
//! `libero/tests/all/timeline.rs` asserts the emitted values, so the geometry is
//! pinned by what it renders rather than by which formula was quoted.

use crate::sx::{Sx, sx};

/// The lengths a rail is built from, each a CSS expression the caller has
/// already resolved from its own theme vars.
pub(crate) struct Rail {
    /// Diameter of the bullet, dot or numbered marker.
    pub marker: String,
    /// Thickness of the connector.
    pub line: String,
    /// Space between two events, which is also how far the connector has to
    /// reach past the bottom of its own item.
    pub gap: String,
    /// `border-left`'s value after the width - `"solid"`, `"dashed"`,
    /// `"dotted"`, or a var reading a per-item override.
    pub style: String,
    /// The connector's colour, as a CSS expression.
    pub color: String,
}

impl Rail {
    /// Half a marker: the distance from the item's rail-side edge to the
    /// rail's **centreline**, as a bare expression fragment.
    ///
    /// Private, and every public method below composes it, so the centreline
    /// is defined once rather than spelled `marker / 2` at four sites. It is a
    /// fragment rather than a wrapped `calc(..)` so the callers stay flat -
    /// nested `calc()` is valid CSS but harder to read in a diff, and these
    /// expressions are read far more often than they are written.
    ///
    /// There is deliberately no public `center()`. Nothing calls it: the
    /// centred marker wants [`Rail::centred_marker_start`] and the one-sided
    /// marker sits at the item's edge and needs no expression at all. If
    /// `Stepper` turns out to want the bare centreline, make this `pub` at
    /// that point, with its caller.
    fn half_marker(&self) -> String {
        format!("{} / 2", self.marker)
    }

    /// Half a connector's thickness - what an expression backs off by to sit
    /// centred on a line rather than beside it.
    fn half_line(&self) -> String {
        format!("{} / 2", self.line)
    }

    /// Where a connector of width `line` has to start for its centre to land
    /// on [`Rail::center`].
    pub fn connector_start(&self) -> String {
        format!("calc({} - {})", self.half_marker(), self.half_line())
    }

    /// Where content sits in a **one-sided** rail: clear of the marker, plus
    /// one space. See [`Rail::centred_content_inset`] for the centred twin.
    pub fn content_inset(&self, space: &str) -> String {
        format!("calc({} + {space})", self.marker)
    }

    /// Where the marker starts in a **centred** rail, so that it straddles the
    /// midline rather than sitting beside it.
    ///
    /// Paired with [`Rail::centred_content_inset`]: the marker's trailing edge
    /// is at `50% + marker/2`, which is exactly where that inset begins
    /// measuring its space from. Kept here rather than composed at the call
    /// site so the two cannot be changed apart.
    pub fn centred_marker_start(&self) -> String {
        format!("calc(50% - {})", self.half_marker())
    }

    /// The same clearance for a **centred** rail, where the marker straddles
    /// the midline: content starts a full half-marker past 50%, not at 50%.
    ///
    /// Getting this wrong is silent - the inset still looks plausible, and the
    /// clearance comes out as `space - marker/2`, so it shrinks as the marker
    /// grows and goes negative on the larger steps of a size scale. SSR cannot
    /// see it.
    ///
    /// **Unstated invariant of the centred arm**: the two `50%`s resolve
    /// against different boxes - percentage padding against the containing
    /// block, `left` on the marker against the item's padding box - and they
    /// agree only while the item is exactly the list's content width. A column
    /// flex at the default `align-items: stretch` gives that. Setting
    /// `align-items` on the list breaks it.
    pub fn centred_content_inset(&self, space: &str) -> String {
        format!("calc(50% + {} + {space})", self.half_marker())
    }

    /// The connector, as an absolutely positioned `::before` on the item.
    ///
    /// It runs from the bottom of the marker to `calc(gap * -1)` - past the
    /// item's own box and into the space before the next one - which is what
    /// makes one continuous rail out of separate items. `border-left` rather
    /// than a background, so `dashed` and `dotted` come free.
    ///
    /// `:not(:last-of-type)` rather than a token on the last item: a rail that
    /// runs past its final marker points at nothing, and the selector needs no
    /// help from the component to know which item is last.
    pub fn connector_sx(&self, inset: RailInset) -> Sx {
        let declarations = sx()
            .content("\"\"")
            .position("absolute")
            .top(self.marker.clone())
            .bottom(format!("calc({} * -1)", self.gap))
            .with(
                "border-left",
                format!("{} {} {}", self.line, self.style, self.color),
            );

        let placed = match inset {
            RailInset::Start => declarations.left(self.connector_start()),
            RailInset::End => declarations.right(self.connector_start()),
            // Centred: the rail is on the item's midline, so it backs itself
            // off by its own half-width rather than by the marker's.
            RailInset::Center => declarations.left(format!("calc(50% - {})", self.half_line())),
        };

        sx().selector("&:not(:last-of-type)::before", placed)
    }
}

/// Which edge of the item the rail runs down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RailInset {
    Start,
    End,
    Center,
}
