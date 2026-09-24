use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::RatingPart;
use libero::components::{Code, Rating, Text};

const VALUES: [&str; 21] = [
    "0", "0.5", "1", "1.5", "2", "2.5", "3", "3.5", "4", "4.5", "5", "5.5", "6", "6.5", "7", "7.5",
    "8", "8.5", "9", "9.5", "10",
];

#[component]
pub fn RatingPage() -> Element {
    rsx! {
        DocPage {
            title: "Rating",
            source: "libero/src/components/form/rating/rating.rs",
            markdown: "/md/rating.md",
            properties: vec![props("Rating", vec![
                prop("value", "f64")
                    .default("0")
                    .doc("The rating, from 0 (unrated) to `count`. Pair it with `onchange`, or bind it with `name` in a `Form`. A value between steps draws as it is, so an average of 4.3 shows 4.3 stars."),
                prop("onchange", "EventHandler<f64>")
                    .doc("Called with the value `value` should take next: on a click or tap, along a sideways drag, and on each key. With `0` when `clearable` clears it."),
                prop("onhover", "EventHandler<Option<f64>>")
                    .doc("The value under a mouse or pen, and `None` once it leaves or presses. The stars show it in place of `value` meanwhile. A touch has no hover."),
                prop("count", "u8").default("5").doc("Number of stars."),
                prop("fractions", "u8")
                    .default("1")
                    .doc("Steps per star: `1` picks whole stars, `2` halves. Each step is a hit zone of its own."),
                prop("clearable", "bool")
                    .default("false")
                    .doc("Picking the current value again resets it to 0, and Home or the arrows can go below the first step. Without it, the first step is the lowest a user can pick."),
                prop("icon", "SvgData")
                    .default("IconSlot::Star")
                    .doc("The symbol, drawn empty and filled in `currentColor`. A stroked glyph turns solid when filled; a solid one only changes colour."),
                prop("color", "ThemeAwareValue")
                    .default("warning")
                    .doc("The filled stars' colour; `theme.rating.color` when unset. Empty stars are `muted`."),
                prop("size", "Size").default("md").doc("Star size and the gap between stars."),
                prop("format", "Callback<f64, String>")
                    .doc("What a screen reader says for the value. Runs during render, so it can translate. The localization's `rating.value` (\"3.5 of 5\") when unset."),
                prop("focusable", "bool")
                    .default("true")
                    .doc("`false` only shows a value: an image named by the label and the value, with no tab stop, no pointer input and nothing posted."),
                prop("name", "FieldName<f64>")
                    .doc("What the rating posts as. A path such as `Review::FIELDS.stars()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                prop("validate", "Validators<f64>")
                    .doc("Rules over the value, shown once the rating loses focus or its form is submitted."),
                prop("label", "Caption").doc("The caption above the stars, and the slider's name."),
                prop("description", "Caption").doc("Under the label."),
                prop("helper", "Caption").doc("Under the stars."),
                prop("status", "FieldStatus")
                    .default("Valid")
                    .doc("Validation state, under the helper. A bare `&str` is an error."),
                prop("required", "bool")
                    .default("false")
                    .doc("Marks the label with an asterisk. No `aria-required`: ARIA does not allow it on a slider."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Dims the stars and drops them from the tab order."),
                prop("readonly", "bool")
                    .default("false")
                    .doc("Focusable, announced and posted, but neither pointer nor keys change it."),
                prop("aria_label", "String").doc("Names the rating when it has no `label`."),
            ])
            .parts("RatingPart", vec![
                (RatingPart::Label, "The label above the control."),
                (RatingPart::Required, "The required asterisk, in the label."),
                (RatingPart::Description, "The caption between the label and the control."),
                (RatingPart::Control, "The row of symbols."),
                (RatingPart::Symbol, "One symbol with its hit area."),
                (RatingPart::Glyph, "A symbol's empty glyph, under its fill."),
                (RatingPart::Fill, "A symbol's filled share, in the rating's colour."),
                (RatingPart::Helper, "The caption under the control."),
                (RatingPart::Status, "The validation message."),
            ])],
            accessibility: a11y()
                .key(["ArrowRight", "ArrowUp"], "One step up: a half with `fractions: 2`. Right to left, ArrowLeft goes up instead.")
                .key(["ArrowLeft", "ArrowDown"], "One step down.")
                .key(["Shift+Arrow", "PageUp", "PageDown"], "A whole star up or down.")
                .key(["Home", "End"], "The lowest step, or every star.")
                .handles([
                    "One tab stop, a `slider` with `aria-valuemin` 0, `aria-valuemax` the count and the value spoken as \"3.5 of 5\".",
                    "Hover only previews: `aria-valuenow` stays the picked value.",
                    "Filled and empty stars differ in shape as well as colour with the default stroked star.",
                    "A sideways touch drag scrubs; a vertical swipe scrolls the page.",
                    "Display-only (`focusable: false`) is an image named by the label and the value.",
                    "A debug build warns when the rating has neither a visible label nor `aria_label`.",
                ])
                .must([
                    "Without a visible label, set `aria_label`.",
                    "Translate the spoken value with the localization or `format`.",
                ])
                .limits([
                    "At the default `md` size a whole star is a 28px target, a half star 14px wide: the row is one slider target, and a drag reaches any half. `size: \"xl\"` makes each half 24px wide.",
                    "A solid custom icon shows the value by colour alone.",
                ]),
            lead: rsx! {
                Text {
                    "A row of stars picking a value, whole or in halves. Click a star, drag "
                    "across the row, or use the arrow keys. Pass "
                    Code { source: "value" }
                    " with "
                    Code { source: "onchange" }
                    ", or bind it with "
                    Code { source: "name" }
                    " in a form. "
                    Code { source: "focusable: false" }
                    " shows a value only, such as an average."
                }
            },
            // snippet: let mut stars = use_signal(|| 3.5);
            Demo {
                component: "Rating",
                children_text: "",
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::color("color"),
                    Control::toggle("fractions", ["1", "2"])
                        .labels(["Whole", "Halves"])
                        .default("2")
                        .code(|_, values| match values.str("fractions").as_str() {
                            "2" => vec!["fractions: 2".to_string()],
                            _ => vec![],
                        }),
                    Control::toggle("count", ["3", "5", "10"])
                        .default("5")
                        .code(|_, values| match values.str("count").as_str() {
                            "5" => vec![],
                            count => vec![format!("count: {count}")],
                        }),
                    // `value` + `onchange` as a pair; the preview writes back here.
                    Control::slider("value", VALUES).default("3.5").code(|_, _| {
                        vec![
                            "value: stars()".to_string(),
                            "onchange: move |value| stars.set(value)".to_string(),
                        ]
                    }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Your rating\"".to_string()],
                            _ => vec!["aria_label: \"Your rating\"".to_string()],
                        }
                    }),
                    Control::switch("clearable"),
                    Control::switch("readonly"),
                    Control::switch("disabled"),
                    Control::switch("focusable").default("true").code(|_, values| {
                        match values.str("focusable").as_str() {
                            "true" => vec![],
                            _ => vec!["focusable: false".to_string()],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Rating {
                        size: values.str("size"),
                        color: values.str("color"),
                        fractions: values.str("fractions").parse::<u8>().ok(),
                        count: values.str("count").parse::<u8>().ok(),
                        value: values.str("value").parse::<f64>().ok(),
                        onchange: {
                            let values = values.clone();
                            EventHandler::new(move |next: f64| values.set("value", next.to_string()))
                        },
                        label: (values.str("label") == "true").then(|| "Your rating".to_string()),
                        aria_label: (values.str("label") != "true").then(|| "Your rating".to_string()),
                        clearable: (values.str("clearable") == "true").then_some(true),
                        readonly: (values.str("readonly") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        focusable: (values.str("focusable") != "true").then_some(false),
                    }
                },
            }
        }
    }
}
