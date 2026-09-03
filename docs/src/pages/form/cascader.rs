use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Cascader, CascaderPick, Code, FieldStatus, Flex, Text, TreeNode},
    sx::sx,
};

/// The tree the preview renders, printed above the snippet so the two cannot
/// drift - `data` is deliberately not a control.
const DATA_CODE: &str = r#"fn categories() -> Vec<TreeNode<&'static str>> {
    vec![
        TreeNode::new("food", "Food").children(vec![
            TreeNode::new("fruit", "Fruit").children(vec![
                TreeNode::new("apple", "Apple"),
                TreeNode::new("pear", "Pear"),
                TreeNode::new("quince", "Quince").disabled(true),
            ]),
            TreeNode::new("veg", "Veg").children(vec![
                TreeNode::new("leek", "Leek"),
                TreeNode::new("kale", "Kale"),
            ]),
        ]),
        TreeNode::new("drink", "Drink").children(vec![
            TreeNode::new("hot", "Hot").children(vec![
                TreeNode::new("tea", "Tea"),
                TreeNode::new("coffee", "Coffee"),
            ]),
            TreeNode::new("cold", "Cold").children(vec![TreeNode::new("juice", "Juice")]),
        ]),
        TreeNode::new("household", "Household").disabled(true).children(vec![
            TreeNode::new("soap", "Soap"),
        ]),
    ]
}"#;

/// What a value looks like: the ids from the root to the picked node.
const CHOSEN: &str = r#"let mut chosen = use_signal(|| vec!["drink".to_string(), "hot".to_string(), "tea".to_string()]);"#;

fn categories() -> Vec<TreeNode<&'static str>> {
    vec![
        TreeNode::new("food", "Food").children(vec![
            TreeNode::new("fruit", "Fruit").children(vec![
                TreeNode::new("apple", "Apple"),
                TreeNode::new("pear", "Pear"),
                TreeNode::new("quince", "Quince").disabled(true),
            ]),
            TreeNode::new("veg", "Veg").children(vec![
                TreeNode::new("leek", "Leek"),
                TreeNode::new("kale", "Kale"),
            ]),
        ]),
        TreeNode::new("drink", "Drink").children(vec![
            TreeNode::new("hot", "Hot").children(vec![
                TreeNode::new("tea", "Tea"),
                TreeNode::new("coffee", "Coffee"),
            ]),
            TreeNode::new("cold", "Cold").children(vec![TreeNode::new("juice", "Juice")]),
        ]),
        TreeNode::new("household", "Household")
            .disabled(true)
            .children(vec![TreeNode::new("soap", "Soap")]),
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
    let mut chosen = use_signal(|| vec!["drink".to_string(), "hot".to_string(), "tea".to_string()]);

    rsx! {
        DocPage {
            title: "Cascader",
            source: "libero/src/components/form/cascader/cascader.rs",
            markdown: "/md/cascader.md",
            properties: vec![
                props("Cascader", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding and font size, of the frame and of the rows alike."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame and the list, independent of size."),
                    prop("data", "Vec<TreeNode<T>>")
                        .doc("The tree to walk - `Tree`'s own node type, so a caller who already has one drops it straight in. Ids are unique across the whole tree, not just among siblings."),
                    prop("value", "Vec<String>")
                        .doc("The selected path's ids, root to leaf; strictly controlled. A path that is not in `data` selects nothing."),
                    prop("onchange", "EventHandler<CascaderPick<T>>")
                        .doc("Called with the path to select next and the nodes on it. An empty `path` means the selection was cleared."),
                    prop("any_level", "bool")
                        .default("false")
                        .doc("Lets a branch be picked as well as expanded. Off, only a leaf commits."),
                    prop("allow_deselect", "bool")
                        .default("true")
                        .doc("Picking the committed path again clears it."),
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
                    prop("format_value", "Callback<Vec<T>, String>")
                        .doc("Overrides the joined labels in the trigger. A `String` and not a node, because the value slot clips for an ellipsis."),
                    prop("node", "Callback<CascaderNodeArgs<T>, Element>")
                        .default("tree_label()")
                        .doc("Draws one row's content. The row itself - its highlight, its `aria-selected`, its chevron, its click - stays the component's."),
                    prop("column_width", "String")
                        .default("220px")
                        .doc("One column's width, and its minimum: when the trigger is wider than the open columns, they share the rest. `\"max-content\"` is how a column takes the width of its longest row."),
                    prop("validate", "Validators<Vec<String>>")
                        .doc("Rules over the path, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<Vec<String>>")
                        .doc("Emits one hidden input of that name per level. A path also binds the selection to the surrounding `Form`'s value."),
                    prop("placeholder", "String").doc("Shown while nothing is selected."),
                    prop("search_placeholder", "String").doc("What the search box says while empty."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x in place of the chevron while a path is selected."),
                    prop("label", "Caption").doc("The field's caption, above the control."),
                    prop("description", "Caption").doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption").doc("Under the control: formatting rules, or what the entry affects."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `aria-required` and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Takes the trigger out of the tab order and dims the field."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A field for choosing one branch of a tree, level by level. Its value is the "
                    Code { source: "Vec<String>" }
                    " path from the root to the picked node, which is what makes it neither a "
                    Code { source: "Select" }
                    " - whose value is one "
                    Code { source: "T" }
                    " - nor a "
                    Code { source: "Tree" }
                    ", which expands and activates rather than selecting. It takes "
                    Code { source: "Tree" }
                    "'s own "
                    Code { source: "TreeNode<T>" }
                    ", so the same data drives both."
                }
            },
            Demo {
                component: "Cascader",
                children_text: "",
                fixed: vec![
                    "sx: sx().width(\"320px\")".to_string(),
                    "data: categories()".to_string(),
                    "value: chosen()".to_string(),
                    "onchange: move |pick: CascaderPick<&'static str>| chosen.set(pick.path)".to_string(),
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
                            sx: sx().width("320px"),
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
                            onchange: move |pick: CascaderPick<&'static str>| chosen.set(pick.path),
                        }
                        Text { size: "sm", "value: {chosen():?}" }
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "Focus never leaves the trigger. Each column is its own "
                    Code { source: "role=\"listbox\"" }
                    " whose rows are never focused, and the trigger names the row the arrows are "
                    "on with "
                    Code { source: "aria-activedescendant" }
                    " - the pattern "
                    Code { source: "Select" }
                    " and "
                    Code { source: "Combobox" }
                    " already use, one cursor over several lists instead of one. A column past "
                    "the first is named by the row it hangs off, so nothing has to invent a "
                    "label for \"level 2\"."
                }
                Text {
                    Code { source: "aria-selected" }
                    " follows the cursor's own chain rather than the committed path, so the row "
                    Code { source: "aria-activedescendant" }
                    " points at always carries it - with several columns open those are "
                    "routinely different rows. The committed path keeps a mark of its own, in "
                    "weight rather than in ARIA."
                }
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
