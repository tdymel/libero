use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::CodeBlock};

#[test]
fn code_block_renders_its_source_in_a_pre() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { source: "let x = 1;" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<pre"));
    assert!(body(&html).contains("let x = 1;"));
}

/// The copy confirmation is spoken through a status region that exists before
/// the copy, and a box nobody has measured as overflowing is no tab stop.
#[test]
fn code_block_mounts_an_empty_copy_status_and_no_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { source: "let x = 1;" }
            }
        }
    }

    let html = render(app);
    let status = &html[html.find(r#"role="status""#).expect("a status region")..];
    let status = &status[..status.find("</span>").unwrap()];
    assert!(!status.contains("Copied"), "{status}");
    assert!(!html.contains("tabindex=\"0\""), "{html}");
    assert!(!html.contains(r#"role="region""#), "{html}");
}

static BARE: libero::theme::Theme = libero::theme::Theme {
    code_block: libero::theme::CodeBlockDefaults {
        header: false,
        copyable: false,
        line_numbers: false,
        ..libero::theme::Theme::DEFAULT.code_block
    },
    ..libero::theme::Theme::DEFAULT
};

/// `header` and `copyable` are house style: unset, each takes the theme's,
/// and a call site still wins over it. `line_numbers` goes the same way, but
/// the gutter is drawn only once highlighting resolves, which SSR never sees.
#[test]
fn unset_chrome_props_follow_the_theme() {
    fn default() -> Element {
        rsx! {
            LiberoProvider { CodeBlock { source: "let x = 1;", language: "text" } }
        }
    }
    fn bare() -> Element {
        rsx! {
            LiberoProvider { themes: &BARE, CodeBlock { source: "let x = 1;", language: "text" } }
        }
    }
    fn overridden() -> Element {
        rsx! {
            LiberoProvider { themes: &BARE, CodeBlock { source: "let x = 1;", copyable: true } }
        }
    }

    let html = body(&render(default));
    assert!(html.contains("<button"), "{html}");
    // The header's text, not the group's `aria-label`.
    assert!(html.contains(">Plain text<"), "{html}");

    let html = body(&render(bare));
    assert!(!html.contains("<button"), "{html}");
    assert!(!html.contains(">Plain text<"), "{html}");

    assert!(body(&render(overridden)).contains("<button"));
}

/// Todo 826. No language, no label in the header; the copy button stays.
#[test]
fn a_block_without_a_language_shows_no_language_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { CodeBlock { source: "a -> b" } }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("<button"), "{html}");
    assert!(!html.contains("Unrecognized language"), "{html}");
    assert!(
        html.contains(r#"<span data-slot="language"></span>"#),
        "{html}"
    );
}

/// Todo 1252. A header with no language and no copy button would be an empty bar.
#[test]
fn a_header_with_nothing_to_show_is_dropped() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { CodeBlock { source: "a -> b", header: true, copyable: false } }
        }
    }

    let html = body(&render(app));
    assert!(!html.contains(r#"data-slot="header""#), "{html}");
}

/// Todo 668. No grammar still draws the diff's marker and spoken word, on the
/// first render: there is no highlighting to wait for.
#[test]
fn a_diff_without_a_language_still_marks_its_lines() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { diff: true, source: "keep\n-old\n+new" }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("Removed "), "{html}");
    assert!(html.contains("Added "), "{html}");
    assert!(!html.contains("-old") && !html.contains("+new"), "{html}");
}

/// Todo 668. An unknown language renders plain rows rather than one plain
/// `pre` once a line is to be highlighted.
#[test]
fn highlight_lines_without_a_known_language_still_marks_the_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { language: "no-such-language", highlight_lines: "2", source: "a\nb" }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("highlighted"), "{html}");
}

/// Todo 1541. With no header the copy button floats over the first row, so only
/// that row reserves its width; a header holds the button and reserves nothing.
#[test]
fn a_floating_copy_button_reserves_space_on_the_first_row_only() {
    fn floating() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { header: false, highlight_lines: "2", source: "first\nsecond" }
            }
        }
    }
    fn headed() -> Element {
        rsx! {
            LiberoProvider {
                CodeBlock { header: true, highlight_lines: "2", source: "first\nsecond" }
            }
        }
    }

    let html = body(&render(floating));
    assert_eq!(html.matches("copy-space").count(), 1, "{html}");
    assert!(html.find("copy-space") < html.find(">first<"), "{html}");
    assert!(!body(&render(headed)).contains("copy-space"));
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
/// No clipboard without `native`; `native` has arboard, which may write, and the in-crate tests fake both answers.
#[cfg(not(feature = "native"))]
mod dispatched {

    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{LiberoProvider, components::CodeBlock};

    /// No clipboard, so the copy fails and the status says so.
    #[test]
    fn a_copy_without_a_clipboard_is_announced_as_failed() {
        fn app() -> Element {
            rsx! { LiberoProvider { CodeBlock { source: "let x = 1;" } } }
        }

        let (before, after) = click_the_last_listener(app);
        let status = |html: &str| {
            let status = &html[html.find(r#"role="status""#).expect("a status region")..];
            status[..status.find("</span>").unwrap()].to_string()
        };
        assert!(!status(&before).contains("Copy failed"), "{before}");
        assert!(status(&after).contains("Copy failed"), "{after}");
    }
}
