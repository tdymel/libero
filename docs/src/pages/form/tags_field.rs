use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Button, Chip, Code, CodeBlock, FieldStatus, Fields, Flex, Form, SelectionArgs,
        TagsField, Text,
    },
    sx::sx,
};

/// The whole chip is the caller's, remove control included - so a custom tag
/// owns its own x, and with it whether that x is in the tab order.
const CUSTOM_TAG: &str = r##"tag: move |t: SelectionArgs<String>| rsx! {
    Chip { size: "xs", variant: "outlined",
        "#{t.value}"
        span {
            ActionIcon {
                aria_label: "Remove {t.value}",
                size: "xs",
                // A `<button>` inherits no colour of its own.
                sx: sx().color("inherit"),
                onclick: move |_| t.remove.call(()),
                "x"
            }
        }
    }
}"##;

const SUGGESTIONS_CODE: &str = r#"TagsField {
    label: "Topics",
    description: "Type your own, or pick one.",
    suggestions: vec!["rust".into(), "dioxus".into(), "wasm".into(), "css".into()],
    value: topics(),
    onchange: move |next| topics.set(next),
}"#;

const RULES_CODE: &str = r#"TagsField {
    label: "Topics",
    // One tag at a time, before it joins the list. Silent: a refused tag
    // simply does not appear.
    tag_rules: move |tag: String| tag.len() >= 3,
    onrefuse: move |tag: String| refused.set(Some(tag)),
    // The ordinary field line, over the whole list.
    validate: (|tags: &Vec<String>| !tags.is_empty()).error("Add at least one topic."),
    value: topics(),
    onchange: move |next| topics.set(next),
}"#;

const FORM_CODE: &str = r#"#[derive(Clone, PartialEq, Default, Fields)]
struct Article {
    topics: Vec<String>,
}

let article = use_store(Article::default);

