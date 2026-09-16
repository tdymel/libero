use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{ActionIcon, Chip, Code, FieldStatus, SelectionArgs, TagsField, Text},
    sx::sx,
};

/// The whole chip is the caller's, remove control included - so a custom tag
/// owns its own x, and with it whether that x is in the tab order.
// snippet: let mut topics = use_signal(Vec::<String>::new);
// snippet: in TagsField { value: topics(), onchange: move |next| topics.set(next), .. }
const CUSTOM_TAG: &str = r##"tag: move |t: SelectionArgs<String>| rsx! {
    Chip { size: "xs", variant: "outlined",
        trailing: rsx! {
            span { "data-slot": "remove",
                ActionIcon {
                    aria_label: "Remove {t.value}",
                    size: "xs",
                    // A `<button>` inherits no colour of its own.
                    sx: sx().color("inherit"),
                    // The input is the field's one tab stop.
                    tabindex: "-1",
                    onclick: move |_| t.remove.call(()),
                    "x"
                }
            }
        },
        "#{t.value}"
    }
}"##;

// snippet: let mut topics = use_signal(Vec::<String>::new);
// snippet: in TagsField { value: topics(), onchange: move |next| topics.set(next), .. }
const SUGGESTIONS: &str =
    r#"suggestions: vec!["rust".into(), "dioxus".into(), "wasm".into(), "css".into()]"#;

/// One tag at a time, before it joins the list. The field only announces a
/// refusal, so the helper is how the caller shows why.
// snippet: let mut topics = use_signal(Vec::<String>::new);
// snippet: let mut refused = use_signal(|| None::<String>);
// snippet: in TagsField { value: topics(), onchange: move |next| topics.set(next), .. }
const TAG_RULES: &str = r#"tag_rules: |tag: String| tag.chars().count() >= 3,
onrefuse: move |tag: String| refused.set(Some(tag)),
helper: refused().map(|tag| format!("\"{tag}\" was refused."))"#;

fn topic_tag(t: SelectionArgs<String>) -> Element {
    let label = t.value.clone();
    rsx! {
        Chip { size: "xs", variant: "outlined",
            trailing: rsx! {
                span { "data-slot": "remove",
                    ActionIcon {
                        aria_label: "Remove {label}",
                        size: "xs",
                        sx: sx().color("inherit"),
                        tabindex: "-1",
                        onclick: move |_| t.remove.call(()),
                        "x"
                    }
                }
            },
            "#{label}"
        }
    }
}

fn max_tags(values: &DemoValues) -> Option<usize> {
    values.str("max_tags").parse().ok()
}

#[component]
pub fn TagsFieldPage() -> Element {
    let mut value = use_signal(Vec::<String>::new);
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
                        .doc("Accepts or refuses one tag before it is added. A refused tag stays in the draft, and the field announces why."),
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
                        .doc("Draws one tag. A caller who overrides it draws the whole chip, remove control included - `args.remove` is the wiring. Make that control a `<button>` with `tabindex: \"-1\"`: the chip cursor focuses it. The field already keeps a click on it from taking the focus."),
                    prop("label", "Caption").doc("The field's caption, above the control."),
                    prop("description", "Caption").doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption").doc("Under the control: formatting rules, or what the entry affects."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `aria-required` and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Takes the input out of the tab order and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
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
            // snippet: let mut topics = use_signal(Vec::<String>::new);
            // snippet: let mut refused = use_signal(|| None::<String>);
            Demo {
                component: "TagsField",
                children_text: "",
                fixed: vec![
                    "sx: sx().width(\"100%\").max_width(\"320px\")".to_string(),
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
                    // Its own `code`, so the snippet prints `max_tags: 5` and
                    // not `max_tags: "5"`.
                    Control::toggle("max_tags", ["unset", "3", "5"])
                        .default("unset")
                        .code(|_, values| match max_tags(values) {
                            Some(max) => vec![format!("max_tags: {max}")],
                            None => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Topics\"".to_string()],
                            _ => vec!["aria_label: \"Topics\"".to_string()],
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
                    Control::switch("suggestions").code(|_, values| {
                        match values.str("suggestions").as_str() {
                            "true" => vec![SUGGESTIONS.to_string()],
                            _ => vec![],
                        }
                    }),
                    // Rules over one tag, not the list; the switch stands for
                    // the rule, its refusal handler and the helper that shows it.
                    Control::switch("tag_rules").code(|_, values| {
                        match values.str("tag_rules").as_str() {
                            "true" => vec![TAG_RULES.to_string()],
                            _ => vec![],
                        }
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
                render: move |values: DemoValues| {
                    let ruled = values.str("tag_rules") == "true";
                    rsx! {
                        TagsField {
                            sx: sx().width("100%").max_width("320px"),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            label: (values.str("label") == "true").then(|| "Topics".to_string()),
                            aria_label: (values.str("label") != "true").then_some("Topics"),
                            description: (values.str("description") == "true")
                                .then(|| "Comma or Enter adds one.".to_string()),
                            placeholder: (values.str("placeholder") == "true")
                                .then(|| "Add a topic".to_string()),
                            status: match values.str("status").as_str() {
                                "warning" => FieldStatus::Warning("Two of these are rarely read.".to_string()),
                                "error" => FieldStatus::Error("Add at least one topic.".to_string()),
                                _ => FieldStatus::Valid,
                            },
                            suggestions: (values.str("suggestions") == "true").then(|| {
                                ["rust", "dioxus", "wasm", "css"].map(String::from).to_vec()
                            }),
                            tag_rules: ruled
                                .then(|| Callback::new(|tag: String| tag.chars().count() >= 3)),
                            // Only the rule's refusals: with the switch off, a
                            // duplicate is refused too and nothing prints it.
                            onrefuse: move |tag: String| {
                                if ruled {
                                    refused.set(Some(tag));
                                }
                            },
                            helper: refused()
                                .filter(|_| ruled)
                                .map(|tag| format!("\"{tag}\" was refused.")),
                            tag: (values.str("tag") == "true").then(|| Callback::new(topic_tag)),
                            max_tags: max_tags(&values),
                            allow_duplicates: (values.str("allow_duplicates") == "true").then_some(true),
                            clearable: (values.str("clearable") == "true").then_some(true),
                            required: (values.str("required") == "true").then_some(true),
                            disabled: (values.str("disabled") == "true").then_some(true),
                            value: value(),
                            onchange: move |next| value.set(next),
                        }
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "The whole field is one tab stop, plus the Clear button when "
                    Code { source: "clearable" }
                    " shows it. Backspace on an empty input removes the last tag. ArrowLeft on "
                    "an empty input moves onto the tags: the arrows walk them, Delete or "
                    "Backspace removes the focused one, and ArrowRight past the last returns "
                    "to the input. A custom "
                    Code { source: "tag" }
                    " must make its remove control a button with "
                    Code { source: "tabindex: \"-1\"" }
                    ": the cursor focuses it, and without the tabindex each tag adds a tab stop."
                }
            }
        }
    }
}
