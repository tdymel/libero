//! Marker-and-connector geometry for a vertical rail, one shape for `Timeline` and `Stepper`.
//! Lengths are CSS expressions, so the two share geometry, not a theme namespace.

use crate::sx::{Sx, sx};

/// The lengths a rail is built from, each a resolved CSS expression.
pub(crate) struct Rail {
    /// Diameter of the marker.
    pub marker: String,
    /// Thickness of the connector.
    pub line: String,
    /// Space between two events; the connector reaches this far past its item.
    pub gap: String,
    /// `border-left`'s style: `"solid"`, `"dashed"`, `"dotted"` or a var.
    pub style: String,
    pub color: String,
}

impl Rail {
    /// Distance to the rail's centreline, as a bare fragment so callers stay one flat `calc`.
    fn half_marker(&self) -> String {
        format!("{} / 2", self.marker)
    }

    fn half_line(&self) -> String {
        format!("{} / 2", self.line)
    }

    /// Where a connector starts for its centre to land on the centreline.
    pub fn connector_start(&self) -> String {
        format!("calc({} - {})", self.half_marker(), self.half_line())
    }

    /// Where content sits in a one-sided rail: clear of the marker, plus one space.
    pub fn content_inset(&self, space: &str) -> String {
        format!("calc({} + {space})", self.marker)
    }

    /// Where the marker starts in a centred rail, straddling the midline.
    pub fn centred_marker_start(&self) -> String {
        format!("calc(50% - {})", self.half_marker())
    }

    /// Content inset in a centred rail, a half-marker past 50%. The two `50%`s agree only
    /// while the item spans the list's width: don't set `align-items` on the list.
    pub fn centred_content_inset(&self, space: &str) -> String {
        format!("calc(50% + {} + {space})", self.half_marker())
    }

    /// The connector: a `::before` from the marker into the gap below the item.
    /// `border-left`, so `dashed` and `dotted` come free.
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
            // On the midline: backs off by its own half-width, not the marker's.
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
