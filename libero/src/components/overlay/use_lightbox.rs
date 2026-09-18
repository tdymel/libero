use dioxus::prelude::*;

use super::use_modal::{ModalHandle, ModalScope, use_modal};
use crate::components::overlay::Lightbox;

/// One picture in a gallery.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightboxItem {
    pub src: String,
    /// The strip's smaller source. Falls back to `src`.
    pub thumbnail_src: Option<String>,
    pub alt: String,
    pub caption: Option<String>,
}

impl LightboxItem {
    pub fn new(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: alt.into(),
            ..Self::default()
        }
    }

    pub fn thumbnail(mut self, src: impl Into<String>) -> Self {
        self.thumbnail_src = Some(src.into());
        self
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }
}

/// What one opening shows: the gallery, and where it starts. The gallery
/// travels with the opening rather than with the hook, because a `use_modal`
/// render closure captures at its first render - a gallery passed there would
/// freeze.
#[derive(Clone, Debug, PartialEq)]
pub struct LightboxOpening {
    pub items: Vec<LightboxItem>,
    pub index: usize,
}

impl From<LightboxItem> for LightboxOpening {
    fn from(item: LightboxItem) -> Self {
        Self {
            items: vec![item],
            index: 0,
        }
    }
}

impl From<Vec<LightboxItem>> for LightboxOpening {
    fn from(items: Vec<LightboxItem>) -> Self {
        Self { items, index: 0 }
    }
}

impl From<(Vec<LightboxItem>, usize)> for LightboxOpening {
    fn from((items, index): (Vec<LightboxItem>, usize)) -> Self {
        Self { items, index }
    }
}

/// How the viewer behaves, shared by every opening.
#[derive(Clone, Debug, PartialEq)]
pub struct LightboxOptions {
    /// Wheel, double-click, `z`, `+` and `-` zoom; drag and arrow pan.
    pub zoom: bool,
    /// Upper scale bound. Defaults to the theme's.
    pub max_zoom: Option<f64>,
    /// The strip under the stage. Never shown for a single picture.
    pub thumbnails: bool,
    /// Shows [`LightboxItem::caption`].
    pub captions: bool,
    /// The previous / next arrows.
    pub controls: bool,
    /// Neighbours each side of the current picture loaded `eager`; the rest
    /// are `lazy`.
    pub preload: usize,
    /// A downward swipe on a touch screen closes. Off while zoomed, where a
    /// drag pans.
    pub close_on_swipe_down: bool,
    /// Names the dialog. Defaults to the theme's `"Gallery"`.
    pub aria_label: Option<String>,
}

impl Default for LightboxOptions {
    fn default() -> Self {
        Self {
            zoom: true,
            max_zoom: None,
            thumbnails: true,
            captions: true,
            controls: true,
            preload: 1,
            close_on_swipe_down: true,
            aria_label: None,
        }
    }
}

/// A modal image viewer: [`use_modal`] with a gallery around it - same handle,
/// same openings.
///
/// Open it from the thumbnail's own click handler, and focus goes back to that
/// thumbnail when it closes.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::hooks::{LightboxItem, LightboxOptions, use_lightbox};
/// # fn app() -> Element {
/// # struct Photo { url: String, alt: String, title: String }
/// # let photos: Vec<Photo> = Vec::new();
/// # let index = 0;
/// let lightbox = use_lightbox(LightboxOptions::default());
/// let items: Vec<LightboxItem> = photos.iter()
///     .map(|p| LightboxItem::new(&p.url, &p.alt).caption(&p.title))
///     .collect();
/// // in a thumbnail's `onclick`:
/// lightbox.open_with((items.clone(), index));
/// # rsx! {}
/// # }
/// ```
pub fn use_lightbox(options: LightboxOptions) -> ModalHandle<LightboxOpening> {
    use_modal(move |s: ModalScope<LightboxOpening>| {
        rsx! {
            Lightbox { opening: s.args(), options: options.clone() }
        }
    })
}
