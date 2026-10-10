//! `GridZone` on its own: two half-width items and a named gap. Then a zone
//! filling a `Grid` area, a zone as a shrink-to-fit flex item, an item spanning
//! rows of its own, and a masonry zone.

use dioxus::prelude::*;
use libero::components::{Grid, GridArea, GridItem, GridSpan, GridTemplate, GridZone};
use libero::theme::responsive;

use crate::Routes;

pub const ROUTES: Routes = &[("/grid-zone", || rsx! { GridZonePage {} })];

#[derive(Clone, Copy, PartialEq)]
struct Body;

impl GridArea for Body {
    fn name(&self) -> &'static str {
        "body"
    }
}

#[component]
fn GridZonePage() -> Element {
    let template = GridTemplate::new()
        .row(|row| row.cells(Body, 4))
        .build()
        .expect("a rectangular template");
    // Full below `md`, half from it: the 400px zone is below, the viewport above.
    let span = responsive(GridSpan::Full).md(GridSpan::Half);
    rsx! {
        div { style: "width: 400px",
            GridZone { id: "zone", gap: "lg",
                GridItem { id: "a", span: GridSpan::Half, "A" }
                GridItem { id: "b", span: GridSpan::Half, "B" }
            }
        }
        div { style: "width: 400px",
            Grid { template,
                GridZone { id: "area-zone", area: Body,
                    GridItem { id: "responsive-a", span, "A" }
                    GridItem { id: "responsive-b", span, "B" }
                    div { id: "area-fixed", style: "position: fixed; top: 0; left: 0; width: 10px; height: 10px" }
                }
            }
        }
        div { style: "display: flex",
            GridZone { id: "loose-zone",
                GridItem { id: "loose-a", span: GridSpan::Half, "Loose A" }
                GridItem { id: "loose-b", span: GridSpan::Half, "Loose B" }
            }
        }
        div { style: "width: 400px",
            GridZone {
                GridItem { id: "tall", span: GridSpan::Half, rows: 2u8, "Tall" }
                GridItem { id: "right-1", span: GridSpan::Half, div { style: "height: 40px", "1" } }
                GridItem { id: "right-2", span: GridSpan::Half, div { style: "height: 40px", "2" } }
            }
        }
        div { style: "width: 400px",
            GridZone { masonry: true, gap: "lg",
                GridItem { id: "stone-a", div { style: "height: 30px", "A" } }
                GridItem { id: "stone-b", div { style: "height: 30px", "B" } }
            }
        }
    }
}
