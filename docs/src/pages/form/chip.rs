use crate::components::{Control, Demo, DemoValues, DocPage, a11y, disabled_prop, prop, props};
use dioxus::prelude::*;
use libero::components::ChipPart;
use libero::components::{Chip, Code, Icon, Input, Text};
use libero::use_theme;
use pictogram_icons_lucide as lucide;

/// A bare glyph in the chip's label colour, as the demo's code prints it.
fn glyph_code(name: &str) -> String {
    format!(r#"Icon {{ variant: "standard", color: "inherit", svg: lucide::{name}::outlined }}"#)
}

#[component]
pub fn ChipPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Chip",
            source: "libero/src/components/form/chip.rs",
            markdown: "/md/chip.md",
            properties: vec![props("Chip", vec![
                prop("color", "ThemeAwareValue")
                    .default(theme.chip.color.as_str())
                    .doc("Accent color. A theme color name or any CSS color; `theme.chip.color` when unset."),
                prop("variant", "Variant")
                    .default(theme.chip.variant.as_str())
                    .doc("The unselected look. A checked chip is always a tonal container, with a check before its label."),
                prop("size", "Size").default(theme.chip.size.as_str()).doc("Height, padding and font size."),
                prop("radius", "ThemeAwareValue")
                    .default(theme.chip.radius.as_str())
                    .doc("Corner radius, or any CSS, e.g. `radius: \"0\"`."),
                prop("checked", "bool")
                    .doc("Whether it is selected. Pair it with `onchange`. Left out, a chip with a `name` keeps its own state, or the form's when that name binds it. A debug build warns when one comes without the other."),
                disabled_prop("chip"),
                prop("readonly", "bool")
                    .default("false")
                    .doc("A selectable chip stays focusable and posted with the form, but clicks and Space no longer toggle it."),
                prop("onchange", "EventHandler<bool>")
                    .doc("Called with the value `checked` should take next. Makes the chip a checkbox."),
                prop("name", "FieldName<bool>")
                    .doc("Makes the chip a checkbox that posts under this name. A path such as `Filters::FIELDS.open()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                prop("value", "String")
                    .default("on")
                    .doc("What the chip posts under its `name` when checked, so a row of filter chips can share one name."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("A plain action. Makes the chip a `<button>`; with `checked` it is a toggle button and says `aria-pressed`. Not with `onchange`, which ignores it and `to`."),
                prop("to", "NavigationTarget")
                    .doc("Makes the chip a router-aware link. Wins over `onclick`."),
                prop("target", "String")
                    .doc("Link target, such as `_blank`. Only with `to`. `\"_blank\"` adds a hidden \"(opens in a new tab)\"."),
                prop("new_tab_hint", "bool")
                    .default("true")
                    .doc("`false` drops the hidden text a `\"_blank\"` target adds, and the icon."),
                prop("new_tab_icon", "bool")
                    .default(if theme.anchor.new_tab_icon { "true" } else { "false" })
                    .doc("With the hint, draws an external icon after the label. Unset, the theme's `anchor.new_tab_icon`, which is off."),
                prop("icon", "Element")
                    .doc("Drawn before the label, with a gap. It never shrinks. An `Icon` with no `size` takes the chip's icon step: `xs` up to an `md` chip, `sm` for `lg` and `xl`, `md` for `xxl`."),
                prop("trailing", "Element")
                    .doc("Drawn after the label, with a gap, such as a remove button or a glyph. It never shrinks, and sizes an unsized `Icon` as `icon` does. Not a control on an `onclick` or `to` chip, which is a button already."),
                prop("children", "Element")
                    .doc("The label, cut at the chip's edge. Text and `Icon` only."),
            ])
            .parts("ChipPart", vec![
                (ChipPart::Icon, "The leading glyph's wrapper."),
                (ChipPart::Trailing, "The slot after the label, with `trailing`."),
                (ChipPart::NewTab, "The new-tab icon after the label, on a `to` chip with `target: \"_blank\"` and `new_tab_icon` only."),
            ])],
            accessibility: a11y()
                .key(["Space"], "Toggles a selectable chip.")
                .handles([
                    "A `readonly` chip keeps its tab stop and ignores the toggle.",
                    "An `onclick` chip with `checked` sets `aria-pressed` from it; without `checked` it is a plain button.",
                    "A selectable chip's `aria_label` goes on its checkbox, not on the root `span`.",
                    "A chip in a list field (`MultiSelect`, `TagsField`) cut with an ellipsis shows its full text in a `title`. That is a hover reveal only: a keyboard or touch user does not get it, so keep labels short.",
                ])
                .must([
                    "Keep `children` to text and `Icon`: a selectable chip is a `<label>`, which takes the clicks of any control inside it.",
                    "Name an icon-only chip: its checkbox takes the name of its label, so give the `Icon` an `aria_label`, or the chip an `aria_label`.",
                ])
                .example("A filter chip, `Chip { checked, onchange, \"Vegan\" }`: Tab lands on it, Space toggles it, and a `readonly` chip keeps its tab stop but stays as it is.")
                .limits(["With `selected_check`, picking a chip widens it by the check, so a row of chips reflows; theme `selected_check: false` keeps its width."]),
            lead: rsx! {
                Text {
                    "A compact token. With "
                    Code { source: "onchange" }
                    " or a "
                    Code { source: "name" }
                    " it is a checkbox, with "
                    Code { source: "onclick" }
                    " a button, with "
                    Code { source: "to" }
                    " a link, and with none of them a plain tag. "
                    Code { source: "variant" }
                    " sets the unselected look. A checked chip is always a tinted container, with a check before its label. "
                    Code { source: "icon" }
                    " and "
                    Code { source: "trailing" }
                    " take a glyph on either side; "
                    Code { source: "color: \"neutral\", variant: \"tonal\"" }
                    " makes the grey info pill."
                }
            },
            // snippet: item use pictogram_icons_lucide as lucide;
            // snippet: let mut selected = use_signal(|| false);
            Demo {
                component: "Chip",
                children_text: "rust",
                controls: vec![
                    Control::color("color").default(theme.chip.color.as_str()),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "text"],
                    ).default(theme.chip.variant.as_str())
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Text"]),
                    Control::sizes("size")
                        .default(theme.chip.size.as_str()),
                    Control::sizes("radius")
                        .default(theme.chip.radius.as_str()),
                    // What the chip is: a plain tag, a checkbox, a button or
                    // a link. The last three never combine.
                    Control::toggle("kind", ["tag", "filter", "action", "link"])
                        .labels(["Tag", "Filter", "Action", "Link"])
                        .code(|_, values| match values.str("kind").as_str() {
                            // Controlled state is `checked` + `onchange`; the
                            // library warns about one without the other.
                            "filter" => vec![
                                "checked: selected()".to_string(),
                                "onchange: move |next| selected.set(next)".to_string(),
                            ],
                            "action" => vec!["onclick: move |_| {}".to_string()],
                            "link" => vec![
                                r#"to: "https://dioxuslabs.com""#.to_string(),
                                r#"target: "_blank""#.to_string(),
                            ],
                            _ => vec![],
                        }),
                    // The preview writes `onchange` back into this switch;
                    // `kind` prints the pair a caller writes.
                    Control::switch("checked")
                        .hidden_when(|values| values.str("kind") != "filter")
                        .code(|_, _| vec![]),
                    // On here so the icon shows; the component's own default is off.
                    Control::switch("new_tab_icon")
                        .default("true")
                        .hidden_when(|values| values.str("kind") != "link")
                        .code(|_, values| {
                            match (values.str("kind").as_str(), values.str("new_tab_icon").as_str()) {
                                ("link", "true") => vec!["new_tab_icon: true".to_string()],
                                _ => vec![],
                            }
                        }),
                    // On by default so the slots show; an unsized `Icon` takes the chip's icon step.
                    Control::toggle("icons", ["none", "leading", "trailing", "both"])
                        .labels(["None", "Leading", "Trailing", "Both"])
                        .default("both")
                        .code(|_, values| {
                            let icons = values.str("icons");
                            let mut lines = Vec::new();
                            if matches!(icons.as_str(), "leading" | "both") {
                                lines.push(format!("icon: rsx! {{ {} }}", glyph_code("info")));
                            }
                            if matches!(icons.as_str(), "trailing" | "both") {
                                lines.push(format!("trailing: rsx! {{ {} }}", glyph_code("chevron_down")));
                            }
                            lines
                        }),
                    Control::switch("disabled"),
                    Control::switch("readonly")
                        .hidden_when(|values| values.str("kind") != "filter"),
                ],
                render: move |values: DemoValues| rsx! {
                    Chip {
                        color: values.str("color"),
                        variant: values.str("variant"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        // Both or neither: `checked` alone can never
                        // change, `onchange` alone can never look selected.
                        checked: (values.str("kind") == "filter")
                            .then(|| values.str("checked") == "true"),
                        onchange: (values.str("kind") == "filter").then(|| {
                            let values = values.clone();
                            EventHandler::new(move |next: bool| values.set("checked", next.to_string()))
                        }),
                        onclick: (values.str("kind") == "action")
                            .then(|| EventHandler::new(move |_: MouseEvent| {})),
                        to: match values.str("kind").as_str() {
                            "link" => Input::from("https://dioxuslabs.com"),
                            _ => Input::None,
                        },
                        target: (values.str("kind") == "link").then(|| "_blank".to_string()),
                        new_tab_icon: (values.str("kind") == "link"
                            && values.str("new_tab_icon") == "true")
                            .then_some(true),
                        disabled: values.str("disabled") == "true",
                        readonly: values.str("kind") == "filter" && values.str("readonly") == "true",
                        icon: matches!(values.str("icons").as_str(), "leading" | "both").then(|| rsx! {
                            Icon { variant: "standard", color: "inherit", svg: lucide::info::outlined }
                        }),
                        // No control in `trailing` on an `onclick` or `to` chip: a glyph only.
                        trailing: matches!(values.str("icons").as_str(), "trailing" | "both").then(|| rsx! {
                            Icon { variant: "standard", color: "inherit", svg: lucide::chevron_down::outlined }
                        }),
                        "rust"
                    }
                    // A new-tab link beside it that opts into the icon, shown without the `link` kind.
                    if values.str("kind") != "link" {
                        Chip {
                            color: values.str("color"),
                            variant: values.str("variant"),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            to: "https://dioxuslabs.com",
                            target: "_blank",
                            new_tab_icon: true,
                            "Dioxus"
                        }
                    }
                },
            }
        }
    }
}
