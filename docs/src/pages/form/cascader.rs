use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Cascader, CascaderOption, Code, FieldStatus, Flex, Text},
    sx::sx,
};

/// The tree the preview renders, printed above the snippet so the two cannot
/// drift - `data` is deliberately not a control.
const DATA_CODE: &str = r#"fn categories() -> Vec<CascaderOption<String>> {
    vec![
        CascaderOption::new("food", "Food").children(vec![
            CascaderOption::new("fruit", "Fruit").children(vec![
                CascaderOption::new("apple", "Apple"),
                CascaderOption::new("pear", "Pear"),
                CascaderOption::new("quince", "Quince").disabled(true),
            ]),
            CascaderOption::new("veg", "Veg").children(vec![
                CascaderOption::new("leek", "Leek"),
                CascaderOption::new("kale", "Kale"),
            ]),
        ]),
        CascaderOption::new("drink", "Drink").children(vec![
            CascaderOption::new("hot", "Hot").children(vec![
                CascaderOption::new("tea", "Tea"),
                CascaderOption::new("coffee", "Coffee"),
            ]),
            CascaderOption::new("cold", "Cold")
                .children(vec![CascaderOption::new("juice", "Juice")]),
        ]),
        CascaderOption::new("household", "Household")
            .disabled(true)
            .children(vec![CascaderOption::new("soap", "Soap")]),
    ]
}"#;

/// What a value looks like: one option's value. The path to it is the
/// cascader's to find.
const CHOSEN: &str = r#"let mut chosen = use_signal(|| Some("tea".to_string()));"#;

fn categories() -> Vec<CascaderOption<String>> {
    vec![
        CascaderOption::new("food", "Food").children(vec![
            CascaderOption::new("fruit", "Fruit").children(vec![
                CascaderOption::new("apple", "Apple"),
                CascaderOption::new("pear", "Pear"),
                CascaderOption::new("quince", "Quince").disabled(true),
            ]),
            CascaderOption::new("veg", "Veg").children(vec![
                CascaderOption::new("leek", "Leek"),
                CascaderOption::new("kale", "Kale"),
            ]),
        ]),
        CascaderOption::new("drink", "Drink").children(vec![
            CascaderOption::new("hot", "Hot").children(vec![
                CascaderOption::new("tea", "Tea"),
                CascaderOption::new("coffee", "Coffee"),
            ]),
            CascaderOption::new("cold", "Cold")
                .children(vec![CascaderOption::new("juice", "Juice")]),
        ]),
        CascaderOption::new("household", "Household")
            .disabled(true)
            .children(vec![CascaderOption::new("soap", "Soap")]),
    ]
}

/// The tree, the value's declaration, and the value printed under the field.
fn wrap_value(_: &DemoValues, code: &str) -> String {
    format!(
        "{DATA_CODE}\n\n{CHOSEN}\n\nFlex {{\n    direction: \"column\",\n    gap: \"sm\",\n    align: \"flex-start\",\n{}    Text {{ size: \"sm\", \"value: {{chosen():?}}\" }}\n}}",
        indent(code)
    )
}

