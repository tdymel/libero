//! `Pagination` as a server renders it: the landmark, the accessible names, and
//! which control carries `aria-current`.
//!
//! The range arithmetic has its own unit tests beside the function; this is the
//! markup contract that four of the plan's five a11y claims live in, and the
//! part a caller re-derives and gets wrong.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Pagination};

fn markup(app: fn() -> Element) -> String {
    body(&render(app))
}

#[test]
fn it_is_a_named_landmark_around_a_list() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Pagination { total: 10, page: 4, aria_label: "Search results", onchange: |_| {} }
            }
        }
    }

    let html = markup(app);
    assert!(html.contains("<nav"), "expected a nav landmark: {html}");
    assert!(
        html.contains(r#"aria-label="Search results""#),
        "the nav has to be nameable - two paginations on one page must differ: {html}"
    );
    assert!(html.contains("<ul"), "expected a list: {html}");
    assert!(html.contains("<li"), "expected list items: {html}");
}

/// MUI's asymmetry. `aria-current` already says "current", so repeating "go to"
/// on the page the reader is on would be a lie.
#[test]
fn the_current_page_is_named_differently_from_the_rest() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Pagination { total: 10, page: 4, aria_label: "Pages", onchange: |_| {} }
            }
        }
    }

    let html = markup(app);
    assert!(html.contains(r#"aria-label="Page 4""#), "{html}");
    assert!(html.contains(r#"aria-label="Go to page 3""#), "{html}");
    assert_eq!(
        html.matches(r#"aria-current="page""#).count(),
        1,
        "exactly one control is current, and never aria-current=\"true\": {html}"
    );
}

/// Not focusable, not announced: it is a gap, not a control.
#[test]
fn the_ellipsis_is_hidden_from_the_accessibility_tree() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Pagination { total: 40, page: 20, aria_label: "Pages", onchange: |_| {} }
            }
        }
    }

    let html = markup(app);
    assert!(
        html.contains("…"),
        "expected an ellipsis at total=40: {html}"
    );
    // The `<li>` carries it, not just the span inside. Hiding only the text
    // silences the `…` and leaves an empty list item in the tree, so the list
    // announces more entries than it has - and the old assertion passed either
    // way, which is why this names the element.
    assert!(
        html.contains(r#"<li aria-hidden="true""#),
        "the gap itself must be out of the accessibility tree: {html}"
    );
}

#[test]
fn controls_are_opt_in_and_named() {
    fn with_edges() -> Element {
        rsx! {
            LiberoProvider {
                Pagination {
                    total: 10,
                    page: 5,
                    aria_label: "Pages",
                    with_edges: true,
                    onchange: |_| {},
                }
            }
        }
    }

    fn bare() -> Element {
        rsx! {
            LiberoProvider {
                Pagination {
                    total: 10,
                    page: 5,
                    aria_label: "Pages",
                    with_controls: false,
                    onchange: |_| {},
                }
            }
        }
    }

    let edges = markup(with_edges);
    for name in [
        "Go to first page",
        "Go to previous page",
        "Go to next page",
        "Go to last page",
    ] {
        assert!(edges.contains(name), "missing {name}: {edges}");
    }

    let bare = markup(bare);
    assert!(!bare.contains("Go to previous page"), "{bare}");
    assert!(!bare.contains("Go to first page"), "{bare}");
}

/// An empty result set has no pages, and a lone control implies otherwise.
#[test]
fn a_total_of_zero_renders_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Pagination { total: 0, page: 1, aria_label: "Pages", onchange: |_| {} }
            }
        }
    }

    assert!(!markup(app).contains("<nav"), "{}", markup(app));
}

/// The label closure sees the five *named* controls and never the ellipsis, so
/// an exhaustive match has no dead branch. It also has to be able to put the
/// number somewhere other than last, which is the whole reason it exists.
#[test]
fn the_label_closure_renames_every_control() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Pagination {
                    total: 10,
                    page: 4,
                    aria_label: "Pages",
                    with_edges: true,
                    onchange: |_| {},
                    label: |label: libero::components::PaginationLabel| {
                        use libero::components::PaginationLabel::*;
                        match label {
                            Page { number, current: true } => format!("Seite {number}, aktuell"),
                            Page { number, .. } => format!("Seite {number}"),
                            First => "Erste".to_string(),
                            Previous => "Zurück".to_string(),
                            Next => "Weiter".to_string(),
                            Last => "Letzte".to_string(),
                        }
                    },
                }
            }
        }
    }

    let html = markup(app);
    for name in [
        "Seite 4, aktuell",
        "Seite 3",
        "Erste",
        "Zurück",
        "Weiter",
        "Letzte",
    ] {
        assert!(html.contains(name), "missing {name}: {html}");
    }
    assert!(
        !html.contains("Go to page"),
        "the override must be total: {html}"
    );
}

/// Page 1 disables what points backwards; the last page disables what points
/// forwards. The current page's own button stays enabled, which is what gives
/// the focus repair somewhere to land.
#[test]
fn the_end_controls_disable_and_the_current_page_does_not() {
    fn first_page() -> Element {
        rsx! {
            LiberoProvider {
                Pagination {
                    total: 10,
                    page: 1,
                    aria_label: "Pages",
                    with_edges: true,
                    onchange: |_| {},
                }
            }
        }
    }

    let html = markup(first_page);
    let previous = control(&html, "Go to previous page");
    let first = control(&html, "Go to first page");
    let next = control(&html, "Go to next page");
    let current = control(&html, "Page 1");

    assert!(
        previous.contains("disabled"),
        "previous on page 1: {previous}"
    );
    assert!(first.contains("disabled"), "first on page 1: {first}");
    assert!(!next.contains("disabled"), "next on page 1: {next}");
    assert!(
        !current.contains("disabled"),
        "the current page stays enabled so focus has somewhere to land: {current}"
    );
}

/// The list's `<li>`s are flex containers, which is what keeps a digit level
/// with an arrow: a `list-item` `<li>` puts its `inline-flex` control in a line
/// box, and an `<svg>`-only control's synthesized baseline makes that line box
/// a strut-descender taller than a digit control's.
///
/// **SSR cannot see the alignment itself.** There is no layout here and no
/// computed rect - this asserts only that the rule reaches the stylesheet. The
/// offset it removes (2.5px at `md`, 3.75px at `xs`) was measured in Chromium
/// and lives in the comment on `PAGINATION_LIST_SX`.
#[test]
fn the_list_items_are_flex_containers_rather_than_list_items() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Pagination { total: 10, page: 1, aria_label: "Pages", onchange: |_| {} }
            }
        }
    }

    let html = render(app);
    let rule = html
        .split('}')
        .find(|rule| rule.contains(">li") || rule.contains("> li"))
        .unwrap_or_else(|| panic!("no `> li` rule in the emitted CSS:\n{html}"));

    assert!(
        rule.contains("display:flex"),
        "the `> li` rule must make the list items flex containers: {rule}"
    );
}

/// The whole opening tag carrying `aria-label="{name}"`, so a test can assert
/// on one control rather than on the document.
fn control(html: &str, name: &str) -> String {
    let needle = format!("aria-label=\"{name}\"");
    let at = html
        .find(&needle)
        .unwrap_or_else(|| panic!("no control named {name} in:\n{html}"));
    let start = html[..at].rfind('<').expect("an opening tag");
    let end = at + html[at..].find('>').expect("an unterminated tag");
    html[start..=end].to_string()
}
