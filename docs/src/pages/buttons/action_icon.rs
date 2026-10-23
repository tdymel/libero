use crate::components::{
    Control, Demo, DemoValues, DocPage, PictogramNote, a11y, gradient_controls, gradient_value,
    not_gradient_variant, prop, props,
};
use dioxus::prelude::*;
use libero::components::{ActionIcon, Code, Input, Loader, Text};
use libero::use_theme;

/// The `shortcut` switch's chord, shown in the tooltip and as `aria-keyshortcuts`.
const SHORTCUT: &str = "mod+enter";

const CUSTOM_LOADER: &str =
    "loader: rsx! { Loader { variant: \"dots\", size: \"xs\", color: \"currentColor\" } }";

fn is_link(values: &DemoValues) -> bool {
    values.str("link") == "true"
}

fn custom_loader(values: &DemoValues) -> Option<Element> {
    (values.str("custom_loader") == "true")
        .then(|| rsx! { Loader { variant: "dots", size: "xs", color: "currentColor" } })
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
                    .doc("Visual style, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`, `gradient`. Set alone, it takes the primary color. With neither `variant` nor `color`, the button takes the surrounding text color."),
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
                    .doc("Makes it a toggle button. A bare toggle shows a 1px ring while on; with `variant` or `color` set it takes that variant's selected look. Leave it unset for a plain action."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button: `aria-disabled` rather than `disabled`, so a focused button keeps its focus. It leaves the Tab order and ignores presses."),
                prop("focusable_when_disabled", "bool")
                    .default("false")
                    .doc("With `disabled`: keeps the button in the Tab order."),
                prop("loading", "bool")
                    .default("false")
                    .doc("Shows a `Loader` over the icon and ignores clicks. The button stays focusable and sets `aria-busy`; announce the progress in your own status region. Ignored on a link."),
                prop("loader", "Element")
                    .doc("Drawn in place of the `Loader` while `loading`. Size it yourself, e.g. `Loader { size: \"xs\" }`."),
                prop("ripple", "bool")
                    .default("true")
                    .doc("The press ripple. `false` drops it. Reduced motion drops it anyway."),
                prop("value", "String")
                    .doc("Its key in the enclosing `ButtonGroup`'s `value`. With that set, the button is a toggle and a press asks the group to select it; its own `selected` wins. Also the `<button>`'s `value` attribute."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler. Not called on a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders a link instead of a `<button>`."),
                prop("target", "String")
                    .doc("The link's `target` attribute."),
                prop("new_tab_hint", "bool")
                    .default("true")
                    .doc("With `target: \"_blank\"`, \"(opens in a new tab)\" in the accessible name and the tooltip. `false` drops it and the icon."),
                prop("new_tab_icon", "bool")
                    .default(if theme.anchor.new_tab_icon { "true" } else { "false" })
                    .doc("With the hint, a small icon in the corner for sighted users. Unset, the theme's `anchor.new_tab_icon`, which is off."),
                prop("icon", "SvgData")
                    .doc("The glyph, drawn as a `Pictogram`: `pictogram_icons_lucide::x::outlined`. Wins over `children`."),
                prop("children", "Element").doc("The icon, when it is not `SvgData`: your own svg, or any element."),
            ])],
            accessibility: a11y()
                .handles([
                    "Below 24px (a length such as `\"20px\"`) the button still takes presses in a 24x24 box centred on it.",
                    "`focusable_when_disabled` keeps a disabled button in the Tab order, with `aria-disabled`.",
                    "`tooltip` repeats the accessible name, and `shortcut` writes `aria-keyshortcuts` in the ARIA key names (`Control+B`).",
                    "A link with `target: \"_blank\"` adds \"(opens in a new tab)\" to the accessible name and the tooltip. With `new_tab_icon`, it also draws a small icon in the corner for sighted users.",
                    "`loading` sets `aria-busy` and keeps the button focusable, so focus stays on it while it works.",
                ])
                .must([
                    "Name the button with `aria_label`: the icon gives a screen reader nothing to read.",
                    "Below 24px, keep other targets clear of that 24x24 box, or the one drawn later takes the overlap.",
                    "Announce the progress and the result of a `loading` button in your own `role=\"status\"` region. The button announces nothing itself.",
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
                    " is required, because the icon gives a screen reader nothing to read. Once "
                    "the button carries a visible label, use a "
                    Code { source: "Button" }
                    " with its "
                    Code { source: "icon" }
                    " prop instead."
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
                    // On, so the second button below, which always loads, shows it without `loading`.
                    Control::switch("custom_loader").default("true").hidden_when(is_link).code(|_, values| {
                        match values.str("custom_loader").as_str() {
                            "true" => vec![CUSTOM_LOADER.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("ripple").default("true").code(|_, values| {
                        match values.str("ripple").as_str() {
                            "false" => vec!["ripple: false".to_string()],
                            _ => vec![],
                        }
                    }),
                    // `to` and `target` together, since the preview's link
                    // must not navigate the docs away.
                    Control::switch("link").code(|_, values| match values.str("link").as_str() {
                        "true" => vec![
                            r#"to: "https://dioxuslabs.com""#.to_string(),
                            r#"target: "_blank""#.to_string(),
                        ],
                        _ => vec![],
                    }),
                    Control::switch("new_tab_icon")
                        .hidden_when(|values| !is_link(values))
                        .code(|_, values| match (is_link(values), values.str("new_tab_icon").as_str()) {
                            (true, "true") => vec!["new_tab_icon: true".to_string()],
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
                        loader: custom_loader(&values),
                        ripple: values.str("ripple") != "false",
                        to: match values.str("link").as_str() {
                            "true" => Input::from("https://dioxuslabs.com"),
                            _ => Input::None,
                        },
                        target: (values.str("link") == "true").then(|| "_blank".to_string()),
                        new_tab_icon: (values.str("new_tab_icon") == "true").then_some(true),
                        aria_label: "Confirm",
                        icon: pictogram_icons_lucide::check::outlined,
                    }
                    // Always loading, so the loader shows without the `loading` switch.
                    if !is_link(&values) {
                        ActionIcon {
                            variant: values.str("variant"),
                            gradient: gradient_value(&values),
                            color: match values.str("color").as_str() {
                                "primary" => Input::None,
                                color => Input::from(color),
                            },
                            size: values.str("size"),
                            radius: values.str("radius"),
                            loading: true,
                            loader: custom_loader(&values),
                            aria_label: "Saving",
                            icon: pictogram_icons_lucide::save::outlined,
                        }
                    }
                    // A new-tab link beside it that opts into the corner icon, shown without the `link` switch.
                    if !is_link(&values) {
                        ActionIcon {
                            variant: values.str("variant"),
                            gradient: gradient_value(&values),
                            color: match values.str("color").as_str() {
                                "primary" => Input::None,
                                color => Input::from(color),
                            },
                            size: values.str("size"),
                            radius: values.str("radius"),
                            disabled: values.str("disabled") == "true",
                            tooltip: values.str("tooltip") == "true",
                            to: "https://dioxuslabs.com",
                            target: "_blank",
                            new_tab_icon: true,
                            aria_label: "Dioxus",
                            icon: pictogram_icons_lucide::link::outlined,
                        }
                    }
                },
            }
        }
    }
}
