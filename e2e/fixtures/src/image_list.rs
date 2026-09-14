//! `ImageList`, and the gap arithmetic of its `quilted` variant.

use dioxus::prelude::*;
use libero::components::{GridSpan, ImageBar, ImageItem, ImageList};
use libero::theme::responsive;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/image-list-quilted", || rsx! { ImageListQuiltedPage {} }),
    (
        "/image-list-responsive",
        || rsx! { ImageListResponsivePage {} },
    ),
    ("/image-list-links", || rsx! { ImageListLinksPage {} }),
];

/// Todo 618: link cells fill their clipped `<li>`, and a bar holds a button.
#[component]
fn ImageListLinksPage() -> Element {
    let cell = || rsx! { div { style: "background: #777" } };

    rsx! {
        div { style: "width: 600px",
            ImageList {
                cols: 2u8,
                gap: "md",
                items: vec![
                    ImageItem::new(cell()).to("/image-list-quilted"),
                    ImageItem::new(cell())
                        .to("/image-list-responsive")
                        .bar(ImageBar::new(rsx! {
                            span { "Caption" }
                            button { id: "bar-action", "Act" }
                        })),
                ],
            }
        }
    }
}

/// Todo 73: one column, two from `sm` (48rem), four from `md` (62rem).
#[component]
fn ImageListResponsivePage() -> Element {
    let cell = || ImageItem::new(rsx! { div { style: "background: #777" } });

    rsx! {
        div { id: "responsive-frame",
            ImageList {
                cols: responsive(1).sm(2).md(4),
                gap: "md",
                items: vec![cell(), cell(), cell(), cell()],
            }
        }
    }
}

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