#[component]
pub fn CascaderPage() -> Element {
    let mut chosen = use_signal(|| Some("tea".to_string()));

    rsx! {
        DocPage {
            title: "Cascader",
            source: "libero/src/components/form/cascader/cascader.rs",
            markdown: "/md/cascader.md",
            properties: vec![
                props("Cascader", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding and font size, of the frame and of the rows alike."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame and the list, independent of size."),
                    prop("data", "Vec<CascaderOption<T>>")
                        .doc("The tree to walk: `CascaderOption::new(value, label)`, with `.children(..)` and `.disabled(..)`. `T` is any `Options` type, as for `Select`. Values are unique across the whole tree, not just among siblings."),
                    prop("value", "Option<T>")
                        .doc("The selected option's value; strictly controlled. The cascader finds the path to it in `data`. A value no option holds selects nothing."),
                    prop("onchange", "EventHandler<Option<T>>")
                        .doc("Called with the value to select next, or `None` when the selection was cleared."),
                    prop("any_level", "bool")
                        .default("false")
                        .doc("Lets a branch be picked as well as expanded, as its own value. Off, only a leaf commits."),
                    prop("allow_deselect", "bool")
                        .default("true")
                        .doc("Picking the selected option again clears it."),
                    prop("layout", "CascaderLayout")
                        .default("columns")
                        .doc("`\"columns\"` draws one listbox per level; `\"paths\"` draws one row per full path. A search renders `\"paths\"` whatever this says."),
                    prop("searchable", "bool")
                        .default("false")
                        .doc("Puts a search box at the top of the list, which narrows it to the paths that match."),
                    prop("filter", "Callback<CascaderFilterArgs<T>, bool>")
                        .doc("Narrows the paths while searching. Defaults to a case-insensitive `contains` over the joined path."),
                    prop("separator", "String")
                        .default("\" / \"")
                        .doc("Between labels, in the trigger and in a `\"paths\"` row."),
                    prop("format_value", "Callback<Vec<String>, String>")
                        .doc("Overrides the joined labels in the trigger; takes the labels root to option. A `String` and not a node, because the value slot clips for an ellipsis."),
                    prop("node", "Callback<CascaderNodeArgs<T>, Element>")
                        .default("label")
                        .doc("Draws one row's content. The row itself - its highlight, its `aria-selected`, its chevron, its click - stays the component's."),
                    prop("column_width", "String")
                        .default("220px")
                        .doc("One column's width, and its minimum: when the trigger is wider than the open columns, they share the rest. `\"max-content\"` is how a column takes the width of its longest row."),
                    prop("validate", "Validators<Option<T>>")
                        .doc("Rules over the selected value, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<Option<T>>")
                        .doc("Emits a hidden input of that name carrying the selected value's `Options::value()`. A path also binds the selection to the surrounding `Form`'s value."),
                    prop("placeholder", "String").doc("Shown while nothing is selected."),
                    prop("search_placeholder", "String").doc("What the search box says while empty."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x in place of the chevron while a value is selected."),
                    prop("label", "Caption").doc("The field's caption, above the control."),
                    prop("description", "Caption").doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption").doc("Under the control: formatting rules, or what the entry affects."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `aria-required` and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Takes the trigger out of the tab order and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A field for choosing one option from a tree, level by level. Its value is one "
                    "option's "
                    Code { source: "value" }
                    " - any "
                    Code { source: "T" }
                    " a "
                    Code { source: "Select" }
                    " could hold. The cascader finds the path to that value itself and shows it in the "
                    "trigger. The tree is built from "
                    Code { source: "CascaderOption<T>" }
                    ", not from "
                    Code { source: "Tree" }
                    "'s nodes: a "
                    Code { source: "Tree" }
                    " expands and activates rather than selecting."
                }
            },
            Demo {
                component: "Cascader",
                children_text: "",
                fixed: vec![
                    "sx: sx().width(\"100%\").max_width(\"320px\")".to_string(),
                    "data: categories()".to_string(),
                    "value: chosen()".to_string(),
                    "onchange: move |next: Option<String>| chosen.set(next)".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("sm"),
                    Control::toggle("layout", ["columns", "paths"]).default("columns"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"That aisle is being retired.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Pick a category.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Category\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Down to a leaf.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Pick a category\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("searchable"),
                    Control::switch("any_level"),
                    Control::switch("allow_deselect").default("true"),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                wrap: Wrap(wrap_value),
                render: move |values: DemoValues| rsx! {
                    Flex { direction: "column", gap: "sm", align: "flex-start",
                        Cascader {
                            sx: sx().width("100%").max_width("320px"),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            layout: values.str("layout"),
                            label: (values.str("label") == "true").then(|| "Category".to_string()),
                            description: (values.str("description") == "true")
                                .then(|| "Down to a leaf.".to_string()),
                            placeholder: (values.str("placeholder") == "true")
                                .then(|| "Pick a category".to_string()),
                            status: match values.str("status").as_str() {
                                "warning" => FieldStatus::Warning("That aisle is being retired.".to_string()),
                                "error" => FieldStatus::Error("Pick a category.".to_string()),
                                _ => FieldStatus::Valid,
                            },
                            searchable: (values.str("searchable") == "true").then_some(true),
                            any_level: (values.str("any_level") == "true").then_some(true),
                            allow_deselect: Some(values.str("allow_deselect") == "true"),
                            clearable: (values.str("clearable") == "true").then_some(true),
                            required: (values.str("required") == "true").then_some(true),
                            disabled: (values.str("disabled") == "true").then_some(true),
                            data: categories(),
                            value: chosen(),
                            onchange: move |next: Option<String>| chosen.set(next),
                        }
                        Text { size: "sm", "value: {chosen():?}" }
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "Closed, the trigger opens on ArrowDown, ArrowUp, ArrowRight, Enter or "
                    "Space. Open, ArrowDown and ArrowUp move inside the cursor's column and skip "
                    "disabled rows, ArrowRight expands into the children, ArrowLeft goes back up "
                    "a level, Enter commits a leaf - or expands a branch, unless "
                    Code { source: "any_level" }
                    " - and Escape closes without changing the value. In "
                    Code { source: "\"paths\"" }
                    ", and so while searching, ArrowLeft and ArrowRight belong to the search "
                    "box's caret instead."
                }
            }
        }
    }
}
