use super::CLEAR_NAME;
use super::dropdown_parts::{CASCADER_DROPDOWN, list_dropdown_parts};
use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, Wrap, a11y, field_controls, field_props, indent,
    prop, props, readonly_prop, required_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::CascaderPart;
use libero::use_theme;
use libero::{
    components::{Cascader, CascaderNodeArgs, CascaderOption, Code, Flex, Text},
    sx::sx,
};

struct CategoryCopy;

impl FieldCopy for CategoryCopy {
    const LABEL: &'static str = "Category";
    const DESCRIPTION: &'static str = "Down to a leaf.";
    const HELPER: &'static str = "Shown on the product page.";
    const WARNING: &'static str = "That aisle is being retired.";
    const ERROR: &'static str = "Pick a category.";
}

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

/// Uses every `CascaderNodeArgs` flag: an icon on the roots, the expanded branch bold, a check on
/// the committed option.
// snippet: after DATA_CODE
// snippet: let mut chosen = use_signal(|| Some("tea".to_string()));
// snippet: in Cascader { data: categories(), value: chosen(), onchange: move |next: Option<String>| chosen.set(next), aria_label: "Category", .. }
const CUSTOM_NODE: &str = r#"node: move |n: CascaderNodeArgs<String>| {
    let icon = match n.value.as_str() {
        "food" => "🍽️",
        "drink" => "🥤",
        _ => "🧽",
    };
    rsx! {
        // Decoration, hidden from screen readers: the row reads as its label.
        if n.level == 0 {
            span { "aria-hidden": "true", "{icon} " }
        }
        Text {
            component: "span",
            sx: sx().font_weight(if n.expanded { "600" } else { "inherit" }),
            "{n.label}"
        }
        if n.selected {
            span { "aria-hidden": "true", " ✓" }
        }
    }
}"#;

