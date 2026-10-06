//! `Blockquote`'s rendered contract: the spec-correct shape, the three
//! attribution props that mean three different things, and the focus-contrast
//! twin a tinted surface owes anything focusable inside it.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Blockquote};

fn full_app() -> Element {
    rsx! {
        LiberoProvider {
            Blockquote {
                color: "info",
                attribution: rsx! { "Albert Einstein" },
                work: "Letter to his son",
                cite_url: "https://example.org/42",
                "Life is like riding a bicycle."
            }
        }
    }
}

fn bare_app() -> Element {
    rsx! {
        LiberoProvider { Blockquote { "A quote with nothing attached." } }
    }
}

/// Why the attribution sits outside the quote. WHATWG: "Attribution for the
/// quotation, if any, must be placed outside the blockquote element" - nested,
/// assistive technology and quote-extraction tools read the attribution as
/// quoted words.
#[test]
fn the_attribution_sits_outside_the_quote() {
    let html = body(&render(full_app));

    assert!(html.contains("<figure"), "{html}");
    let quote_end = html.find("</blockquote>").expect("a blockquote");
    let caption = html.find("<figcaption").expect("a figcaption");
    assert!(
        caption > quote_end,
        "attribution is inside the quote:\n{html}"
    );
    assert!(!html[..quote_end].contains("Albert Einstein"), "{html}");
}

/// A quote with no attribution still renders the same root, so a caller's
/// `sx`, `class` and attributes always land on the same element.
#[test]
fn the_figure_is_the_root_whether_or_not_there_is_an_attribution() {
    let bare = body(&render(bare_app));

    assert!(bare.contains("<figure"), "{bare}");
    assert!(!bare.contains("<figcaption"), "{bare}");
}

/// `<cite>` is the title of a work. The spec says it "must therefore not be
/// used to mark up people's names", so the person goes in the `<figcaption>`
/// as text and only `work` becomes a `<cite>`.
#[test]
fn only_the_work_becomes_a_cite_element() {
    let html = body(&render(full_app));

    assert!(html.contains("<cite"), "{html}");
    let cite_open = html.find("<cite").expect("a cite");
    let cite_close = html.find("</cite>").expect("a closed cite");
    let inside = &html[cite_open..cite_close];

    assert!(inside.contains("Letter to his son"), "{html}");
    assert!(
        !inside.contains("Albert Einstein"),
        "a name inside <cite>:\n{html}"
    );
}

/// Two adjacent inline nodes have nothing between them, so a speaker and a
/// work ran together as "Albert EinsteinLetter to his son". The comma is the
/// spec's own worked example, and it only appears when there are two halves to
/// separate.
#[test]
fn a_speaker_and_a_work_are_separated() {
    let html = body(&render(full_app));
    assert!(html.contains("Albert Einstein, <cite"), "{html}");

    fn work_only() -> Element {
        rsx! {
            LiberoProvider { Blockquote { work: "Letter to his son", "A quote." } }
        }
    }
    let html = body(&render(work_only));
    assert!(!html.contains(", <cite"), "{html}");
}

/// The third thing called "cite": an attribute, on the quote, that no browser
/// renders.
#[test]
fn cite_url_is_an_attribute_on_the_blockquote() {
    let html = render(full_app);

    assert_eq!(
        attributes_of(&html, "blockquote")["cite"],
        "https://example.org/42"
    );
    // And it is absent rather than empty when unset.
    assert!(
        !body(&render(bare_app)).contains("cite="),
        "{}",
        body(&render(bare_app))
    );
}

/// The tint, the bar and the body text come from one resolution, so they
/// cannot disagree - a tint from one shade with contrast text for another is
/// the unreadable combination `component-building-rules` warns about.
#[test]
fn one_colour_resolves_the_tint_the_bar_and_the_text() {
    let style = attributes_of(&render(full_app), "blockquote")["style"].clone();

    // The fill ramp, the one `contrast-N` is computed on (todo 605).
    assert!(
        style.contains("--lsx-blockquote-background:var(--lsx-info-fill-1)"),
        "{style}"
    );
    assert!(
        style.contains("--lsx-blockquote-border-color:var(--lsx-info-6)"),
        "{style}"
    );
    // The tint's own contrast twin, never an accent shade.
    assert!(
        style.contains("--lsx-blockquote-color:var(--lsx-info-contrast-1)"),
        "{style}"
    );
}

/// A quote containing a link is an ordinary thing, and `sx`'s `background()`
/// only infers `--lsx-focus-contrast` from a *literal* colour - ours is a
/// `var()`, which is opaque to it. Without this the ring falls back to
/// `primary.6` on a `primary.1` tint.
#[test]
fn a_tinted_quote_publishes_the_focus_contrast_twin() {
    let html = render(full_app);
    let style = attributes_of(&html, "blockquote")["style"].clone();

    assert!(
        style.contains("--lsx-focus-contrast:var(--lsx-info-contrast-1)"),
        "{style}"
    );
    // Published *badly* is worse than not at all: a var that is set but whose
    // referent is undeclared never falls back, and the ring disappears. So the
    // referent has to exist.
    assert!(html.contains("--lsx-info-contrast-1:"), "{html}");
}

