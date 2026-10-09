use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, PictogramNote, a11y, gradient_controls,
    gradient_value, not_gradient_variant, prop, props,
};
use dioxus::prelude::*;
use libero::components::{ActionIcon, Code, Input, Text};
use libero::use_theme;

/// The `shortcut` switch's chord, shown in the tooltip and as `aria-keyshortcuts`.
const SHORTCUT: &str = "mod+enter";

fn is_link(values: &DemoValues) -> bool {
    values.str("link") == "true"
}

#[component]
pub fn ActionIconPage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(not_gradient_variant);
    rsx! {
        DocPage {
            title: "ActionIcon",
            source: "libero/src/components/buttons/action_icon.rs",
            markdown: "/md/action_icon.md",
            properties: vec![props("ActionIcon", vec![
                prop("variant", "Variant")
                    .doc("Visual style, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`, `gradient`. With `color` also unset, the button takes the surrounding text color."),
                prop("gradient", "Gradient")
                    .doc("With `variant: \"gradient\"`: the second stop and the angle, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`. The first stop is `color`. Ignored by the other variants."),
                prop("color", "ThemeAwareValue")
                    .doc("Accent color. A theme color name or any CSS color. Set alone, it gives the theme's default variant, `filled`. Under a gradient, its first stop."),
                prop("size", "ThemeAwareValue")
                    .default(theme.action_icon.size.as_str())
                    .doc("A size word takes `Button`'s height at that step, so the two line up in a row, and sizes the icon inside as `Icon`'s. A length such as `\"20px\"` sizes the box, and the icon fills it."),
                prop("radius", "ThemeAwareValue")
                    .default(theme.action_icon.radius.as_str())
                    .doc("Corner radius, independent of `size`."),
                prop("aria_label", "String")
                    .default("required")
                    .doc("The button's accessible name."),
                prop("tooltip", "bool")
                    .default("false")
                    .doc("Shows `aria_label` in a `Tooltip` on hover and keyboard focus. The text is the name itself, so a screen reader does not hear it twice."),
                prop("shortcut", "String")
                    .doc("The chord that runs the action, as `use_hotkeys` takes it (`\"mod+b\"`). Shown in the tooltip as keys named for the platform, and set as `aria-keyshortcuts`. Binding the key is up to you."),
                prop("selected", "bool")
                    .doc("Makes it a toggle button. The selected look shows once `variant` or `color` is set. Leave it unset for a plain action."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button."),
                prop("focusable_when_disabled", "bool")
                    .default("false")
                    .doc("With `disabled`: keeps the button in the Tab order. It renders `aria-disabled` rather than `disabled` and ignores presses."),
                prop("loading", "bool")
                    .default("false")
                    .doc("Shows a `Loader` over the icon and ignores clicks. The button stays focusable. Ignored on a link."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler. Not called on a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders a link instead of a `<button>`."),
                prop("target", "String")
                    .doc("The link's `target` attribute."),
                prop("icon", "SvgData")
                    .doc("The glyph, drawn as a `Pictogram`: `pictogram_icons_lucide::x::outlined`. Wins over `children`."),
                prop("children", "Element").doc("The icon, when it is not `SvgData`: your own svg, or any element."),
            ])],
            accessibility: a11y()
                .handles([
                    "Below 24px (a length such as `\"20px\"`) the button still takes presses in a 24x24 box centred on it.",
                    "`focusable_when_disabled` keeps a disabled button in the Tab order, with `aria-disabled`.",
                    "`tooltip` repeats the accessible name, and `shortcut` writes `aria-keyshortcuts` in the ARIA key names (`Control+B`).",
                ])
                .must([
                    "Name the button with `aria_label`: the icon gives a screen reader nothing to read.",
                    "Below 24px, keep other targets clear of that 24x24 box, or the one drawn later takes the overlap.",
                ])
                .example("A bold button in an editor bar, `ActionIcon { aria_label: \"Bold\", shortcut: \"Control+B\", .. }`: a screen reader reads \"Bold, button\" and announces the shortcut."),
            lead: rsx! {
                PictogramNote {}
                Text {
                    "An icon-only button for actions like copy, close or delete. It renders a "
                    Code { source: "<button>" }
                    ", or a link when "
                    Code { source: "to" }
                    " is set. "
                    Code { source: "aria_label" }
                    " is required, because the icon gives a screen reader nothing to read."
                }
                Text {
                    "With neither "
                    Code { source: "variant" }
                    " nor "
                    Code { source: "color" }
                    " set, it has no background of its own and takes the surrounding text color."
                }
            },
            Demo {
                component: "ActionIcon",
                children_text: "",
                fixed: vec![
                    "aria_label: \"Confirm\"".to_string(),
                    "icon: pictogram_icons_lucide::check::outlined".to_string(),
                ],
                controls: vec![
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard", "gradient"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard", "Gradient"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color("color"),
                    gradient_to,
                    gradient_deg,
                    Control::sizes("size").default(theme.action_icon.size.as_str()),
                    Control::sizes("radius")
                        .default(theme.action_icon.radius.as_str()),
                    // As on Button: off is still a toggle, unset a plain action.
                    Control::toggle("selected", ["unset", "false", "true"])
                        .labels(["Unset", "Off", "On"])
                        .default("unset")
                        .hidden_when(is_link)
                        .code(|_, values| match values.str("selected").as_str() {
                            "unset" => vec![],
                            value => vec![format!("selected: {value}")],
                        }),
                    Control::switch("tooltip"),
                    Control::switch("shortcut").code(|_, values| match values.str("shortcut").as_str() {
                        "true" => vec![format!("shortcut: {SHORTCUT:?}")],
                        _ => vec![],
                    }),
                    Control::switch("disabled"),
                    Control::switch("focusable_when_disabled")
                        .hidden_when(|values| values.str("disabled") != "true"),
                    // An `<a>` has nothing to wait for: ignored, with a warning.
                    Control::switch("loading").hidden_when(is_link),
                    // `to` and `target` together, since the preview's link
                    // must not navigate the docs away.
                    Control::switch("link").code(|_, values| match values.str("link").as_str() {
                        "true" => vec![
                            r#"to: "https://dioxuslabs.com""#.to_string(),
                            r#"target: "_blank""#.to_string(),
                        ],
                        _ => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    ActionIcon {
                        variant: values.str("variant"),
                        gradient: gradient_value(&values),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        selected: match values.str("selected").as_str() {
                            _ if is_link(&values) => None,
                            "true" => Some(true),
                            "false" => Some(false),
                            _ => None,
                        },
                        tooltip: values.str("tooltip") == "true",
                        shortcut: (values.str("shortcut") == "true").then(|| SHORTCUT.to_string()),
                        disabled: values.str("disabled") == "true",
                        focusable_when_disabled: values.str("focusable_when_disabled") == "true",
                        loading: values.str("loading") == "true" && !is_link(&values),
                        to: match values.str("link").as_str() {
                            "true" => Input::from("https://dioxuslabs.com"),
                            _ => Input::None,
                        },
                        target: (values.str("link") == "true").then(|| "_blank".to_string()),
                        aria_label: "Confirm",
                        icon: pictogram_icons_lucide::check::outlined,
                    }
                },
            }

            DocSection {
                title: "When to use it",
                Text {
                    "Use it when the icon is the whole control: a square button that lines up with a "
                    Code { source: "Button" }
                    " of the same size, and cannot be built without an accessible name. Once the button "
                    "carries a visible label, use a "
                    Code { source: "Button" }
                    " with its "
                    Code { source: "icon" }
                    " prop instead. A "
                    Code { source: "Button" }
                    " with an icon and no text is a wide pill and has no name unless you add "
                    Code { source: "aria-label" }
                    " yourself."
                }
            }
        }
    }
}
