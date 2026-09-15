//! `Alert`'s rendered contract: a `role` the caller can replace, `aria-*`
//! wiring that names only what exists, and a close button that is there only
//! when something handles it.

use crate::common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Alert};

/// The alert's own framework class - the first one on its root `<div>`.
fn alert_class(html: &str) -> String {
    classes_of(&body(html), "div")
        .first()
        .expect("a class on the alert")
        .clone()
}

/// The declarations of the first rule for `selector`, up to its `}`.
fn rule<'a>(html: &'a str, selector: &str) -> &'a str {
    let start = html
        .find(&format!("{selector}{{"))
        .unwrap_or_else(|| panic!("no rule for {selector}:\n{html}"));
    let rule = &html[start..];
    &rule[..rule.find('}').expect("an unterminated rule")]
}

/// Only error and warning interrupt; the default `info`, the rest of the
/// palette and a literal colour are a polite `status`.
#[test]
fn the_role_follows_the_colour() {
    fn role_of(app: fn() -> Element) -> Option<String> {
        attributes_of(&body(&render(app)), "div").remove("role")
    }
    fn default() -> Element {
        rsx! { LiberoProvider { Alert { "Saved." } } }
    }
    fn error() -> Element {
        rsx! { LiberoProvider { Alert { color: "error", "Failed." } } }
    }
    fn warning_shade() -> Element {
        rsx! { LiberoProvider { Alert { color: "warning.7", "Expiring." } } }
    }
    fn success() -> Element {
        rsx! { LiberoProvider { Alert { color: "success", "Saved." } } }
    }
    fn literal() -> Element {
        rsx! { LiberoProvider { Alert { color: "#c00", "Saved." } } }
    }

    assert_eq!(role_of(default).as_deref(), Some("status"));
    assert_eq!(role_of(error).as_deref(), Some("alert"));
    assert_eq!(role_of(warning_shade).as_deref(), Some("alert"));
    assert_eq!(role_of(success).as_deref(), Some("status"));
    assert_eq!(role_of(literal).as_deref(), Some("status"));
}

/// The `attr_default` contract, and the thing `Form`'s summary depends on. A
/// plain `.attr` would render two `role`s, and which one survives differs
/// between SSR and the DOM.
#[test]
fn a_callers_role_replaces_the_default_and_appears_once() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Alert { role: "log", "Saved." } }
        }
    }

    let html = body(&render(app));
    let root = &html[html.find("<div").unwrap()..];
    let open_tag = &root[..root.find('>').unwrap()];

    assert_eq!(open_tag.matches("role=").count(), 1, "{open_tag}");
    assert!(open_tag.contains(r#"role="log""#), "{open_tag}");
}

#[test]
fn it_adopts_a_callers_id_and_derives_the_aria_ids_from_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Alert { id: "card", title: "Card expiring", "Update it." }
            }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "div");

    assert_eq!(attributes.get("id").map(String::as_str), Some("card"));
    assert_eq!(html.matches(r#"id="card""#).count(), 1, "{html}");
    assert_eq!(
        attributes.get("aria-labelledby").map(String::as_str),
        Some("card-title")
    );
    assert_eq!(
        attributes.get("aria-describedby").map(String::as_str),
        Some("card-body")
    );
    assert!(html.contains(r#"id="card-title""#), "{html}");
    assert!(html.contains(r#"id="card-body""#), "{html}");
}

/// An `aria-labelledby` pointing at nothing is worse than none: the region
/// loses its name rather than falling back to its content.
#[test]
fn it_names_only_what_exists() {
    fn untitled() -> Element {
        rsx! {
            LiberoProvider { Alert { "Saved." } }
        }
    }
    fn wordless() -> Element {
        rsx! {
            LiberoProvider { Alert { title: "Saved" } }
        }
    }

    let html = body(&render(untitled));
    let attributes = attributes_of(&html, "div");
    assert!(
        !attributes.contains_key("aria-labelledby"),
        "{attributes:?}"
    );
    assert!(!html.contains("data-slot=\"title\""), "{html}");

    let html = body(&render(wordless));
    let attributes = attributes_of(&html, "div");
    assert!(
        !attributes.contains_key("aria-describedby"),
        "{attributes:?}"
    );
    assert!(!html.contains("data-slot=\"message\""), "{html}");
}

#[test]
fn the_close_button_is_there_only_with_an_onclose() {
    fn static_alert() -> Element {
        rsx! {
            LiberoProvider { Alert { "Saved." } }
        }
    }
    fn closable() -> Element {
        rsx! {
            LiberoProvider { Alert { onclose: move |_| {}, "Saved." } }
        }
    }

    assert!(!body(&render(static_alert)).contains("<button"));

    let html = body(&render(closable));
    assert_eq!(html.matches("<button").count(), 1, "{html}");
    let button = attributes_of(&html, "button");
    assert_eq!(button.get("aria-label").map(String::as_str), Some("Close"));
}

/// An alert rests in the flow. `paper_sx()` brings a shadow, and the chained
/// `none` has to replace it rather than race it in the cascade. Losing
/// `paper_sx()`'s focus-contrast declaration would take the close button's
/// focus ring with it.
#[test]
fn the_base_rule_has_one_shadow_and_keeps_the_focus_contrast() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Alert { "Saved." } }
        }
    }

    let html = render(app);
    let class = alert_class(&html);
    let base = rule(&html, &format!(".{class}"));

    assert_eq!(base.matches("box-shadow:").count(), 1, "{base}");
    assert!(base.contains("box-shadow:none"), "{base}");
    assert!(base.contains("--lsx-focus-contrast:"), "{base}");
}

/// A static surface, `Badge`'s rule: a tint that moved under the pointer
/// would claim the alert is a target.
#[test]
fn the_variant_chrome_has_no_hover() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Alert { "Saved." } }
        }
    }

    let html = render(app);
    let class = alert_class(&html);

    assert!(
        body(&html).contains(r#"data-state="tonal""#),
        "{}",
        body(&html)
    );
    assert!(!html.contains(&format!(".{class}[data-state~=\"tonal\"]:hover")));
    assert!(!html.contains(&format!(".{class}:hover")));
}

/// `paper_sx()` publishes a black ring, which reads on the tint and on the
/// page but not on a solid `filled` ground. There a child's ring takes the
/// text colour, and a literal colour - which has no `-contrast` twin - falls
/// back to `currentColor` rather than to an unset var, which would erase the
/// ring. The alert's own ring keeps the black: it is drawn outside, on the
/// page, where a white text colour measured 1:1.
#[test]
fn a_filled_alert_rings_its_children_in_its_text_colour() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Alert { variant: "filled", "Saved." } }
        }
    }

    let html = render(app);
    let class = alert_class(&html);
    let children = rule(&html, &format!(".{class}[data-state~=\"filled\"] > *"));
    let own = rule(&html, &format!(".{class}[data-state~=\"filled\"]"));

    assert!(
        children.contains("--lsx-focus-contrast:var(--lsx-alert-contrast, currentColor)"),
        "{children}"
    );
    assert!(!own.contains("--lsx-focus-contrast:"), "{own}");
}
