use crate::components::{Child, Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Box, Code, FloatingActionButton, Input, Pictogram, Text};
use libero::sx::sx;
use libero::use_theme;
use pictogram_icons_lucide as lucide;

const ICON: &str = "icon: rsx! { Pictogram { icon: lucide::plus::outlined } }";
const LABEL: &str = "New message";

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
    ]
}

#[component]
pub fn FloatingActionButtonPage() -> Element {
    let theme = use_theme();
    let defaults = theme.floating_action_button;
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
                prop("variant", "Variant")
                    .default(defaults.variant.as_str())
                    .doc("Visual style, as on `Button`."),
                prop("gradient", "Gradient")
                    .doc("With `variant: \"gradient\"`: the second stop and the angle. The first stop is `color`."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Accent color. A theme color name or any CSS color."),
                prop("size", "ThemeAwareValue")
                    .default(defaults.size.as_str())
                    .doc("A step of `Button`'s heights, 60px at `xl`. Icon-only, a CSS length works too; with `children`, only a size word applies."),
                prop("radius", "ThemeAwareValue")
                    .default(defaults.radius.as_str())
                    .doc("Corner radius. The default rounds an icon-only button into a circle and an extended one into a pill; `lg` gives a rounded square."),
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
            ])],
            accessibility: a11y()
                .handles([
                    "Icon-only, it is an `ActionIcon`, named by `aria_label`; an empty one warns in the console.",
                    "Its shadow steps aside on keyboard focus, so the focus ring is drawn whole over it.",
                    "Under a finger the button is at least 44 by 44px, at every size.",
                    "A fixed one sits above a docked `NavigationBar` and out of the safe areas; `start` and `end` flip under `dir=\"rtl\"`.",
                ])
                .must([
                    "Name an icon-only button after its action with `aria_label`, such as \"New message\".",
                    "Render it where its action belongs in the reading order, such as at the end of `main`: Tab reaches it in DOM order, not where it floats.",
                    "Pad the end of the page by its height plus `offset`, so it never covers the last content or the control that has focus.",
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
                    ". The preview sets "
                    Code { source: "fixed: false" }
                    " to stay inside its frame."
                }
            },
            Demo {
                component: "FloatingActionButton",
                children_text: "",
                fixed: vec![ICON.to_string(), "fixed: false".to_string()],
                controls: controls(
                    defaults.variant.as_str(),
                    defaults.size.as_str(),
                    defaults.radius.as_str(),
                    defaults.placement.as_str(),
                ),
                render: move |values: DemoValues| rsx! {
                    Box { sx: sx().position("relative").width("100%").height("240px"),
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
                            children: extended(&values).then(|| rsx! { {LABEL} }),
                        }
                    }
                },
                child: Child(text_child),
            }
        }
    }
}
