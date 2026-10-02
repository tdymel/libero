//! `use_lightbox`'s rendered contract: a dialog, one picture per item with its
//! own `alt`, one tab stop on the picture showing, the caption linked to it,
//! the preload window, and a roving thumbnail strip. And `Image { zoomable }`,
//! which is the same viewer with one picture.

use std::collections::BTreeMap;

use crate::common::{attributes_of, body, render, render_with, tag_with, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Image,
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
};

fn gallery() -> Vec<LightboxItem> {
    vec![
        LightboxItem::new("/a.jpg", "A lake").caption("Morning"),
        LightboxItem::new("/b.jpg", "A hill").thumbnail("/b-small.jpg"),
        LightboxItem::new("/c.jpg", "A field"),
        LightboxItem::new("/d.jpg", "A town").caption("Evening"),
    ]
}

/// Opens `options` on the gallery at `index` on the first render.
fn open(options: LightboxOptions, index: usize) -> String {
    open_items(options, gallery(), index)
}

type Setup = (LightboxOptions, Vec<LightboxItem>, usize);

fn open_items(options: LightboxOptions, items: Vec<LightboxItem>, index: usize) -> String {
    /// `use_lightbox` needs the provider above it, so the setup reaches the
    /// opener through context.
    #[component]
    fn Opener() -> Element {
        let (options, items, index) = use_context::<Setup>();
        let lightbox = use_lightbox(options);
        use_hook(move || lightbox.open_with((items, index)));

        rsx! {}
    }

    fn app(setup: Setup) -> Element {
        use_context_provider(|| setup);
        rsx! {
            LiberoProvider { Opener {} }
        }
    }

    body(&render_with(app, (options, items, index)))
}

/// The pictures on the stage - the thumbnails' `alt` is empty.
fn stage_images(html: &str) -> Vec<BTreeMap<String, String>> {
    tags_with(html, "<img")
        .into_iter()
        .filter(|img| img.get("alt").is_some_and(|alt| !alt.is_empty()))
        .collect()
}

fn thumbnails(html: &str) -> Vec<BTreeMap<String, String>> {
    tags_with(html, "<button")
        .into_iter()
        .filter(|button| {
            button
                .get("aria-label")
                .is_some_and(|label| label.starts_with("Go to slide"))
        })
        .collect()
}

#[test]
fn a_gallery_is_a_named_modal_dialog() {
    let html = open(LightboxOptions::default(), 0);
    let dialog = tag_with(&html, r#"role="dialog""#);

    assert_eq!(dialog["role"], "dialog");
    assert_eq!(dialog["aria-modal"], "true");
    assert_eq!(dialog["aria-label"], "Gallery");
    assert!(
        html.contains(r#"aria-label="Close""#),
        "the dialog's own close button, named from the theme:\n{html}"
    );
}

fn button_named<'a>(
    buttons: &'a [BTreeMap<String, String>],
    name: &str,
) -> Option<&'a BTreeMap<String, String>> {
    buttons
        .iter()
        .find(|button| button.get("aria-label").is_some_and(|label| label == name))
}

/// Todo 864: zoom out, zoom in, then close. Fitted on open, so zoom out is
/// `aria-disabled` and keeps its tab stop; the close button takes the focus.
#[test]
fn the_toolbar_zooms_and_opens_fitted() {
    let html = open(LightboxOptions::default(), 0);
    let buttons = tags_with(&html, "<button");
    let names: Vec<&str> = buttons
        .iter()
        .filter_map(|button| button.get("aria-label").map(String::as_str))
        .take(3)
        .collect();

    assert_eq!(names, ["Zoom out", "Zoom in", "Close"], "{html}");
    assert_eq!(
        button_named(&buttons, "Zoom out").unwrap()["aria-disabled"],
        "true"
    );
    assert!(
        !button_named(&buttons, "Zoom in")
            .unwrap()
            .contains_key("aria-disabled")
    );
    assert!(
        button_named(&buttons, "Close")
            .unwrap()
            .contains_key("data-autofocus")
    );
}

