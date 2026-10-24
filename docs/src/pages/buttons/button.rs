use crate::components::{
    Control, Demo, DemoValues, DocPage, a11y, gradient_controls, gradient_value,
    not_gradient_variant, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Button, ButtonPart, Code, Flex, Input, Loader, Pictogram, Text};
use libero::use_theme;
use pictogram_icons_lucide as lucide;

const TRAILING_ICON: &str = "trailing_icon: rsx! { Pictogram { icon: lucide::chevron_right::outlined, width: \"1.25em\", height: \"1.25em\" } }";

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
pub fn ButtonPage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(not_gradient_variant);
    rsx! {
        DocPage {
            title: "Button",
            source: "libero/src/components/buttons/button.rs",
            markdown: "/md/button.md",
            properties: vec![props("Button", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Accent color. A theme color name or any CSS color. Under a gradient, its first stop. Unset, a `standard` or `outlined` button on a gradient `Paper`, a coloured or gradient `Header`, a filled, tonal or gradient `Alert` or a `Mark` takes that surface's text color. A secondary action beside a filled primary one, such as \"Cancel\" beside \"Save\", is `color: \"neutral\"` with `variant: \"outlined\"`; `\"ink\"` draws it in the page's text colour instead."),
                prop("variant", "Variant")
                    .default(theme.button.variant.as_str())
                    .doc("Visual style, from most to least emphasis: `filled`, `tonal`, `elevated`, `outlined`, `standard`. `gradient` fills it from `color` into the theme's second stop."),
                prop("gradient", "Gradient")
                    .doc("With `variant: \"gradient\"`: the second stop and the angle, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`. The first stop is `color`. A literal CSS stop's label contrast is yours to check. Ignored by the other variants."),
                prop("radius", "ThemeAwareValue")
                    .default(theme.button.radius.as_str())
                    .doc("Corner radius, independent of `size`, or any CSS, e.g. `radius: \"0\"`."),
                prop("size", "Size")
                    .default(theme.button.size.as_str())
                    .doc("Height, padding and font size."),
                prop("full_width", "bool")
                    .default("false")
                    .doc("Stretches the button to fill its container."),
                prop("justify", "ThemeAwareValue")
                    .default("center")
                    .doc("Where the icons and the label sit along the button, as `justify-content`: `start`, `end`, `space-between` or any CSS. It shows on a `full_width` button or one given a width. Unset, they are centred, and an overlong label is cut at its end."),
                prop("gap", "ThemeAwareValue")
                    .default("xs")
                    .doc("Space between the icons and the label: a size word or any CSS, as `gap: \"0\"`. It faces the label under `dir=\"rtl\"` too."),
                prop("selected", "bool")
                    .doc("Makes it a toggle button with the selected look. Leave it unset for a plain action."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables and dims the button: `aria-disabled` rather than `disabled`, so a focused button keeps its focus. It leaves the Tab order and ignores presses."),
                prop("focusable_when_disabled", "bool")
                    .default("false")
                    .doc("With `disabled`: keeps the button in the Tab order."),
                prop("loading", "bool")
                    .default("false")
                    .doc("Shows a `Loader` over the label and ignores clicks. The button stays focusable and keeps its width, and sets `aria-busy`; announce the progress in your own status region. Ignored on a link."),
                prop("loader", "Element")
                    .doc("Drawn in place of the `Loader` while `loading`. Size it yourself, e.g. `Loader { size: \"xs\" }`."),
                prop("loading_position", "LoadingPosition")
                    .default("center")
                    .doc("Where the loader sits: `center` over the hidden label, or `start` and `end` in the place of `icon` and `trailing_icon`, with the label still visible."),
                prop("ripple", "bool")
                    .default("true")
                    .doc("The press ripple. `false` drops it, for a dense toolbar or a theme without one. Reduced motion drops it anyway."),
                prop("value", "String")
                    .doc("Its key in the enclosing `ButtonGroup`'s `value`. With that set, the button is a toggle and a press asks the group to select it; its own `selected` wins. Also the `<button>`'s `value` attribute."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler. Not called on a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders a link instead of a `<button>`. Takes a path, a URL or a typed route (`Route::Foo {}`), or an `Option` of one: `None` is a plain button."),
                prop("target", "String")
                    .doc("The link's `target` attribute. `\"_blank\"` adds a hidden \"(opens in a new tab)\"."),
                prop("new_tab_hint", "bool")
                    .default("true")
                    .doc("With `target: \"_blank\"`, the hidden \"(opens in a new tab)\". `false` drops it and the icon."),
                prop("new_tab_icon", "bool")
                    .default(if theme.anchor.new_tab_icon { "true" } else { "false" })
                    .doc("With `target: \"_blank\"`, draws a small external icon after the label. Unset, the theme's `anchor.new_tab_icon`, which is off."),
                prop("icon", "Element")
                    .doc("Drawn before the label. It never shrinks."),
                prop("trailing_icon", "Element")
                    .doc("Drawn after the label, as a chevron or an arrow. It never shrinks, and on a link it comes before the new-tab icon."),
                prop("parts", "Parts<ButtonPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                prop("children", "Element")
                    .default("required")
                    .doc("The label, on one line. A long one is cut at the edge."),
            ]).extends("button")
            .parts("ButtonPart", vec![
                (ButtonPart::Icon, "The `icon` wrapper."),
                (ButtonPart::TrailingIcon, "The `trailing_icon` wrapper."),
            ])],
            accessibility: a11y()
                .handles([
                    "A label cut at the edge is still the full accessible name.",
                    "A link with `target: \"_blank\"` reads a hidden \"(opens in a new tab)\". With `new_tab_icon`, it also draws a small external icon.",
                    "`selected: Some(false)` announces a toggle that is off. An unset `selected` announces no state.",
                    "`focusable_when_disabled` keeps a disabled button in the Tab order, with `aria-disabled`.",
                    "`loading` sets `aria-busy` and keeps the button focusable, so focus stays on it while it works.",
                ])
                .must([
                    "Pass a label that may be cut as `title` too, so sighted users can read it on hover.",
                    "Announce the progress and the result of a `loading` button in your own `role=\"status\"` region. The button announces nothing itself.",
                ])
                .example("A narrow toolbar button whose label may be cut, `Button { title: \"Export as PDF\", \"Export as PDF\" }`: a screen reader still reads the full label, and the title shows it on hover."),
            lead: rsx! {
                Text {
                    "A clickable action, a toggle, or a link when "
                    Code { source: "to" }
                    " is set. It defaults to "
                    Code { source: "type=\"button\"" }
                    ", so it never submits a form by accident. A submit button sets "
                    Code { source: "r#type: \"submit\"" }
                    ". For an icon-only button, use "
                    Code { source: "ActionIcon" }
                    "."
                }
            },
            Demo {
                component: "Button",
                children_text: "Save changes",
                controls: vec![
                    Control::color("color"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard", "gradient"],
                    ).default(theme.button.variant.as_str())
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard", "Gradient"]),
                    gradient_to,
                    gradient_deg,
                    Control::sizes("size")
                        .default(theme.button.size.as_str()),
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.button.radius.as_str()),
                    Control::switch("full_width"),
                    // The first stop is the unset look, so it prints nothing.
                    Control::toggle("justify", ["center", "start", "end", "space-between"])
                        .labels(["Center", "Start", "End", "Space between"])
                        .default("center")
                        .code(|_, values| match values.str("justify").as_str() {
                            "center" => vec![],
                            value => vec![format!("justify: {value:?}")],
                        }),
                    // `0` is plain CSS; the trailing icon below shows it without another switch.
                    Control::slider("gap", ["0", "xs", "sm", "md", "lg", "xl"]).default("xs"),
                    // On, so the icon shows without another switch.
                    Control::switch("trailing_icon").default("true").code(|_, values| {
                        match values.str("trailing_icon").as_str() {
                            "true" => vec![TRAILING_ICON.to_string()],
                            _ => vec![],
                        }
                    }),
                    // Off is still a toggle (`aria-pressed="false"`); unset
                    // is a plain action.
                    Control::toggle("selected", ["unset", "false", "true"])
                        .labels(["Unset", "Off", "On"])
                        .default("unset")
                        // A link keeps the look but not `aria-pressed`, and warns.
                        .hidden_when(is_link)
                        .code(|_, values| match values.str("selected").as_str() {
                            "unset" => vec![],
                            value => vec![format!("selected: {value}")],
                        }),
                    Control::switch("disabled"),
                    Control::switch("focusable_when_disabled")
                        .hidden_when(|values| values.str("disabled") != "true"),
                    // An `<a>` has nothing to wait for: ignored, with a warning.
                    Control::switch("loading").hidden_when(is_link),
                    // On and at the start, so the second button below, which always loads,
                    // shows both without another switch.
                    Control::toggle("loading_position", ["center", "start", "end"])
                        .labels(["Center", "Start", "End"])
                        .default("start")
                        .hidden_when(is_link)
                        .code(|_, values| match values.str("loading_position").as_str() {
                            "center" => vec![],
                            value => vec![format!("loading_position: {value:?}")],
                        }),
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
                    // A `GlobalAttributes` pass-through, printed as the raw identifier. A link has no `type`.
                    Control::toggle("type", ["button", "submit", "reset"])
                        .labels(["Button", "Submit", "Reset"])
                        .hidden_when(is_link)
                        .code(
                        |control, values| match values.str("type") {
                            _ if values.str("link") == "true" => vec![],
                            value if value == control.default => vec![],
                            value => vec![format!("r#type: {value:?}")],
                        },
                    ),
                ],
                render: move |values: DemoValues| rsx! {
                    Flex { direction: "row", gap: "sm",
                        Button {
                            color: values.str("color"),
                            variant: values.str("variant"),
                            gradient: gradient_value(&values),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            full_width: values.str("full_width") == "true",
                            justify: match values.str("justify").as_str() {
                                "center" => Input::None,
                                value => Input::from(value),
                            },
                            selected: match values.str("selected").as_str() {
                                _ if is_link(&values) => None,
                                "true" => Some(true),
                                "false" => Some(false),
                                _ => None,
                            },
                            disabled: values.str("disabled") == "true",
                            focusable_when_disabled: values.str("focusable_when_disabled") == "true",
                            loading: values.str("loading") == "true" && !is_link(&values),
                            loading_position: values.str("loading_position"),
                            loader: custom_loader(&values),
                            gap: values.str("gap"),
                            ripple: values.str("ripple") != "false",
                            r#type: values.str("type"),
                            to: match values.str("link").as_str() {
                                "true" => Input::from("https://dioxuslabs.com"),
                                _ => Input::None,
                            },
                            target: (values.str("link") == "true").then(|| "_blank".to_string()),
                            new_tab_icon: (values.str("new_tab_icon") == "true").then_some(true),
                            trailing_icon: (values.str("trailing_icon") == "true")
                                .then(|| rsx! { Pictogram { icon: lucide::chevron_right::outlined, width: "1.25em", height: "1.25em" } }),
                            "Save changes"
                        }
                        // Always loading, so the position and the loader show without the `loading` switch.
                        if !is_link(&values) {
                            Button {
                                variant: values.str("variant"),
                                color: values.str("color"),
                                size: values.str("size"),
                                loading: true,
                                loading_position: values.str("loading_position"),
                                loader: custom_loader(&values),
                                icon: rsx! { Pictogram { icon: lucide::save::outlined } },
                                "Saving"
                            }
                        }
                        // A full-width row with `justify`, shown without a switch.
                        if !is_link(&values) {
                            Button {
                                variant: values.str("variant"),
                                color: values.str("color"),
                                size: values.str("size"),
                                full_width: true,
                                justify: "space-between",
                                icon: rsx! { Pictogram { icon: lucide::user::outlined, width: "1.25em", height: "1.25em" } },
                                trailing_icon: rsx! { Pictogram { icon: lucide::chevron_right::outlined, width: "1.25em", height: "1.25em" } },
                                "Account"
                            }
                        }
                        // The secondary action beside a filled one, shown without a switch.
                        if !is_link(&values) {
                            Button { color: "neutral", variant: "outlined", size: values.str("size"), "Cancel" }
                        }
                        // A new-tab link beside it that opts into the icon, shown without the `link` switch.
                        if !is_link(&values) {
                            Button {
                                variant: values.str("variant"),
                                color: values.str("color"),
                                size: values.str("size"),
                                to: "https://dioxuslabs.com",
                                target: "_blank",
                                new_tab_icon: true,
                                "Dioxus"
                            }
                        }
                    }
                },
            }
        }
    }
}
