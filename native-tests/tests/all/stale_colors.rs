//! Colours and states the Maintainer saw go stale natively (todos 621, 624,
//! 628, 651): an active `NavLink`'s text after a theme switch, `Select`'s clear
//! x and every field's trailing icon on the dark theme, `CodeBlock`'s copy
//! check after Tab moves away.
//!
//! One mount per test: a second mount on the same thread does not flip the
//! root attribute on a toggle.

use std::{rc::Rc, time::Duration};

use dioxus::prelude::*;
use libero::{
    components::{
        Autocomplete, Box, Cascader, CascaderOption, CodeBlock, MultiSelect, NavLink, NumberField,
        Options, PasswordField, Select, States, TagsField, TextField, Tree, TreeNode,
        TreeNodeRenderArgs,
    },
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
    // "Copy failed" without a display server (CI): either status proves the click landed.
    let status = page.text("[role=status]");
    assert!(
        ["Copied", "Copy failed"].contains(&status.as_str()),
        "status {status:?}"
    );

    page.tab();
    assert!(!page.is_focused(COPY), "Tab stayed on the copy button");
    assert_eq!(page.text("[role=status]"), "");
}

/// Every field with a trailing icon, each holding a value so the x shows.
fn fields_app() -> Element {
    let mut quantity = use_signal(|| Some(3i32));
    let mut fruit = use_signal(|| Some(Fruit::Banana));
    let mut fruits = use_signal(|| vec![Fruit::Apple]);
    let mut none = use_signal(|| None::<Fruit>);
    let mut city = use_signal(|| "B".to_string());
    let mut topics = use_signal(|| vec!["rust".to_string()]);
    let mut place = use_signal(|| Some("paris".to_string()));
    rsx! {
        Select { label: "Empty", value: none(), onchange: move |next| none.set(next) }
        Autocomplete {
            label: "City",
            clearable: true,
            options: ["Berlin", "Bern", "Bonn"].map(String::from).to_vec(),
            value: city(),
            oninput: move |next| city.set(next),
        }
        Cascader {
            label: "Place",
            clearable: true,
            data: vec![
                CascaderOption::new("france", "France")
                    .children(vec![CascaderOption::new("paris", "Paris")]),
            ],
            value: place(),
            onchange: move |next: Option<String>| place.set(next),
        }
        TagsField {
            label: "Topics",
            clearable: true,
            value: topics(),
            onchange: move |next| topics.set(next),
        }
        TextField { label: "Text", value: "hello", oninput: move |_| {} }
        PasswordField { label: "Password", value: "secret" }
        NumberField {
            label: "Number",
            steppers: true,
            value: quantity(),
            onchange: move |next| quantity.set(next),
        }
        Select {
            label: "Fruit",
            clearable: true,
            value: fruit(),
            onchange: move |next| fruit.set(next),
        }
        MultiSelect {
            label: "Fruits",
            clearable: true,
            value: fruits(),
            onchange: move |next| fruits.set(next),
        }
    }
}

