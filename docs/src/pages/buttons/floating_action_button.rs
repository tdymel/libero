use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{
    Box, Code, FloatingActionButton, Input, Menu, MenuEntry, MenuItem, Pictogram, Text, use_menu,
};
use libero::hooks::{Align, Side};
use libero::sx::sx;
use libero::use_theme;
use pictogram_icons_lucide as lucide;

const ICON: &str = "icon: rsx! { Pictogram { icon: lucide::plus::outlined } }";
const LABEL: &str = "New message";

fn menu_items() -> Vec<MenuEntry> {
    vec![
        MenuItem::new("Message").onselect(|_| {}).into(),
        MenuItem::new("Event").onselect(|_| {}).into(),
    ]
}

/// What the `menu` switch prints around the button, as `render` draws it.
fn wrap(values: &DemoValues, source: &str) -> String {
    // A busy button stops a press, not the arrow keys, so the `Menu` is disabled with it.
    let disabled = match values.str("loading") == "true" {
        true => "        disabled: true,\n",
        false => "",
    };
    match values.str("menu") == "true" {
        true => format!(
            "let menu = use_menu();\n\nrsx! {{\n    Menu {{\n        state: menu,\n{disabled}        side: Side::Top,\n        align: Align::End,\n        items: vec![\n            MenuItem::new(\"Message\").onselect(|_| {{}}).into(),\n            MenuItem::new(\"Event\").onselect(|_| {{}}).into(),\n        ],\n{}\n    }}\n}}",
            indent(&indent(source))
        ),
        false => source.to_string(),
    }
}

fn extended(values: &DemoValues) -> bool {
    values.str("children") == "true"
}

/// The label is a child, not a prop, so a control decides it here.
fn text_child(values: &DemoValues) -> String {
    if extended(values) {
        LABEL.to_string()
    } else {
        String::new()
    }
}

fn color(values: &DemoValues) -> Input<libero::sx::ThemeAwareValue> {
    match values.str("color").as_str() {
        "primary" => Input::None,
        color => Input::from(color),
    }
}