/// Todo 605: the brand `info.6` under its twin was white at 2.78:1. The twin
/// is computed on `fill-6`, so an explicit shade paints that.
#[test]
fn an_explicit_shade_paints_the_fill_its_twin_reads_on() {
    fn shaded() -> Element {
        rsx! {
            LiberoProvider { Blockquote { color: "info.6", "A quote." } }
        }
    }
    let style = attributes_of(&render(shaded), "blockquote")["style"].clone();

    for pair in [
        "--lsx-blockquote-background:var(--lsx-info-fill-6)",
        "--lsx-blockquote-color:var(--lsx-info-contrast-6)",
        "--lsx-focus-contrast:var(--lsx-info-contrast-6)",
        // Todo 630: the fill is the ring's halo, or a white stripe sits on white.
        "--lsx-focus-ring-halo:var(--lsx-info-fill-6)",
    ] {
        assert!(style.contains(pair), "{pair} in {style}");
    }
}

/// Both size axes are independent, and both have to reach the stylesheet -
/// a `data-state` token with no rule behind it is the silent half of this.
#[test]
fn size_and_radius_are_independent_axes_with_rules_behind_them() {
    let html = render(bare_app);
    let class = attributes_of(&html, "blockquote")["class"]
        .split_whitespace()
        .next()
        .expect("a framework class")
        .to_string();

    assert_eq!(
        attributes_of(&html, "blockquote")["data-state"],
        "size-md radius-sm"
    );
    assert!(
        html.contains(&format!(".{class}[data-state~=\"size-md\"]{{padding:")),
        "{html}"
    );
    assert!(
        html.contains(&format!(
            ".{class}[data-state~=\"radius-sm\"]{{border-top-right-radius:"
        )),
        "{html}"
    );
}

/// The attribution is a *sibling* of the quote, so an `em` on it resolves
/// against the figure and never sees the quote's font size: at `xxl` it sat at
/// 13.6px under 22px text, and at `xs` it was larger than the quote. It needs
/// the size token and a rule of its own behind it.
#[test]
fn the_attribution_scales_with_the_quote() {
    fn large() -> Element {
        rsx! {
            LiberoProvider {
                Blockquote { size: "xxl", attribution: rsx! { "Albert Einstein" }, "A quote." }
            }
        }
    }
    let html = render(large);
    let caption = first_class(&html, "figcaption");

    assert_eq!(attributes_of(&html, "figcaption")["data-state"], "size-xxl");
    assert!(
        html.contains(&format!(
            ".{caption}[data-state~=\"size-xxl\"]{{font-size:calc(var(--lsx-text-font-size-xxl)"
        )),
        "{html}"
    );
}

/// Todo 2484: `opacity` dimmed a link in the attribution with the caption, to
/// about 2.4:1. The caption dims its own ink, so a child with a colour keeps it.
#[test]
fn the_attribution_dims_its_text_not_its_children() {
    let html = render(full_app);
    let caption = first_class(&html, "figcaption");
    let rule = format!(".{caption}{{");
    let start = html.find(&rule).expect("a caption rule") + rule.len();
    let body = &html[start..start + html[start..].find('}').expect("a closed rule")];

    assert!(
        !body.contains(";opacity:") && !body.starts_with("opacity:"),
        "{body}"
    );
    assert!(
        body.contains(
            "color:color-mix(in srgb, currentColor calc(var(--lsx-blockquote-cite-opacity) * 100%), transparent)"
        ),
        "{body}"
    );
}

/// The UA stylesheet gives *both* `<figure>` and `<blockquote>` a 40px inline
/// margin, and the plan only mentions the blockquote's. Scoped to the two
/// component classes rather than counting `{margin:0;` across the sheet - the
/// global `body` reset matches that too, which is how this assertion was wrong
/// the first time.
#[test]
fn both_ua_margins_are_reset() {
    let html = render(bare_app);
    let figure = first_class(&html, "figure");
    let quote = first_class(&html, "blockquote");

    assert!(
        html.contains(&format!(".{figure}{{margin:0;}}")),
        "no figure reset"
    );
    assert!(
        html.contains(&format!(".{quote}{{margin:0;")),
        "no blockquote reset"
    );
}

/// The framework class an element carries, which every rule for it hangs off.
fn first_class(html: &str, tag: &str) -> String {
    attributes_of(html, tag)["class"]
        .split_whitespace()
        .next()
        .expect("a framework class")
        .to_string()
}
