use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Combobox, ComboboxTarget, Flex, OptionLabel, Options, Text},
    sx::sx,
};

/// The enum is the option list, so the snippet has to show it.
const FRUIT_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    #[option(label = "Dragon fruit")]
    Dragon,
    Elderberry,
    Mango,
    Grape,
}

"#;

const TARGET: &str = r#"target: move |t: ComboboxTarget| rsx! {
    Button {
        variant: "outlined",
        sx: sx().width("280px"),
        onclick: t.onclick,
        onkeydown: t.onkeydown,
        attributes: t.aria,
        match t.labels.first() {
            Some(label) => label.render(),
            None => rsx! { "Pick a fruit" },
        }
    }
}"#;

const OPTION_LABEL: &str = r#"option_label: move |fruit: Fruit| OptionLabel::rich(
    fruit.label(),
    rsx! {
        Text { component: "span", size: "xl", "{fruit.emoji()}" }
        Flex {
            direction: "column",
            justify: "center",
            align: "flex-start",
            // Gives the row its height, so the pitch below is not a guess.
            sx: sx().gap("0").height("56px"),
            Text { component: "span", size: "sm", "{fruit.label()}" }
            Text {
                component: "span",
                size: "xs",
                sx: sx().color("grey.6"),
                "{fruit.note()}"
            }
        }
    },
)"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    #[option(label = "Dragon fruit")]
    Dragon,
    Elderberry,
    Mango,
    Grape,
}

impl Fruit {
    fn emoji(self) -> &'static str {
        match self {
            Self::Apple => "🍎",
            Self::Banana => "🍌",
            Self::Cherry => "🍒",
            Self::Dragon => "🐉",
            Self::Elderberry => "🫐",
            Self::Mango => "🥭",
            Self::Grape => "🍇",
        }
    }

    /// The second line of a rich option row.
    fn note(self) -> &'static str {
        match self {
            Self::Apple => "Crisp, keeps for weeks",
            Self::Banana => "Ripens on the counter",
            Self::Cherry => "In season for a fortnight",
            Self::Dragon => "Mild, mostly texture",
            Self::Elderberry => "Cook it, never raw",
            Self::Mango => "Ripe when it gives to a thumb",
            Self::Grape => "Sweetest straight off the vine",
        }
    }
}

/// The rich row's height: set on the row's content so it is a fact rather than
/// whatever the two lines happen to measure, and handed to `option_height` so
/// virtualization works to the same number.
const RICH_OPTION_HEIGHT: f64 = 56.0;

#[component]
pub fn ComboboxPage() -> Element {
    let mut value = use_signal(|| None::<Fruit>);

    rsx! {
        DocPage {
            title: "Combobox",
            source: "libero/src/components/inputs/combobox",
            markdown: "/md/combobox.md",
            properties: vec![
                props("Combobox", vec![
                    prop("value", "Option<T>")
                        .doc("The selected option; strictly controlled."),
                    prop("onchange", "EventHandler<Option<T>>")
                        .doc("Called with what should be selected next. `None` clears the selection."),
                    prop("target", "Callback<ComboboxTarget, Element>")
                        .doc("The whole control. Required - `Combobox` renders no field of its own."),
                    prop("options", "Vec<T>")
                        .default("T::options()")
                        .doc("The options to show."),
                    prop("option_label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Overrides `Options::label`. Returns an `OptionLabel`, so a row can hold an icon or a badge."),
                    prop("searchable", "bool")
                        .default("true")
                        .doc("A search field above the options. `false` leaves a plain listbox."),
                    prop("search_placeholder", "String")
                        .doc("Placeholder for the search field inside the dropdown."),
                    prop("filter", "Callback<ComboboxFilterArgs<T>, bool>")
                        .doc("Whether an option survives the query. Defaults to a case-insensitive contains on the resolved label."),
                    prop("empty", "Element")
                        .doc("Shown in place of the list when nothing matches."),
                    prop("size", "Size")
                        .default("md")
                        .doc("Rows, the search field, and the row height virtualization assumes."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("The dropdown's corner radius."),
                    prop("option_height", "f64")
                        .doc("A custom row's real height in px. Rows are virtualized against the themed row height, which a taller `option_label` outgrows."),
                    prop("max_dropdown_height", "ThemeAwareValue")
                        .default("260px")
                        .doc("Height past which the option list scrolls."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Blocks opening. Pass it to the target too, which draws its own dimmed state."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A listbox over an enum, with its own search field "
                    "inside the dropdown. The control is the caller's, through "
                    Code { source: "target" }
                    " - so the selected option is displayed exactly as the caller draws "
                    "it, and "
                    Code { source: "Combobox" }
                    " never owns a field's styling. Rows are virtualized, so a list of "
                    "thousands costs the same as a list of ten."
                }
            },
            Demo {
                component: "Combobox",
                children_text: "",
                wrap: Wrap(|_: &DemoValues, source: &str| format!("{FRUIT_ENUM}{source}")),
                fixed: vec![
                    "value: value()".to_string(),
                    "onchange: move |next| value.set(next)".to_string(),
                    TARGET.to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::switch("searchable").default("true"),
                    Control::switch("option_label").code(|_, values| {
                        match values.str("option_label").as_str() {
                            "true" => vec![
                                OPTION_LABEL.to_string(),
                                format!("option_height: {RICH_OPTION_HEIGHT}"),
                            ],
                            _ => vec![],
                        }
                    }),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Combobox {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        searchable: values.str("searchable") == "true",
                        option_label: (values.str("option_label") == "true").then(|| {
                            Callback::new(|fruit: Fruit| {
                                OptionLabel::rich(
                                    fruit.label(),
                                    rsx! {
                                        Text {
                                            component: "span",
                                            size: "xl",
                                            "{fruit.emoji()}"
                                        }
                                        Flex {
                                            direction: "column",
                                            justify: "center",
                                            align: "flex-start",
                                            sx: sx()
                                                .gap("0")
                                                .height(format!("{RICH_OPTION_HEIGHT}px")),
                                            Text {
                                                component: "span",
                                                size: "sm",
                                                "{fruit.label()}"
                                            }
                                            Text {
                                                component: "span",
                                                size: "xs",
                                                sx: sx().color("grey.6"),
                                                "{fruit.note()}"
                                            }
                                        }
                                    },
                                )
                            })
                        }),
                        option_height: (values.str("option_label") == "true")
                            .then_some(RICH_OPTION_HEIGHT),
                        search_placeholder: "Search fruit",
                        disabled: (values.str("disabled") == "true").then_some(true),
                        value: value(),
                        onchange: move |next| value.set(next),
                        target: move |t: ComboboxTarget| rsx! {
                            Button {
                                variant: "outlined",
                                sx: sx().width("280px"),
                                disabled: t.disabled,
                                onclick: move |event| t.onclick.call(event),
                                onkeydown: move |event| t.onkeydown.call(event),
                                attributes: t.aria.clone(),
                                match t.labels.first() {
                                    Some(label) => label.render(),
                                    None => rsx! { "Pick a fruit" },
                                }
                            }
                        },
                    }
                },
            }
        }
    }
}
