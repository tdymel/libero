//! `ImageList`, and the gap arithmetic of its `quilted` variant.

use dioxus::prelude::*;
use libero::components::{GridSpan, ImageItem, ImageList};

use crate::Routes;

pub const ROUTES: Routes = &[("/image-list-quilted", || rsx! { ImageListQuiltedPage {} })];

/// Todo 89(c): every `quilted` shape against an ordinary cell, at gap `md`.
/// The test sets `#quilt-frame`'s width; the cells are flat colour, no pictures.
#[component]
fn ImageListQuiltedPage() -> Element {
    let cell = || rsx! { div { style: "background: #777" } };

    rsx! {
        div { id: "quilt-frame", style: "width: 600px",
            ImageList {
                cols: 2u8,
                variant: "quilted",
                gap: "md",
                items: vec![
                    ImageItem::new(cell()).rows(2),
                    ImageItem::new(cell()),
                    ImageItem::new(cell()),
                    ImageItem::new(cell()).span(GridSpan::Full),
                    ImageItem::new(cell()).span(GridSpan::Full).rows(2),
                ],
            }
        }
    }
}