fn controls(variant: &str, size: &str, radius: &str, placement: &str) -> Vec<Control> {
    vec![
        Control::toggle(
            "variant",
            ["filled", "tonal", "elevated", "outlined", "standard"],
        )
        .default(variant)
        .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"]),
        // A bare `primary` is what an unset `color` resolves to, so that swatch prints nothing.
        Control::color("color"),
        Control::sizes("size").default(size),
        Control::sizes("radius").default(radius),
        Control::select(
            "placement",
            [
                "top-start",
                "top-center",
                "top-end",
                "center-start",
                "center-center",
                "center-end",
                "bottom-start",
                "bottom-center",
                "bottom-end",
            ],
        )
        .labels([
            "Top start",
            "Top center",
            "Top end",
            "Center start",
            "Center",
            "Center end",
            "Bottom start",
            "Bottom center",
            "Bottom end",
        ])
        .default(placement),
        Control::slider("offset", ["0", "xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
        // Icon-only, the name is `aria_label`; extended, the label.
        Control::switch("children").code(|_, values| match extended(values) {
            true => vec![],
            false => vec![format!("aria_label: {LABEL:?}")],
        }),
        Control::switch("tooltip").hidden_when(extended),
        Control::switch("disabled"),
        Control::switch("loading"),
        Control::switch("hidden"),
        // The trigger of a `Menu`, not a prop: `wrap` prints the Menu around the button.
        Control::switch("menu").default("true").code(|_, values| {
            match values.str("menu") == "true" {
                true => vec!["attributes: menu.a11y_attributes()".to_string()],
                false => vec![],
            }
        }),
    ]
}

#[component]
pub fn FloatingActionButtonPage() -> Element {
    let theme = use_theme();
    let defaults = theme.floating_action_button;
    let menu = use_menu();
    rsx! {
        DocPage {
            title: "FloatingActionButton",
            source: "libero/src/components/buttons/floating_action_button.rs",
            markdown: "/md/floating_action_button.md",
            properties: vec![props("FloatingActionButton", vec![
                prop("icon", "Element")
                    .default("required")
                    .doc("The glyph, as `rsx! { Pictogram { icon: lucide::plus::outlined } }`. It takes `Icon`'s size at the button's step."),
                prop("aria_label", "String")
                    .doc("The name of an icon-only button, after the action: \"New message\", not \"Plus\". With `children`, leave it unset: the label names the button."),
                prop("tooltip", "bool")
                    .default("false")
                    .doc("Icon-only: shows `aria_label` in a `Tooltip` on hover and keyboard focus."),
                prop("variant", "Responsive<Variant>")
                    .default(defaults.variant.as_str())
                    .doc("Visual style, as on `Button`. Or one per breakpoint, `variant: responsive(Variant::Outlined).md(Variant::Filled)`."),
                prop("gradient", "Gradient")
                    .doc("With `variant: \"gradient\"`: the second stop and the angle. The first stop is `color`."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Accent color. A theme color name or any CSS color."),
                prop("size", "Responsive<ThemeAwareValue>")
                    .default(defaults.size.as_str())
                    .doc("A step of `Button`'s heights, 60px at `xl`. Icon-only, a CSS length works too; with `children`, only a size word applies. Or one per breakpoint, `size: responsive(Size::Sm).md(Size::Lg)`."),
                prop("radius", "Responsive<ThemeAwareValue>")
                    .default(defaults.radius.as_str())
                    .doc("Corner radius. The default rounds an icon-only button into a circle and an extended one into a pill; `lg` gives a rounded square. Or one per breakpoint, `radius: responsive(Size::Sm).md(Size::Xl)`."),
                prop("hidden_from", "Size")
                    .doc("Hidden from this breakpoint's width up, as `hidden_from: \"md\"`."),
                prop("visible_from", "Size")
                    .doc("Hidden below this breakpoint's width, as `visible_from: \"md\"`."),
                prop("placement", "Placement")
                    .default(defaults.placement.as_str())
                    .doc("The corner or edge it sits at. `start` and `end` follow the text direction."),
                prop("offset", "ThemeAwareValue")
                    .default(defaults.offset.as_str())
                    .doc("Distance from the placed edges: a spacing word or any CSS length."),
                prop("fixed", "bool")
                    .default("true")
                    .doc("Against the viewport. A bottom one sits above a docked `NavigationBar` and the bottom safe area, a top one below the top safe area. `false` places it in the nearest `position: relative` ancestor instead."),
                prop("z_index", "ThemeAwareValue")
                    .default("header layer")
                    .doc("Stacking layer. The default is `Header`'s: above the page, below the focused skip link, floating windows, overlays and modals."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
                prop("loading", "bool")
                    .default("false")
                    .doc("Shows a `Loader` and ignores presses; the button stays focusable and sets `aria-busy`. Ignored on a link."),
                prop("loader", "Element")
                    .doc("Drawn in place of the `Loader` while `loading`."),
                prop("ripple", "bool")
                    .default("true")
                    .doc("The press ripple. `false` drops it."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler. Not called on a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders a link instead of a `<button>`: a path, a URL or a typed route."),
                prop("target", "String")
                    .doc("The link's `target` attribute. `\"_blank\"` adds a hidden \"(opens in a new tab)\"."),
                prop("children", "Element")
                    .doc("A visible label: an extended button, the `icon` before the label."),
                prop("hidden", "bool")
                    .default("false")
                    .doc("Fades it out, and out of the tab order and the accessibility tree at once, as for hiding it while the page scrolls down. Instant under reduced motion."),
            ])],
            accessibility: a11y()
                .handles([
                    "Icon-only, it is an `ActionIcon`, named by `aria_label`; an empty one warns in the console.",
                    "Its shadow steps aside on keyboard focus, so the focus ring is drawn whole over it.",
                    "Under a finger the button is at least 44 by 44px, at every size.",
                    "A fixed one sits above a docked `NavigationBar` and out of the safe areas; `start` and `end` flip under `dir=\"rtl\"`.",
                    "`hidden` takes it out of the tab order and the accessibility tree at once, before the fade ends.",
                    "As a `Menu`'s trigger, a press on it while `loading` or `disabled` opens nothing, and its `tooltip` leaves while the menu is open.",
                ])
                .must([
                    "Name an icon-only button after its action with `aria_label`, such as \"New message\".",
                    "Render it where its action belongs in the reading order, such as at the end of `main`: Tab reaches it in DOM order, not where it floats.",
                    "Pad the end of the page by its height plus `offset`, so it never covers the last content or the control that has focus.",
                    "Move focus elsewhere before setting `hidden` while focus may be on it: hidden, it drops focus to the page.",
                    "As a `Menu`'s trigger, pass `Menu { disabled: loading }` too, as the arrow keys on a loading button still reach the menu.",
                    "As a `Menu`'s trigger, keep it shown while the menu is open, as with `hidden: scrolled() && !menu.is_open()`: hidden, it leaves the menu nothing to anchor on, and Escape drops focus to the page.",
                    "Keep it apart from `Notifications`, whose stack also sits at `bottom-end`: move one with `Notifications { placement }`.",
                ])
                .limits([
                    "An ancestor with a `transform`, `filter` or `contain` traps a fixed one inside it, as it does any `position: fixed` box.",
                ])
                .example("A mail app's compose button, `FloatingActionButton { aria_label: \"New message\", icon }`: a screen reader reads \"New message, button\", and Tab reaches it after the message list."),
            lead: rsx! {
                Text {
                    "The main action of a screen, as a button that floats over the page at a "
                    "corner. Icon-only by default, named by "
                    Code { source: "aria_label" }
                    "; "
                    Code { source: "children" }
                    " make it an extended button with a label. It is fixed to the viewport, "
                    Code { source: "offset" }
                    " from the edges and above a docked "
                    Code { source: "NavigationBar" }
                    ". For several actions, make it the trigger of a "
                    Code { source: "Menu" }
                    " with "
                    Code { source: "menu.a11y_attributes()" }
                    ", as the preview does. "
                    Code { source: "hidden" }
                    " fades it out. The preview sets "
                    Code { source: "fixed: false" }
                    " to stay inside its frame."
                }
            },
            Demo {
                component: "FloatingActionButton",
                children_text: "",
                wrap: Wrap(wrap),
                fixed: vec![ICON.to_string(), "fixed: false".to_string()],
                controls: controls(
                    defaults.variant.as_str(),
                    defaults.size.as_str(),
                    defaults.radius.as_str(),
                    defaults.placement.as_str(),
                ),
                render: move |values: DemoValues| {
                    let in_menu = values.str("menu") == "true";
                    let button = rsx! {
                        FloatingActionButton {
                            fixed: false,
                            icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
                            aria_label: (!extended(&values)).then(|| LABEL.to_string()),
                            tooltip: (!extended(&values) && values.str("tooltip") == "true").then_some(true),
                            variant: values.str("variant"),
                            color: color(&values),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            placement: values.str("placement"),
                            offset: values.str("offset"),
                            disabled: values.str("disabled") == "true",
                            loading: values.str("loading") == "true",
                            hidden: values.str("hidden") == "true",
                            attributes: if in_menu { menu.a11y_attributes() } else { Vec::new() },
                            children: extended(&values).then(|| rsx! { {LABEL} }),
                        }
                    };
                    rsx! {
                        Box { sx: sx().position("relative").width("100%").height("240px"),
                            if in_menu {
                                Menu {
                                    state: menu,
                                    disabled: values.str("loading") == "true",
                                    side: Side::Top,
                                    align: Align::End,
                                    items: menu_items(),
                                    {button}
                                }
                            } else {
                                {button}
                            }
                        }
                    }
                },
                child: Child(text_child),
            }
        }
    }
}
