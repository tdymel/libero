//! `ImageList`, the gap arithmetic of its `quilted` variant, and `masonry` order.

use dioxus::prelude::*;
use libero::components::{GridSpan, Image, ImageBar, ImageItem, ImageList};
use libero::theme::{BarPosition, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/image-list-quilted", || rsx! { ImageListQuiltedPage {} }),
    (
        "/image-list-responsive",
        || rsx! { ImageListResponsivePage {} },
    ),
    ("/image-list-links", || rsx! { ImageListLinksPage {} }),
    ("/image-list-masonry", || rsx! { ImageListMasonryPage {} }),
    ("/image-list-captions", || rsx! { ImageListCaptionsPage {} }),
    ("/image-list-focus", || rsx! { ImageListFocusPage {} }),
    (
        "/image-list-quilted-below",
        || rsx! { ImageListQuiltedBelowPage {} },
    ),
];

/// Todo 2523: unclipped `Below` bars on quilted cells of one and two rows.
#[component]
fn ImageListQuiltedBelowPage() -> Element {
    let cell = || rsx! { div { style: "background: #777" } };
    let below = || ImageBar::new(rsx! { span { "Caption" } }).position(BarPosition::Below);

    rsx! {
        div { style: "width: 600px",
            ImageList {
                cols: 2u8,
                variant: "quilted",
                gap: "md",
                items: vec![
                    ImageItem::new(cell()).rows(2).bar(below()),
                    ImageItem::new(cell()).bar(below()),
                    ImageItem::new(cell()),
                    ImageItem::new(cell()).span(GridSpan::Full).bar(below()),
                ],
            }
        }
    }
}

const PICTURE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 4 3'%3E%3Crect width='4' height='3' fill='%23369'/%3E%3C/svg%3E";

/// Todo 609: a captioned cell is a `figure` with a `figcaption`, a bare one is not.
#[component]
fn ImageListCaptionsPage() -> Element {
    let picture = |alt: &str| rsx! { Image { src: PICTURE, alt: alt.to_string() } };

    rsx! {
        div { style: "max-width: 600px",
            ImageList {
                cols: 2u8,
                items: vec![
                    ImageItem::new(picture("A blue field"))
                        .bar(ImageBar::new(rsx! { span { "Morning" } })),
                    ImageItem::new(picture("Another blue field")),
                ],
            }
        }
    }
}

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
                            span { id: "bar-caption", "Caption" }
                            button { id: "bar-action", "Act" }
                        })),
                ],
            }
        }
    }
}

/// Todos 2425, 2426, 2481, 2577: an unlinked zoomable picture, buttons flush with a
/// `Below` bar's start and end, and a focusable `video`.
#[component]
fn ImageListFocusPage() -> Element {
    rsx! {
        div { style: "width: 600px",
            ImageList {
                cols: 2u8,
                gap: "md",
                items: vec![
                    ImageItem::new(rsx! { Image { src: PICTURE, alt: "A blue field", zoomable: true } }),
                    ImageItem::new(rsx! { Image { src: PICTURE, alt: "Another blue field" } })
                        .bar(ImageBar::new(rsx! {
                            button { id: "below-start", "Pick" }
                            span { style: "flex: 1", "Caption" }
                            button { id: "below-action", "Act" }
                        }).position(BarPosition::Below)),
                    ImageItem::new(rsx! {
                        video { id: "cell-video", controls: true, tabindex: "0", style: "background: #369" }
                    }),
                ],
            }
        }
    }
}

/// Todo 610: cells of uneven heights, so `masonry` packs them short under
/// tall.
#[component]
fn ImageListMasonryPage() -> Element {
    const HEIGHTS: [u32; 9] = [120, 40, 80, 60, 100, 30, 90, 50, 70];

    rsx! {
        div { id: "masonry-frame", style: "width: 600px",
            ImageList {
                cols: 3u8,
                variant: "masonry",
                gap: "sm",
                items: HEIGHTS
                    .iter()
                    .map(|height| ImageItem::new(rsx! { div { style: "height: {height}px; background: #777" } }))
                    .collect::<Vec<_>>(),
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
