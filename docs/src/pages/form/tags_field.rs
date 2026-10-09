use super::CLEAR_NAME;
use super::dropdown_parts::{SUGGESTION_DROPDOWN, list_dropdown_parts};
use crate::components::{
    Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::TagsFieldPart;
use libero::{
    components::{
        ActionIcon, Chip, Code, FieldStatus, SelectionArgs, TagRejectReason, TagRejection,
        TagsField, Text,
    },
    hooks::use_localization,
    localization::{Localization, fill},
    sx::sx,
    use_theme,
};

/// A custom tag draws the whole chip, so it owns its x and that x's tab order.
// snippet: let mut topics = use_signal(Vec::<String>::new);
// snippet: let words = use_localization();
// snippet: in TagsField { value: topics(), onchange: move |next| topics.set(next), .. }
const CUSTOM_TAG: &str = r##"tag: move |t: SelectionArgs<String>| rsx! {
    // Unclipped, so the x's 24px hit area reaches past the pill.
    Chip { size: "xs", variant: "outlined", sx: sx().overflow("visible"),
        trailing: rsx! {
            // A disabled or read-only field's tag has no remove control.
            if !t.disabled && !t.readonly {
                span { "data-slot": "remove",
                    // Keeps focus in the field, and the click off the field.
                    onmousedown: move |event| event.prevent_default(),
                    onclick: move |event| event.stop_propagation(),
                    ActionIcon {
                        // `words` is `use_localization()`, read in the component.
                        aria_label: fill(words.common.remove, &[("label", &t.value)]),
                        size: "16px",
                        // A `<button>` inherits no colour of its own.
                        sx: sx().color("inherit"),
                        // The input is the field's one tab stop.
                        tabindex: "-1",
                        onclick: move |_| t.remove.call(()),
                        "x"
                    }
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

/// One tag at a time, before it joins the list.
// snippet: let mut topics = use_signal(Vec::<String>::new);
// snippet: in TagsField { value: topics(), onchange: move |next| topics.set(next), .. }
const TAG_RULES: &str = r#"tag_rules: |tag: String| tag.chars().count() >= 3"#;

/// Printed above the snippet while `onreject` is on. The field announces a
/// rejection to screen readers only, so the caller shows the reason on screen.
const WHY: &str = r#"/// The on-screen message for a rejected tag.
fn why(rejection: TagRejection) -> String {
    let tag = rejection.tag;
    match rejection.reason {
        TagRejectReason::Duplicate => format!("\"{tag}\" is already a topic."),
        TagRejectReason::Full => format!("\"{tag}\" was not added: the topic limit is reached."),
        TagRejectReason::NotAllowed => format!("\"{tag}\" is too short: 3 characters at least."),
    }
}

"#;

/// The demo's copy of `WHY`.
fn why(rejection: TagRejection) -> String {
    let tag = rejection.tag;
    match rejection.reason {
        TagRejectReason::Duplicate => format!("\"{tag}\" is already a topic."),
        TagRejectReason::Full => format!("\"{tag}\" was not added: the topic limit is reached."),
        TagRejectReason::NotAllowed => format!("\"{tag}\" is too short: 3 characters at least."),
    }
}

fn refusing(values: &DemoValues) -> bool {
    values.str("onreject") == "true"
}

/// The status the switch picks, as code.
fn base_status(values: &DemoValues) -> &'static str {
    match values.str("status").as_str() {
        "warning" => "FieldStatus::Warning(\"Two of these are rarely read.\".into())",
        "error" => "FieldStatus::Error(\"Add at least one topic.\".into())",
        _ => "FieldStatus::Valid",
    }
}

fn topic_tag(words: &'static Localization, t: SelectionArgs<String>) -> Element {
    let label = t.value.clone();
    let remove_label = fill(words.common.remove, &[("label", &label)]);
    rsx! {
        Chip { size: "xs", variant: "outlined", sx: sx().overflow("visible"),
            trailing: rsx! {
                if !t.disabled && !t.readonly {
                    span { "data-slot": "remove",
                        onmousedown: move |event: MouseEvent| event.prevent_default(),
                        onclick: move |event: MouseEvent| event.stop_propagation(),
                        ActionIcon {
                            aria_label: remove_label,
                            size: "16px",
                            sx: sx().color("inherit"),
                            tabindex: "-1",
                            onclick: move |_| t.remove.call(()),
                            "x"
                        }
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
    let words = use_localization();
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "TagsField",
            source: "libero/src/components/form/tags_field.rs",
            markdown: "/md/tags_field.md",
            properties: vec![
                props("TagsField", vec![
                    prop("size", "Size")
                        .default(theme.tags_field.size.as_str())
                        .doc("Height, padding, font size and the chips' size. A chip is one step smaller than the field."),
                    prop("radius", "ThemeAwareValue")
                        .default(theme.tags_field.radius.as_str())
                        .doc("Corner radius of the frame and the list, independent of `size`. Or any CSS, e.g. `radius: \"0\"`."),
                    prop("value", "Vec<String>")
                        .default("[]")
                        .doc("The tags, in order, strictly controlled. Pair it with `onchange`."),
                    prop("onchange", "EventHandler<Vec<String>>")
                        .doc("Called with the whole list the caller should hold next."),
                    prop("suggestions", "Vec<String>")
                        .doc("Adds a dropdown of tags to pick. Tags already held are not offered."),
                    prop("split_chars", "Vec<String>")
                        .default("[\",\"]")
                        .doc("Each one commits the text before it, typed or pasted."),
                    prop("allow_duplicates", "bool")
                        .default("false")
                        .doc("Lets the same tag be added twice. Off, tags are compared trimmed and case-insensitive."),
                    prop("max_tags", "usize")
                        .doc("The most tags the field accepts. A paste fills the room that is left and refuses the rest."),
                    prop("tag_rules", "Callback<String, bool>")
                        .doc("Accepts or refuses one tag before it is added. A refused tag stays in the input, and the field tells screen readers why."),
                    prop("onreject", "EventHandler<TagRejection>")
                        .doc("A tag was rejected: its `tag` and a `TagRejectReason` of `Duplicate`, `Full` (past `max_tags`) or `NotAllowed` (by `tag_rules`). Called after the same edit's `onchange`. The field shows no message, so say why on screen, such as through `status`, and clear it on the next change."),
                    prop("validate", "Validators<Vec<String>>")
                        .doc("Rules over the whole list, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<Vec<String>>")
                        .doc("Posts each tag under this name. A path such as `Article::FIELDS.topics()` also binds the list to the surrounding `Form`'s value when the field has no `onchange`."),
                    prop("placeholder", "String").doc("Shown while there are no tags."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x at the end of the frame that empties the field."),
                    prop("tag", "Callback<SelectionArgs<String>, Element>")
                        .default("Chip")
                        .doc("Draws one tag, remove control included. `args.remove` removes it. Make that control a `<button>` with `tabindex: \"-1\"`, and leave it out when `args.disabled` or `args.readonly` is set."),
                    prop("label", "Caption").doc("The field's caption, above the control."),
                    prop("description", "Caption").doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption").doc("Under the control. Formatting rules, or what the entry affects."),
                    status_prop(),
                    prop("required", "bool").default("false").doc("Marks the field required and adds an asterisk to the label. Inside a `Form`, an empty one fails the submit."),
                    prop("disabled", "bool").default("false").doc("Takes the input out of the tab order and dims the field."),
                    readonly_prop("field"),
                    prop("dropdown_parts", "Parts<DropdownPart>").doc("Styles the portaled `suggestions` dropdown and its inner parts."),
                ])
                .parts("TagsFieldPart", vec![
                    (TagsFieldPart::Label, "The label above the control."),
                    (TagsFieldPart::Required, "The required asterisk, in the label."),
                    (TagsFieldPart::Description, "The caption between the label and the control."),
                    (TagsFieldPart::Frame, "The bordered box around the control."),
                    (TagsFieldPart::Control, "The element the label names."),
                    (TagsFieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (TagsFieldPart::Tag, "One tag's chip, before the draft input."),
                    (TagsFieldPart::Helper, "The caption under the control."),
                    (TagsFieldPart::Status, "The validation message."),
                ])
                .dropdown_parts("DropdownPart", list_dropdown_parts(SUGGESTION_DROPDOWN))
                .extends("input"),
            ],
            accessibility: a11y()
                .key(["Backspace"], "On an empty input: removes the last tag.")
                .key(["Left"], "On an empty input, or with the caret before the typed text: moves onto the tags and keeps the text.")
                .key(["Left", "Right"], "On the tags: walk them. `Right` past the last returns to the input.")
                .key(["Delete", "Backspace", "Enter"], "On a tag: removes it.")
                .key(["Down", "Up"], "With `suggestions`: open the list and move the highlight.")
                .key(["PageUp", "PageDown"], "With `suggestions`, open: moves the highlight 10 rows, stopping at the first or last.")
                .key(["Enter"], "With `suggestions`: picks the highlighted row.")
                .key(["Escape"], "With `suggestions`: closes the list.")
                .handles([
                    "The whole field is one tab stop, plus the clear button when `clearable` shows it.",
                    "With `suggestions`, Android's Back button closes the list as Escape does, rather than the app.",
                    CLEAR_NAME,
                ])
                .must([
                    "A custom `tag` must make its remove control a button with `tabindex: \"-1\"`. The arrow keys focus it, and without the tabindex each tag adds a tab stop.",
                ])
                .example("A topics field, `TagsField { label: \"Topics\" }`: type a topic and press Enter to add it. Backspace on the empty input removes the last tag, and Left walks the tags, where Delete removes one."),
            lead: rsx! {
                Text {
                    "A field whose value is a list of typed strings, drawn as chips around the "
                    "input. A comma, any other "
                    Code { source: "split_chars" }
                    " entry, or Enter adds what was typed, and so does leaving the field. A tag "
                    "is trimmed first. To pick from a fixed set instead, use "
                    Code { source: "MultiSelect" }
                    ", whose value is a "
                    Code { source: "Vec<T>" }
                    " of your own type."
                }
                Text {
                    "The field turns the browser's saved entries off, so they do not cover the "
                    "suggestions. You can pass your own "
                    Code { source: "autocomplete" }
                    " token, such as "
                    Code { source: "email" }
                    " for a list of addresses, and the saved entries then cover the suggestions."
                }
            },
            // snippet: let mut topics = use_signal(Vec::<String>::new);
            // snippet: let mut refused = use_signal(|| None::<String>);
            // snippet: let words = use_localization();
            Demo {
                component: "TagsField",
                children_text: "",
                wrap: Wrap(|values: &DemoValues, source: &str| match refusing(values) {
                    true => format!("{WHY}{source}"),
                    false => source.to_string(),
                }),
                fixed: vec![
                    "sx: sx().width(\"100%\").max_width(\"320px\")".to_string(),
                    "value: topics()".to_string(),
                ],
                controls: vec![
                    // The reason shows in `status` until the next change to the list.
                    Control::switch("onreject").default("true").code(|_, values| {
                        match refusing(values) {
                            true => vec![
                                "onchange: move |next| { refused.set(None); topics.set(next) }".to_string(),
                                "onreject: move |rejection: TagRejection| refused.set(Some(why(rejection)))".to_string(),
                            ],
                            false => vec!["onchange: move |next| topics.set(next)".to_string()],
                        }
                    }),
                    Control::sizes("size").default(theme.tags_field.size.as_str()),
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"]).default(theme.tags_field.radius.as_str()),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .labels(["Valid", "Warning", "Error"])
                        .default("valid")
                        .code(|_, values| match (refusing(values), values.str("status").as_str()) {
                            (true, _) => vec![format!(
                                "status: refused().map(FieldStatus::Error).unwrap_or({})",
                                base_status(values)
                            )],
                            (false, "warning") => vec![
                                "status: FieldStatus::Warning(\"Two of these are rarely read.\".into())".to_string(),
                            ],
                            (false, "error") => vec!["status: \"Add at least one topic.\"".to_string()],
                            _ => vec![],
                        }),
                    // Its own `code`, so the snippet prints `max_tags: 5` and
                    // not `max_tags: "5"`.
                    Control::toggle("max_tags", ["unset", "3", "5"])
                        .labels(["Unset", "3", "5"])
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
                    // Rules over one tag, not the list.
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
                    let refuses = refusing(&values);
                    let status = match values.str("status").as_str() {
                        "warning" => FieldStatus::Warning("Two of these are rarely read.".to_string()),
                        "error" => FieldStatus::Error("Add at least one topic.".to_string()),
                        _ => FieldStatus::Valid,
                    };
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
                            status: refused().filter(|_| refuses).map(FieldStatus::Error).unwrap_or(status),
                            suggestions: (values.str("suggestions") == "true").then(|| {
                                ["rust", "dioxus", "wasm", "css"].map(String::from).to_vec()
                            }),
                            tag_rules: ruled
                                .then(|| Callback::new(|tag: String| tag.chars().count() >= 3)),
                            onreject: move |rejection: TagRejection| {
                                if refuses {
                                    refused.set(Some(why(rejection)));
                                }
                            },
                            tag: (values.str("tag") == "true").then(|| Callback::new(move |t| topic_tag(words, t))),
                            max_tags: max_tags(&values),
                            allow_duplicates: (values.str("allow_duplicates") == "true").then_some(true),
                            clearable: (values.str("clearable") == "true").then_some(true),
                            required: (values.str("required") == "true").then_some(true),
                            disabled: (values.str("disabled") == "true").then_some(true),
                            value: value(),
                            onchange: move |next| {
                                refused.set(None);
                                value.set(next);
                            },
                        }
                    }
                },
            }
        }
    }
}