rsx! {
    Form {
        value: article,
        onsubmit: move |event: FormEvent| {
            // One entry per tag, all named "topics" - the visible input holds
            // the draft, so it has no name and contributes none. `get` keeps
            // the ones under that name; `FormValue` is an enum, so the text
            // comes out of it by match and not by an accessor.
            posted.set(
                event
                    .get("topics")
                    .into_iter()
                    .filter_map(|value| match value {
                        FormValue::Text(tag) => Some(tag),
                        FormValue::File(_) => None,
                    })
                    .collect(),
            );
        },
        // No `onchange`: a path binds the list to the form's own value.
        TagsField { label: "Topics", name: Article::FIELDS.topics() }
        Button { r#type: "submit", "Save" }
    }
}"#;

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Article {
    pub topics: Vec<String>,
}

/// A bound field with no handler of its own, so the form's value is the only
/// place the list lives - and the submit shows what the browser would send.
#[component]
fn TopicsForm() -> Element {
    let article = use_store(Article::default);
    let mut posted = use_signal(Vec::<String>::new);

    rsx! {
        Form {
            sx: sx().width("320px"),
            value: article,
            onsubmit: move |event: FormEvent| {
                // One entry per tag, all named "topics" - the draft input has
                // no name, so it contributes none. `FormValue` is an enum, so
                // the text comes out of it by match and not by an accessor.
                posted
                    .set(
                        event
                            .get("topics")
                            .into_iter()
                            .filter_map(|value| match value {
                                FormValue::Text(tag) => Some(tag),
                                FormValue::File(_) => None,
                            })
                            .collect(),
                    );
            },
            TagsField {
                label: "Topics",
                description: "Comma or Enter adds one.",
                placeholder: "Add a topic",
                name: Article::FIELDS.topics(),
            }
            Button { r#type: "submit", "Save" }
            if !posted().is_empty() {
                Text { "Posted as \"topics\": {posted().join(\", \")}." }
            }
        }
    }
}

fn topic_tag(t: SelectionArgs<String>) -> Element {
    let label = t.value.clone();
    rsx! {
        Chip { size: "xs", variant: "outlined",
            "#{label}"
            span { "data-slot": "remove",
                onmousedown: move |event: MouseEvent| event.prevent_default(),
                ActionIcon {
                    aria_label: "Remove {label}",
                    size: "xs",
                    sx: sx().color("inherit"),
                    onclick: move |_| t.remove.call(()),
                    "x"
                }
            }
        }
    }
}

fn max_tags(values: &DemoValues) -> Option<usize> {
    values.str("max_tags").parse().ok()
}

#[component]
pub fn TagsFieldPage() -> Element {
    let mut value = use_signal(Vec::<String>::new);
    let mut suggested = use_signal(Vec::<String>::new);
    let mut ruled = use_signal(Vec::<String>::new);
    let mut refused = use_signal(|| None::<String>);

    rsx! {
        DocPage {
            title: "TagsField",
            source: "libero/src/components/form/tags_field.rs",
            markdown: "/md/tags_field.md",
            properties: vec![
                props("TagsField", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, font size and the chips' size - a chip sits one step down the field's scale."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the frame and the list, independent of size."),
                    prop("value", "Vec<String>")
                        .default("[]")
                        .doc("The tags, in order; strictly controlled. Pair it with `onchange`."),
                    prop("onchange", "EventHandler<Vec<String>>")
                        .doc("Called with the whole list the caller should hold next."),
                    prop("suggestions", "Vec<String>")
                        .doc("Adds a dropdown of tags to pick. Absent means no dropdown at all - no listbox and no portal. Tags already held are never offered."),
                    prop("split_chars", "Vec<String>")
                        .default("[\",\"]")
                        .doc("Each one commits the text before it, on typing and on paste alike - nothing reads the clipboard."),
                    prop("allow_duplicates", "bool")
                        .default("false")
                        .doc("Lets the same tag be added twice. Off, the comparison is trimmed and case-insensitive."),
                    prop("max_tags", "usize")
                        .doc("The most tags the field accepts. Everything past it is refused one tag at a time, so a paste fills the remaining room instead of being rejected whole."),
                    prop("tag_rules", "Callback<String, bool>")
                        .doc("Accepts or refuses one tag before it is added. Never shows a message - a refusal is silent."),
                    prop("onrefuse", "EventHandler<String>")
                        .doc("A tag was refused: a duplicate, one past `max_tags`, or one `tag_rules` turned down. The only thing `onchange` cannot report."),
                    prop("validate", "Validators<Vec<String>>")
                        .doc("Rules over the whole list, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<Vec<String>>")
                        .doc("Emits one hidden input of that name per tag. A path also binds the list to the surrounding `Form`'s value."),
                    prop("placeholder", "String").doc("Shown while there are no tags."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x that empties the field, at the end of the frame."),
                    prop("tag", "Callback<SelectionArgs<String>, Element>")
                        .default("Chip")
                        .doc("Draws one tag. A caller who overrides it draws the whole chip, remove control included - `args.remove` is the wiring."),
                    prop("label", "Caption").doc("The field's caption, above the control."),
                    prop("description", "Caption").doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption").doc("Under the control: formatting rules, or what the entry affects."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `aria-required` and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Takes the input out of the tab order and dims the field."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A field whose value is a list of free-typed strings, drawn as chips with the "
                    "editor between them. A comma - or any "
                    Code { source: "split_chars" }
                    " entry - and Enter both commit what was typed; Backspace on an empty input "
                    "takes the last tag back. To choose out of a fixed set instead, reach for "
                    Code { source: "MultiSelect" }
                    ": its value is a "
                    Code { source: "Vec<T>" }
                    " over a real domain type, and it cannot be typed into."
                }
            },
            Demo {
                component: "TagsField",
                children_text: "",
                fixed: vec![
                    "sx: sx().width(\"320px\")".to_string(),
                    "value: topics()".to_string(),
                    "onchange: move |next| topics.set(next)".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"Two of these are rarely read.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Add at least one topic.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Topics\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Comma or Enter adds one.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Add a topic\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    // Its own `code`, so the snippet prints `max_tags: 5` and
                    // not `max_tags: "5"`.
                    Control::toggle("max_tags", ["unset", "3", "5"])
                        .default("unset")
                        .code(|_, values| match max_tags(values) {
                            Some(max) => vec![format!("max_tags: {max}")],
                            None => vec![],
                        }),
                    // Draws the whole chip, remove control included.
                    Control::switch("tag").code(|_, values| match values.str("tag").as_str() {
                        "true" => vec![CUSTOM_TAG.to_string()],
                        _ => vec![],
                    }),
                    Control::switch("allow_duplicates"),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    TagsField {
                        sx: sx().width("320px"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Topics".to_string()),
                        description: (values.str("description") == "true")
                            .then(|| "Comma or Enter adds one.".to_string()),
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "Add a topic".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Two of these are rarely read.".to_string()),
                            "error" => FieldStatus::Error("Add at least one topic.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        tag: (values.str("tag") == "true").then(|| Callback::new(topic_tag)),
                        max_tags: max_tags(&values),
                        allow_duplicates: (values.str("allow_duplicates") == "true").then_some(true),
                        clearable: (values.str("clearable") == "true").then_some(true),
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        value: value(),
                        onchange: move |next| value.set(next),
                    }
                },
            }
            DocSection {
                title: "Suggestions",
                Text {
                    "Without "
                    Code { source: "suggestions" }
                    " the field renders no listbox and no portal at all - it is a frame with "
                    "chips and an input. With them it is a combobox: the arrows move a "
                    "highlight, Enter picks the highlighted row, and anything already held "
                    "drops out of the list. A picked suggestion becomes exactly the tag a typed "
                    "one does, which is why the value stays a "
                    Code { source: "Vec<String>" }
                    " either way. The list stays open after a pick - the row it just took has "
                    "left it, and the next one is one key away."
                }
                Flex { direction: "column", gap: "md", align: "flex-start",
                    TagsField {
                        sx: sx().width("320px"),
                        label: "Topics",
                        description: "Type your own, or pick one.",
                        placeholder: "Add a topic",
                        suggestions: vec![
                            "rust".to_string(),
                            "dioxus".to_string(),
                            "wasm".to_string(),
                            "css".to_string(),
                        ],
                        value: suggested(),
                        onchange: move |next| suggested.set(next),
                    }
                    CodeBlock { language: "rust", source: SUGGESTIONS_CODE }
                }
            }
            DocSection {
                title: "Validating one tag",
                Text {
                    Code { source: "validate" }
                    " is the ordinary field line and rules over the whole list; "
                    Code { source: "tag_rules" }
                    " runs over a single tag before it is added, and it never shows a message. "
                    "A refusal is silent on purpose - \"you already added that\" is heavier as an "
                    "error under the field than the mistake it describes - and "
                    Code { source: "onrefuse" }
                    " is there for a caller who wants to say something anyway."
                }
                Flex { direction: "column", gap: "md", align: "flex-start",
                    TagsField {
                        sx: sx().width("320px"),
                        label: "Topics",
                        helper: match refused() {
                            Some(tag) => format!("\"{tag}\" was refused - three characters or more."),
                            None => "Tags shorter than three characters are refused.".to_string(),
                        },
                        placeholder: "Add a topic",
                        tag_rules: move |tag: String| tag.chars().count() >= 3,
                        onrefuse: move |tag: String| refused.set(Some(tag)),
                        value: ruled(),
                        onchange: move |next| ruled.set(next),
                    }
                    CodeBlock { language: "rust", source: RULES_CODE }
                }
            }
            DocSection {
                title: "Inside a Form",
                Text {
                    "The visible input holds the draft, not the value, so it carries no "
                    Code { source: "name" }
                    " - the list posts through one hidden input of that name per tag, the same "
                    "shape a native "
                    Code { source: "<select multiple>" }
                    " sends and the one "
                    Code { source: "MultiSelect" }
                    " already uses. A "
                    Code { source: "FieldName" }
                    " built from a path also binds the list to the surrounding form's value, so "
                    "a bound field needs no "
                    Code { source: "onchange" }
                    " of its own."
                }
                Flex { direction: "column", gap: "md", align: "flex-start",
                    TopicsForm {}
                    CodeBlock { language: "rust", source: FORM_CODE }
                }
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "The "
                    Code { source: "<input>" }
                    " is the labelled control, so the label names it with a plain "
                    Code { source: "for" }
                    " and the captions reach it through "
                    Code { source: "aria-describedby" }
                    ". With "
                    Code { source: "suggestions" }
                    " it is also the combobox: focus never leaves it, and "
                    Code { source: "aria-activedescendant" }
                    " is what moves over the rows."
                }
                Text {
                    "The whole field is one tab stop: each chip's x is "
                    Code { source: "tabindex=\"-1\"" }
                    ", the shape "
                    Code { source: "MultiSelect" }
                    "'s chips have, so Tab moves past the field rather than through it and "
                    "removing a chip never takes the focus with it. Backspace on an empty "
                    "input is how a keyboard takes a tag back. There is no chip cursor: the "
                    "arrows belong to the text, and the platform cannot report a caret, so "
                    "\"ArrowLeft only at caret 0\" is not implementable."
                }
            }
        }
    }
}
