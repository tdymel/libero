//! `Text`'s one-line cuts natively (todo 3973): Blitz clips the overflow at the cut end,
//! but paints no `text-overflow` ellipsis.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{components::Text, sx::sx};

const LONG: &str = "Alpha beta gamma delta epsilon zeta eta theta";

fn app() -> Element {
    rsx! {
        Text { id: "end", truncate: "end", sx: sx().width("120px"), {LONG} }
        Text { id: "start", truncate: "start", sx: sx().width("120px"), {LONG} }
        Text { id: "dots", "\u{2026}" }
    }
}

/// The glyph ids painted inside the box, and how many lie left and right of it.
fn cut(page: &Page, selector: &str) -> (Vec<u32>, usize, usize) {
    let (left, _, width, _) = page.rect(selector);
    let glyphs = page.painted_glyphs(selector);
    let inside = glyphs
        .iter()
        .filter(|(_, x)| (left..left + width).contains(x))
        .map(|&(id, _)| id)
        .collect();
    let before = glyphs.iter().filter(|(_, x)| *x < left).count();
    let after = glyphs.iter().filter(|(_, x)| *x >= left + width).count();
    (inside, before, after)
}

#[test]
fn a_one_line_cut_clips_the_text_at_its_end_or_its_start() {
    let mut page = mount(app);
    page.settle();
    let (inside, before, after) = cut(&page, "#end");
    assert!(
        !inside.is_empty() && before == 0 && after > 0,
        "the end cut: {} inside, {before} before, {after} after",
        inside.len()
    );
    let (inside, before, after) = cut(&page, "#start");
    assert!(
        !inside.is_empty() && before > 0 && after == 0,
        "the start cut: {} inside, {before} before, {after} after",
        inside.len()
    );
}

#[test]
#[ignore = "Blitz paints no text-overflow ellipsis (todo 3973)"]
fn a_one_line_cut_paints_an_ellipsis() {
    let mut page = mount(app);
    page.settle();
    let dots = page.painted_glyphs("#dots")[0].0;
    for selector in ["#end", "#start"] {
        let (inside, ..) = cut(&page, selector);
        assert!(
            inside.contains(&dots),
            "{selector}: no ellipsis in {inside:?}"
        );
    }
}