/// WCAG's relative luminance of a computed `rgb(..)`.
fn luminance(color: &str) -> f64 {
    let channels: Vec<f64> = color
        .trim_start_matches("rgb(")
        .trim_end_matches(')')
        .split(", ")
        .map(|channel| {
            let c = channel.parse::<f64>().unwrap() / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
}

fn contrast(a: &str, b: &str) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Each trailing icon paints the colour it computes, 3:1 on the frame
/// (WCAG 1.4.11).
fn assert_trailing_icons_read(page: &Page) {
    let frame = page.computed("[data-frame]", "background-color");
    let icons = page.query_all("[data-frame] svg");
    assert!(
        icons.len() >= 9,
        "only {} icons\n{}",
        icons.len(),
        page.tree()
    );
    for icon in icons {
        let color = page.computed_of(icon, "color");
        let name = page.describe(
            page.doc
                .inner
                .borrow()
                .get_node(icon)
                .unwrap()
                .parent
                .unwrap(),
        );
        assert_eq!(
            page.painted_stroke_of(icon),
            color,
            "{name} bakes a stale colour"
        );
        // Chip x's sit on the chip's fill, not the frame.
        if name.contains("Remove") {
            continue;
        }
        let ratio = contrast(&color, &frame);
        assert!(ratio >= 3.0, "{name}: {color} on {frame} is {ratio:.2}:1");
    }
}

/// Todo 651.
#[test]
fn field_trailing_icons_read_on_a_dark_window() {
    let page = mount_in(fields_app, ColorScheme::Dark);
    assert_trailing_icons_read(&page);
}

/// A flex row whose text and icon change colour on hover.
fn hover_app() -> Element {
    rsx! {
        Box {
            id: "row",
            sx: sx().display("flex")
                .gap("8px")
                .padding("8px")
                .color("rgb(0, 0, 0)")
                .hover(sx().color("rgb(200, 0, 0)")),
            svg { id: "icon", width: "16", height: "16", view_box: "0 0 16 16",
                // No fill: `painted_stroke` reads the first colour in the tree.
                path { d: "M2 8h12", fill: "none", stroke: "currentColor", stroke_width: "4" }
            }
            "Label"
        }
        div { id: "away", height: "40px", "Away" }
    }
}

/// Todo 634: the anonymous block and the svg rebuild with a hover, not only
/// with a theme switch.
#[test]
fn a_hover_repaints_a_flex_rows_text_and_icon() {
    let mut page = mount(hover_app);
    page.hover("#row");
    let hovered = page.computed("#row", "color");
    assert_eq!(hovered, "rgb(200, 0, 0)", "the hover did not restyle");
    assert_eq!(page.painted_text("#row"), hovered);
    assert_eq!(page.painted_stroke("#icon"), hovered);

    page.hover("#away");
    let left = page.computed("#row", "color");
    assert_eq!(left, "rgb(0, 0, 0)");
    assert_eq!(page.painted_text("#row"), left);
    assert_eq!(page.painted_stroke("#icon"), left);
}

/// Flex rows whose text and icon change colour with a state other than hover:
/// focus, a sibling's `:checked`, an attribute a click sets on the row.
fn state_app() -> Element {
    let mut on = use_signal(|| false);
    let icon = rsx! {
        svg { width: "16", height: "16", view_box: "0 0 16 16",
            path { d: "M2 8h12", fill: "none", stroke: "currentColor", stroke_width: "4" }
        }
    };
    rsx! {
        style {
            "#focus, #checked, #marked {{ display: flex; gap: 8px; color: rgb(0, 0, 0); }}
            #focus:focus, #box:checked + #checked, #marked[data-on=true] {{ color: rgb(200, 0, 0); }}"
        }
        button { id: "before", "Before" }
        div { id: "focus", tabindex: "0", {icon.clone()} "Focus" }
        button { id: "after", "After" }
        input { id: "box", r#type: "checkbox" }
        span { id: "checked", {icon.clone()} "Checked" }
        button { id: "toggle", onclick: move |_| on.toggle(), "Toggle" }
        div { id: "marked", "data-on": "{on}", {icon} "Marked" }
    }
}

const RED: &str = "rgb(200, 0, 0)";
const BLACK: &str = "rgb(0, 0, 0)";

#[track_caller]
fn assert_painted(page: &mut Page, row: &str, color: &str) {
    // The rebuild follows the frame that painted the change.
    page.wait(Duration::from_millis(20));
    assert_eq!(page.computed(row, "color"), color, "{row} did not restyle");
    assert_eq!(page.painted_text(row), color, "{row}'s text");
    assert_eq!(
        page.painted_stroke(&format!("{row} svg")),
        color,
        "{row}'s icon"
    );
}

/// Todo 834: Tab onto and off a row with a `:focus` colour.
#[test]
fn a_focus_by_tab_repaints_a_flex_rows_text_and_icon() {
    let mut page = mount(state_app);
    page.focus("#before");
    page.tab();
    assert!(
        page.is_focused("#focus"),
        "Tab went to {}",
        page.focus_owner()
    );
    assert_painted(&mut page, "#focus", RED);
    page.tab();
    assert_painted(&mut page, "#focus", BLACK);
}

#[test]
fn a_focus_by_click_repaints_a_flex_rows_text_and_icon() {
    let mut page = mount(state_app);
    page.click("#focus");
    assert!(
        page.is_focused("#focus"),
        "the click focused {}",
        page.focus_owner()
    );
    assert_painted(&mut page, "#focus", RED);
    page.click("#after");
    assert_painted(&mut page, "#focus", BLACK);
}

#[test]
fn a_checked_sibling_repaints_a_flex_rows_text_and_icon() {
    let mut page = mount(state_app);
    page.click("#box");
    assert_painted(&mut page, "#checked", RED);
    page.click("#box");
    assert_painted(&mut page, "#checked", BLACK);
}

#[test]
fn an_attribute_change_repaints_a_flex_rows_text_and_icon() {
    let mut page = mount(state_app);
    page.click("#toggle");
    assert_painted(&mut page, "#marked", RED);
    page.click("#toggle");
    assert_painted(&mut page, "#marked", BLACK);
}

/// Past the first paint on a loaded machine.
const TIMER: Duration = Duration::from_millis(500);

/// The flex row of [`state_app`], turned red by a timer: no press or key.
fn timer_app() -> Element {
    let mut on = use_signal(|| false);
    use_hook(move || {
        libero::platform::timer()
            .map(|timer| Rc::new(timer.after(TIMER, Box::new(move || on.set(true)))))
    });
    rsx! {
        style {
            "#marked {{ display: flex; gap: 8px; color: rgb(0, 0, 0); }}
            #marked[data-on=true] {{ color: rgb(200, 0, 0); }}"
        }
        div { id: "marked", "data-on": "{on}",
            svg { width: "16", height: "16", view_box: "0 0 16 16",
                path { d: "M2 8h12", fill: "none", stroke: "currentColor", stroke_width: "4" }
            }
            "Marked"
        }
    }
}

/// Todo 872: a render no input preceded repaints the row's text and icon.
#[test]
fn a_timers_colour_change_repaints_a_flex_rows_text_and_icon() {
    let mut page = mount(timer_app);
    assert_painted(&mut page, "#marked", BLACK);
    page.wait(TIMER);
    assert_painted(&mut page, "#marked", RED);
}

/// libero's hidden flush elements, which a flush replaces.
fn flush_elements(page: &Page) -> Vec<String> {
    let doc = page.doc.inner.borrow();
    let divs = doc.query_selector_all("div").unwrap_or_default();
    divs.into_iter()
        .filter(|&id| {
            let element = doc.get_node(id).and_then(|node| node.element_data());
            element
                .and_then(|element| element.attr(blitz_dom::local_name!("style")))
                .is_some_and(|style| style.contains("none"))
        })
        .map(|id| format!("{id:?}"))
        .collect()
}

/// Todo 872's loop guards: the redraw a flush or a baked-box rebuild asks for
/// arms no flush of its own, so after a render the document comes to rest.
#[test]
fn the_document_comes_to_rest_after_a_timers_render() {
    let mut page = mount(timer_app);
    page.wait(TIMER + Duration::from_millis(100));
    assert_painted(&mut page, "#marked", RED);
    let before = flush_elements(&page);
    assert!(!before.is_empty(), "no flush element in\n{}", page.tree());
    for _ in 0..5 {
        page.wait(Duration::from_millis(40));
        assert_eq!(
            flush_elements(&page),
            before,
            "libero kept flushing at rest"
        );
    }
}

/// Todo 882: the raster paints inline svgs (`blitz-paint`'s `svg` feature).
#[test]
fn an_inline_svg_paints_its_stroke() {
    let mut page = mount(timer_app);
    page.wait(Duration::from_millis(100));
    let (x, y, width, height) = page.rect("#marked svg");
    let centre = page.painted_pixel((x + width / 2.0) as u32, (y + height / 2.0) as u32);
    assert_eq!(centre, BLACK, "the icon's stroke at its centre");
}

/// Todo 893: the baked rebuild a check makes asks for a redraw that arms no
/// check of its own (`redraw::quiet`). Unguarded, each check found an
/// anonymous block stale again before its restyle and rebuilt it for ever.
#[test]
fn a_baked_rebuild_alone_arms_no_check() {
    let mut page = mount(timer_app);
    page.wait(TIMER + Duration::from_millis(100));
    assert_painted(&mut page, "#marked", RED);
    let before = page.redraws();
    for _ in 0..5 {
        page.wait(Duration::from_millis(40));
    }
    assert_eq!(page.redraws(), before, "libero kept redrawing at rest");
}

#[test]
fn field_trailing_icons_read_after_a_live_switch_to_dark() {
    let mut page = mount(fields_app);
    let light = page.computed("[data-frame]", "background-color");
    page.set_color_scheme(ColorScheme::Dark);
    page.wait(TICK);
    assert_ne!(
        page.computed("[data-frame]", "background-color"),
        light,
        "the scheme did not switch"
    );
    assert_trailing_icons_read(&page);
}