/// The demo's `node`, as `CUSTOM_NODE` prints it.
fn category_node(n: CascaderNodeArgs<String>) -> Element {
    let icon = match n.value.as_str() {
        "food" => "🍽️",
        "drink" => "🥤",
        _ => "🧽",
    };
    rsx! {
        if n.level == 0 {
            span { "aria-hidden": "true", "{icon} " }
        }
        Text {
            component: "span",
            sx: sx().font_weight(if n.expanded { "600" } else { "inherit" }),
            "{n.label}"
        }
        if n.selected {
            span { "aria-hidden": "true", " ✓" }
        }
    }
}

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
    let theme = use_theme();
    let mut chosen = use_signal(|| Some("tea".to_string()));

    rsx! {
        DocPage {
            title: "Cascader",
            source: "libero/src/components/form/cascader/cascader.rs",
            markdown: "/md/cascader.md",
            properties: vec![
                props("Cascader", vec![
                    prop("size", "Size").default(theme.cascader.size.as_str()).doc("Height, padding and font size of the frame and its rows."),
                    prop("radius", "Size").default(theme.cascader.radius.as_str()).doc("Corner radius of the frame and the list."),
                    prop("data", "Vec<CascaderOption<T>>").default("required")
                        .doc("The tree, built with `CascaderOption::new(value, label)`, `.children(..)` and `.disabled(..)`. `T` is any `Options` type. Values must be unique across the whole tree."),
                    prop("value", "Option<T>")
                        .doc("The selected option's value. Pair it with `onchange`. The cascader finds the path to it in `data`, and a value no option holds selects nothing there: the trigger shows its `Options::label()` and the form still posts it, as `Select` does."),
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
                        .doc("`\"columns\"` draws one list per level, `\"paths\"` one row per full path. A search always renders `\"paths\"`. Without `any_level` it lists only leaf paths. On a screen narrower than the `sm` breakpoint, `\"columns\"` shows one level at a time under a back header."),
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
                        .default(theme.cascader.column_width)
                        .doc("Width and minimum width of one column. A trigger wider than the open columns shares the rest among them. `\"max-content\"` fits the longest row. On a narrow screen, the minimum width of the one level shown."),
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
                    status_prop(),
                    required_prop().also("Inside a `Form`, an empty one fails the submit."),
                    prop("disabled", "bool").default("false").doc("Takes the trigger out of the tab order and dims the field."),
                    readonly_prop("field"),
                    prop("dropdown_parts", "Parts<DropdownPart>").doc("Styles the portaled dropdown and its inner parts."),
                ])
                .parts("CascaderPart", vec![
                    (CascaderPart::Label, "The label above the control."),
                    (CascaderPart::Required, "The required asterisk, in the label."),
                    (CascaderPart::Description, "The caption between the label and the control."),
                    (CascaderPart::Frame, "The bordered box around the control."),
                    (CascaderPart::Control, "The element the label names."),
                    (CascaderPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (CascaderPart::Value, "The joined path or the placeholder, in the trigger."),
                    (CascaderPart::Helper, "The caption under the control."),
                    (CascaderPart::Status, "The validation message."),
                ])
                .dropdown_parts("DropdownPart", list_dropdown_parts(CASCADER_DROPDOWN)),
                props("CascaderNodeArgs", vec![
                    prop("value", "T").doc("The option this row draws."),
                    prop("label", "String").doc("The option's label."),
                    prop("level", "usize").doc("The option's column: 0 for a root."),
                    prop("expanded", "bool").doc("The column to the right holds this option's children."),
                    prop("selected", "bool").doc("This option holds the committed value."),
                ])
                .without_base_props(),
                props("CascaderFilterArgs", vec![
                    prop("query", "String").doc("What is typed in the search box."),
                    prop("label", "String").doc("The path's labels joined by `separator`, what the default filter matches."),
                    prop("path", "Vec<T>").doc("The values from the root to the option. The last one is what this path commits."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Down", "Up", "Right", "Enter", "Space"], "Closed: opens on the committed path, or with the cursor on the first enabled root (`Up`: the last).")
                .key(["Home", "End"], "Closed: opens with the cursor on the first or last enabled root. Open: moves to the first or last enabled row of the column.")
                .key(["Letter"], "Unless searchable: moves to the next enabled row of the column starting with the typed text. A closed list opens on the roots.")
                .key(["Down", "Up"], "Open: moves within the column, skipping disabled rows.")
                .key(["Right"], "Open, in `\"columns\"`: expands the row, the cursor onto its first enabled child.")
                .key(["Left"], "Open, in `\"columns\"`: up one level. At the root, nothing.")
                .key(["Enter"], "Open, on a leaf: picks it and closes. On the committed leaf it keeps the value, or clears it with `allow_deselect`.")
                .key(["Enter"], "Open, on a branch: expands it. Picks it too, with `any_level`.")
                .key(["Space"], "Open, unless searchable: as `Enter`.")
                .key(["Tab", "Alt+Up"], "Open: picks the cursor's row if `Enter` would, and closes. `Tab` moves on.")
                .key(["Escape"], "Open: closes and keeps the value.")
                .handles([
                    "In `\"paths\"`, and so while searching, `Left` and `Right` move the search box's caret.",
                    "Below the `sm` breakpoint (48rem), `\"columns\"` shows only the cursor's level. A header names its parent, and its back button (\"Back to Europe\", `Localization::cascader.back`) goes up one level, as `Left` does. It never takes focus, so focus stays on the trigger. With `any_level`, a first row \"Select Europe\" picks the parent and closes. The keyboard cursor does not reach that row: `Left`, then `Enter` on the parent, picks it.",
                    "Below the `sm` breakpoint the dropdown is a full-width sheet at the foot of the screen. It is not modal: no backdrop, no focus trap, and `Escape` or a press outside closes it. Opening it scrolls the page, as far as it can, so the trigger stays above the sheet.",
                    "Android's Back button closes the dropdown as Escape does, rather than the app.",
                    CLEAR_NAME,
                ])
                .must(["Without a `label`, set `aria_label`. Otherwise screen readers announce an unnamed combobox."])
                .example("A category picker, `Cascader { label: \"Category\" }`: Down opens it, Right expands a branch, Left goes up a level, Enter on a leaf picks it and closes, and Escape closes without a change."),
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
                controls: [vec![
                    Control::sizes("size").default(theme.cascader.size.as_str()),
                    Control::sizes("radius").default(theme.cascader.radius.as_str()),
                    Control::toggle("layout", ["columns", "paths"])
                        .labels(["Columns", "Paths"])
                        .default("columns"),
                ], field_controls::<CategoryCopy>(), vec![
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Pick a category\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    // Draws the rows through `node`.
                    Control::switch("node").code(|_, values| match values.str("node").as_str() {
                        "true" => vec![CUSTOM_NODE.to_string()],
                        _ => vec![],
                    }),
                    Control::switch("searchable"),
                    Control::switch("any_level"),
                    Control::switch("allow_deselect"),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ]].concat(),
                wrap: Wrap(wrap_value),
                render: move |values: DemoValues| {
                    let field = field_props::<CategoryCopy>(&values);
                    rsx! {
                        Flex { direction: "column", gap: "sm", align: "flex-start",
                            Cascader {
                                sx: sx().width("100%").max_width("320px"),
                                size: values.str("size"),
                                radius: values.str("radius"),
                                layout: values.str("layout"),
                                label: field.label,
                                aria_label: field.aria_label,
                                description: field.description,
                                placeholder: (values.str("placeholder") == "true")
                                    .then(|| "Pick a category".to_string()),
                                helper: field.helper,
                                status: field.status,
                                node: (values.str("node") == "true").then(|| Callback::new(category_node)),
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
                    }
                },
            }
        }
    }
}
