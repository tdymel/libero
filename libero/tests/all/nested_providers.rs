//! A `LiberoProvider` inside another one gives its own subtree its own
//! localization and formats; the outer subtree keeps its own.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    hooks::{use_formats, use_localization},
    localization::{Formats, Localization},
    theme::{HexColor, Theme},
};

#[component]
fn Reading(id: &'static str) -> Element {
    let close = use_localization().common.close;
    let decimal = use_formats().decimal_separator;
    rsx! {
        p { id, "{close}|{decimal}" }
    }
}

#[test]
fn an_inner_provider_localizes_only_its_subtree() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Reading { id: "outer" }
                LiberoProvider { localization: &Localization::GERMAN, formats: &Formats::GERMAN,
                    Reading { id: "inner" }
                }
                Reading { id: "after" }
            }
        }
    }
    let html = body(&render(app));
    let english = format!(
        "{}|{}",
        Localization::ENGLISH.common.close,
        Formats::AMERICAN.decimal_separator
    );
    let german = format!(
        "{}|{}",
        Localization::GERMAN.common.close,
        Formats::GERMAN.decimal_separator
    );
    assert!(html.contains(&format!(r#"id="outer">{english}"#)), "{html}");
    assert!(html.contains(&format!(r#"id="inner">{german}"#)), "{html}");
    assert!(html.contains(&format!(r#"id="after">{english}"#)), "{html}");
}

/// The outer sheet already holds the same text; other themes still get their own (todo 2207).
#[test]
fn an_inner_provider_writes_its_theme_sheet_only_when_it_differs() {
    static TEAL: Theme = Theme {
        primary: HexColor::new(0x0B7285),
        ..Theme::DEFAULT
    };
    fn single() -> Element {
        rsx! { LiberoProvider { "x" } }
    }
    fn same() -> Element {
        rsx! { LiberoProvider { LiberoProvider { "x" } } }
    }
    fn other() -> Element {
        rsx! { LiberoProvider { LiberoProvider { themes: &TEAL, "x" } } }
    }
    let sheets = |app: fn() -> Element| render(app).matches(":root{").count();
    assert_eq!(sheets(same), sheets(single));
    assert!(sheets(other) > sheets(single));
}
