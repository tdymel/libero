//! Colours and states the Maintainer saw go stale natively (todos 621, 624,
//! 628): an active `NavLink`'s text after a theme switch, `Select`'s clear x on
//! the dark theme, `CodeBlock`'s copy check after Tab moves away.
//!
//! One mount per test: a second mount on the same thread does not flip the
//! root attribute on a toggle.

use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::{CodeBlock, NavLink, Options, Select, States, Tree, TreeNode, TreeNodeRenderArgs},
    hooks::use_color_scheme,
    sx::sx,
    theme::{ColorCss, ColorShade},
};
use native_tests::{ColorScheme, Page, mount, mount_in};

// libero's scheme re-read interval, and a margin for a loaded machine.
const TICK: Duration = Duration::from_millis(800);

const LINK: &str = "a[aria-current=page]";

fn nav_app() -> Element {
    let scheme = use_color_scheme();
    let data = vec![TreeNode::new("group", "Group").children(vec![
        TreeNode::new("/here", "Here"),
        TreeNode::new("/elsewhere", "Elsewhere"),
    ])];
    rsx! {
        button { id: "toggle", onclick: move |_| scheme.toggle(), "Toggle" }
        Tree {
            aria_label: "Pages",
            data,
            default_expanded: ["group".to_string()].into_iter().collect(),
            render_node: move |args: TreeNodeRenderArgs<&'static str>| {
                if args.expanded.is_some() {
                    return rsx! { span { "{args.data}" } };
                }
                // The docs sidebar's leaf look.
                let muted = ColorCss::MUTED.value(ColorShade::S3);
                let primary = ColorCss::PRIMARY.value(ColorShade::S6);
                rsx! {
                    NavLink {
                        to: args.id.clone(),
                        active: args.id == "/here",
                        tabindex: args.tabindex,
                        states: States::new().with("leaf", true),
                        sx: sx().align_self("stretch").when(
                            "leaf",
                            sx().border_radius("0")
                                .border_left(format!("2px solid {muted}"))
                                .when("active", sx().border_left(format!("2px solid {primary}"))),
                        ),
                        "{args.data}"
                    }
                }
            },
        }
    }
}

/// The link's text sits in an anonymous block (the link is `flex`), whose
/// style Blitz kept from the box build.
fn assert_text_painted_as_computed(page: &Page) {
    assert_eq!(page.painted_text(LINK), page.computed(LINK, "color"));
}

#[test]
fn an_active_nav_link_follows_a_scheme_toggle() {
    let mut page = mount(nav_app);
    let light = page.computed(LINK, "color");
    assert_text_painted_as_computed(&page);

    page.click("#toggle");
    assert_eq!(page.attr("html", "data-lsx-theme").as_deref(), Some("dark"));
    assert_ne!(
        page.computed(LINK, "color"),
        light,
        "the scheme did not switch"
    );
    assert_text_painted_as_computed(&page);
}

#[test]
fn an_active_nav_link_follows_the_window_scheme() {
    let mut page = mount(nav_app);
    let light = page.computed(LINK, "color");
    page.set_color_scheme(ColorScheme::Dark);
    page.wait(TICK);
    assert_ne!(
        page.computed(LINK, "color"),
        light,
        "the scheme did not switch"
    );
    assert_text_painted_as_computed(&page);
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Fruit {
    Apple,
    Banana,
}

fn select_app() -> Element {
    let scheme = use_color_scheme();
    let mut value = use_signal(|| None);
    rsx! {
        button { id: "toggle", onclick: move |_| scheme.toggle(), "Toggle" }
        button { id: "pick", onclick: move |_| value.set(Some(Fruit::Banana)), "Pick" }
        Select {
            label: "Fruit",
            clearable: true,
            value: value(),
            onchange: move |next| value.set(next),
        }
    }
}

const CLEAR: &str = "[aria-label=Clear] svg";

/// Blitz's UA sheet paints every `<button>` black on either scheme; the x
/// takes its slot's colour instead.
#[test]
fn the_clear_x_is_not_black_on_the_dark_theme() {
    let mut page = mount_in(select_app, ColorScheme::Dark);
    page.click("#pick");
    let color = page.computed(CLEAR, "color");
    assert_ne!(color, "rgb(0, 0, 0)");
    assert_eq!(page.painted_stroke(CLEAR), color);
}

#[test]
fn the_clear_x_follows_a_scheme_toggle() {
    let mut page = mount(select_app);
    page.click("#pick");
    let light = page.computed(CLEAR, "color");
    page.click("#toggle");
    let dark = page.computed(CLEAR, "color");
    assert_ne!(light, dark, "the scheme did not switch");
    assert_eq!(page.painted_stroke(CLEAR), dark);
}

fn code_app() -> Element {
    rsx! {
        CodeBlock { source: "let a = 1;", copyable: true }
        button { id: "after", "After" }
    }
}

const COPY: &str = "[aria-label='Copy code']";

/// Blitz's Tab fires no `blur`, the button's own reset.
#[test]
fn tab_away_from_the_copy_button_resets_it() {
    let mut page = mount(code_app);
    page.click(COPY);
    assert!(
        page.is_focused(COPY),
        "the click focused {}",
        page.focus_owner()
    );
    assert_eq!(page.text("[role=status]"), "Copied");

    page.tab();
    assert!(!page.is_focused(COPY), "Tab stayed on the copy button");
    assert_eq!(page.text("[role=status]"), "");
}
