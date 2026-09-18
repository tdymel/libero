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
                    prop("size", "Size").default("md").doc("Height, padding and font size of the frame and its rows."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame and the list."),
                    prop("data", "Vec<CascaderOption<T>>")
                        .doc("The tree, built with `CascaderOption::new(value, label)`, `.children(..)` and `.disabled(..)`. `T` is any `Options` type. Values must be unique across the whole tree."),
                    prop("value", "Option<T>")
                        .doc("The selected option's value. Pair it with `onchange`. The cascader finds the path to it in `data`, and a value no option holds selects nothing."),
                    prop("onchange", "EventHandler<Option<T>>")
                        .doc("Called with the value to select next, or `None` when the selection was cleared."),
                    prop("any_level", "bool")
                        .default("false")
                        .doc("Lets a branch be picked as well as expanded, as its own value. Off, only a leaf commits."),
                    prop("allow_deselect", "bool")
                        .default("false")
                        .doc("Picking the selected option again clears it. Off, a re-pick keeps the value."),
                    prop("layout", "CascaderLayout")
                        .default("columns")
                        .doc("`\"columns\"` draws one list per level, `\"paths\"` one row per full path. A search always renders `\"paths\"`."),
                    prop("searchable", "bool")
                        .default("false")
                        .doc("Puts a search box at the top of the list, which narrows it to the paths that match."),
                    prop("filter", "Callback<CascaderFilterArgs<T>, bool>")
                        .doc("Narrows the paths while searching. Defaults to a case-insensitive `contains` over the joined path."),
                    prop("separator", "String")
                        .default("\" / \"")
                        .doc("Between labels, in the trigger and in a `\"paths\"` row."),
                    prop("format_value", "Callback<Vec<String>, String>")
                        .doc("Replaces the joined labels in the trigger. Gets the labels from root to option, and returns a `String` so the trigger can still cut it off with an ellipsis."),
                    prop("node", "Callback<CascaderNodeArgs<T>, Element>")
                        .default("label")
                        .doc("Draws one row's content. The highlight, chevron and click stay the component's."),
                    prop("column_width", "String")
                        .default("220px")
                        .doc("Width and minimum width of one column. A trigger wider than the open columns shares the rest among them. `\"max-content\"` fits the longest row."),
                    prop("name", "FieldName<Option<T>>")
                        .doc("Posts the selected value's `Options::value()` in a hidden input of that name. A path such as `Listing::FIELDS.category()` also binds the selection to the surrounding `Form`'s value when there is no `onchange`."),
                    prop("validate", "Validators<Option<T>>")
                        .doc("Rules over the selected value, shown once the field loses focus or its form is submitted."),
                    prop("placeholder", "String").doc("Shown while nothing is selected."),
                    prop("search_placeholder", "String").doc("What the search box says while empty."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x in place of the chevron while a value is selected."),
                    prop("label", "Caption").doc("The caption above the control, and the field's name."),
                    prop("description", "Caption").doc("Between the label and the control. What to pick."),
                    prop("helper", "Caption").doc("Under the control. What the choice changes."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Sets `aria-required` and marks the label."),
                    prop("disabled", "bool").default("false").doc("Takes the trigger out of the tab order and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Picks one option from a tree, one level at a time. The value is that option's "
                    Code { source: "value" }
                    ", any "
                    Code { source: "T" }
                    " a "
                    Code { source: "Select" }
                    " could hold. The cascader finds the path to it and shows the path in the trigger."
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
                            _ => vec!["aria_label: \"Category\"".to_string()],
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
                    Control::switch("allow_deselect"),
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
                            aria_label: (values.str("label") != "true").then_some("Category"),
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
                            allow_deselect: (values.str("allow_deselect") == "true").then_some(true),
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
                    "The arrows, Enter, Space, Home and End open the list. Inside it, ArrowUp and "
                    "ArrowDown move within a column and skip disabled rows. ArrowRight goes into "
                    "the children and ArrowLeft back up. Enter picks a leaf and expands a branch, "
                    "and with "
                    Code { source: "any_level" }
                    " it picks the branch too. Escape closes without a change. Unless the list "
                    "is searchable, Space acts like Enter and typing a letter jumps to a row. "
                    "In "
                    Code { source: "\"paths\"" }
                    ", and so while searching, ArrowLeft and ArrowRight move the search box's caret."
                }
                Text {
                    "Without a "
                    Code { source: "label" }
                    ", set "
                    Code { source: "aria_label" }
                    ". Otherwise screen readers announce an unnamed combobox."
                }
            }
        }
    }
}