/// Nothing to zoom, no zoom buttons: with `zoom` off, or a `max_zoom` of 1.
#[test]
fn the_toolbar_has_no_zoom_buttons_without_zoom() {
    for options in [
        LightboxOptions {
            zoom: false,
            ..LightboxOptions::default()
        },
        LightboxOptions {
            max_zoom: Some(1.0),
            ..LightboxOptions::default()
        },
    ] {
        let html = open(options, 0);
        let buttons = tags_with(&html, "<button");
        assert!(button_named(&buttons, "Zoom in").is_none(), "{html}");
        assert!(button_named(&buttons, "Close").is_some());
    }
}

/// The plan's `alt="" role=presentation` would have left a slide named only
/// "3 of 7". Each picture keeps its own text alternative.
#[test]
fn every_picture_keeps_its_own_alt() {
    let html = open(LightboxOptions::default(), 0);
    let alts: Vec<String> = stage_images(&html)
        .iter()
        .map(|img| img["alt"].clone())
        .collect();

    assert_eq!(alts, ["A lake", "A hill", "A field", "A town"]);
    assert!(!html.contains(r#"role="presentation""#));
}

/// The picture is the pan surface, so it takes the keys - one tab stop, on the
/// picture showing.
#[test]
fn only_the_picture_showing_is_a_tab_stop() {
    let html = open(LightboxOptions::default(), 2);
    let tabindex: Vec<String> = stage_images(&html)
        .iter()
        .map(|img| img["tabindex"].clone())
        .collect();

    assert_eq!(tabindex, ["-1", "-1", "0", "-1"]);
}

#[test]
fn without_zoom_no_picture_is_focusable() {
    let html = open(
        LightboxOptions {
            zoom: false,
            ..LightboxOptions::default()
        },
        0,
    );

    // `.all()` over nothing is true, so first prove the pictures are there.
    let images = stage_images(&html);
    assert_eq!(images.len(), 4, "{html}");
    assert!(images.iter().all(|img| !img.contains_key("tabindex")));
}

/// `preload: 1` - the picture showing and one neighbour each side load at
/// once, the rest when the browser gets to them.
#[test]
fn the_preload_window_is_eager_and_the_rest_lazy() {
    let html = open(LightboxOptions::default(), 0);
    let loading: Vec<String> = stage_images(&html)
        .iter()
        .map(|img| img["loading"].clone())
        .collect();

    assert_eq!(loading, ["eager", "eager", "lazy", "lazy"]);
}

/// Announced after the slide's position rather than in place of it.
#[test]
fn the_caption_describes_the_picture_showing() {
    let html = open(LightboxOptions::default(), 3);
    // `<p ` with the space: an icon's `<path` comes first.
    let caption = attributes_of(&html, "p");
    let images = stage_images(&html);

    assert!(
        html.contains(">Evening</p>"),
        "the current caption:\n{html}"
    );
    assert!(
        !html.contains("Morning"),
        "only the current caption renders"
    );
    // The caption first, then the keys hint (todo 565).
    let ids: Vec<&str> = images[3]["aria-describedby"].split(' ').collect();
    assert_eq!(ids.len(), 2, "{ids:?}");
    assert_eq!(ids[0], caption["id"]);
    assert!(
        html.contains(&format!("id=\"{}\" hidden=true", ids[1])),
        "no keys hint:\n{html}"
    );
    assert!(!images[0].contains_key("aria-describedby"));
}

/// Todo 565: with no caption the zoomable picture is described by its keys
/// alone, and with `zoom` off, which takes the keys away, by nothing.
#[test]
fn a_picture_without_a_caption_is_described_by_its_keys_alone() {
    let html = open(LightboxOptions::default(), 2);

    assert!(!html.contains("<p "));
    // `.all()` over nothing is true, so first prove the pictures are there.
    let images = stage_images(&html);
    assert_eq!(images.len(), 4, "{html}");
    let hint = &images[2]["aria-describedby"];
    assert!(
        html.contains(&format!(
            "id=\"{hint}\" hidden=true>Z, plus or minus to zoom"
        )),
        "{html}"
    );

    let html = open(
        LightboxOptions {
            zoom: false,
            ..LightboxOptions::default()
        },
        2,
    );
    assert!(
        stage_images(&html)
            .iter()
            .all(|img| !img.contains_key("aria-describedby"))
    );
}

#[test]
fn captions_off_hides_them() {
    let html = open(
        LightboxOptions {
            captions: false,
            ..LightboxOptions::default()
        },
        0,
    );

    assert!(!html.contains("Morning"));
}

/// One tab stop for the strip, on the current thumbnail, which says it is
/// current in a second channel beside its frame.
#[test]
fn the_thumbnail_strip_roves() {
    let html = open(LightboxOptions::default(), 1);
    let thumbnails = thumbnails(&html);

    assert_eq!(thumbnails.len(), 4);
    let tabindex: Vec<&str> = thumbnails.iter().map(|t| t["tabindex"].as_str()).collect();
    assert_eq!(tabindex, ["-1", "0", "-1", "-1"]);
    let current: Vec<bool> = thumbnails
        .iter()
        .map(|t| t.get("aria-current").is_some_and(|c| c == "true"))
        .collect();
    assert_eq!(current, [false, true, false, false]);
    assert_eq!(thumbnails[1]["aria-label"], "Go to slide 2");
}

#[test]
fn a_thumbnail_prefers_its_own_source() {
    let html = open(LightboxOptions::default(), 0);
    let sources: Vec<String> = tags_with(&html, "<img")
        .into_iter()
        .filter(|img| img.get("alt").is_some_and(String::is_empty))
        .map(|img| img["src"].clone())
        .collect();

    assert_eq!(sources, ["/a.jpg", "/b-small.jpg", "/c.jpg", "/d.jpg"]);
}

#[test]
fn thumbnails_off_leaves_no_strip() {
    let html = open(
        LightboxOptions {
            thumbnails: false,
            ..LightboxOptions::default()
        },
        0,
    );

    assert!(thumbnails(&html).is_empty());
    assert!(!html.contains(r#"aria-label="Thumbnails""#));
}

/// `Image { zoomable }` is the viewer with one picture: no carousel to page,
/// no strip, and the enlarged picture keeps the `alt`.
#[test]
fn a_zoomed_image_is_a_single_picture_viewer() {
    #[component]
    fn Zoomed() -> Element {
        rsx! {
            Image { src: "/small.png", zoomed_src: "/large.png", alt: "A lake", zoomable: true }
        }
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { Zoomed {} }
        }
    }

    let html = body(&render(app));
    let trigger = attributes_of(&html, "button");

    assert_eq!(trigger["aria-label"], "Zoom in: A lake");
    assert_eq!(trigger["aria-haspopup"], "dialog");
    // Closed: nothing but the trigger's own picture.
    assert_eq!(tags_with(&html, "<img").len(), 1);
    assert!(!html.contains("role=\"dialog\""));
}

/// One picture has nothing to page through: no carousel region, no "1 of 1",
/// no strip.
#[test]
fn a_single_picture_has_no_carousel_and_no_strip() {
    let html = open_items(
        LightboxOptions::default(),
        vec![LightboxItem::new("/a.jpg", "A lake")],
        0,
    );

    assert!(
        !html.contains("aria-roledescription"),
        "no carousel:\n{html}"
    );
    assert!(thumbnails(&html).is_empty());
    assert_eq!(stage_images(&html)[0]["tabindex"], "0");
}

/// An index past the end shows the last picture rather than panicking.
#[test]
fn an_index_past_the_end_shows_the_last_picture() {
    let html = open(LightboxOptions::default(), 99);
    let tabindex: Vec<String> = stage_images(&html)
        .iter()
        .map(|img| img["tabindex"].clone())
        .collect();

    assert_eq!(tabindex, ["-1", "-1", "-1", "0"]);
}

#[test]
fn a_plain_image_is_still_a_bare_img() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Image { src: "/x.png", alt: "X" } }
        }
    }

    let html = body(&render(app));

    assert!(!html.contains("<button"));
    assert_eq!(attributes_of(&html, "img")["alt"], "X");
}
