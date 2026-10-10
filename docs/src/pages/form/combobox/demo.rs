use super::*;

// demo-code: fruit start
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
// demo-code: fruit end

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

/// What a row draws, plain or rich - the same either side of the mode toggle.
fn row(o: &ComboboxOptionArgs<Fruit>, rich: bool) -> Element {
    rsx! {
        if rich {
            // demo-code: rich_row start
            Text { component: "span", size: "xl", "aria-hidden": "true", "{o.value.emoji()}" }
            Flex {
                direction: "column",
                justify: "center",
                align: "flex-start",
                sx: sx().gap("0"),
                Text { component: "span", size: "sm", "{o.value.label()}" }
                Text {
                    component: "span",
                    size: "xs",
                    sx: sx().color("text-dimmed"),
                    "{o.value.note()}"
                }
            }
            // demo-code: rich_row end
        } else {
            // demo-code: plain_row start
            "{o.value.label()}"
            // demo-code: plain_row end
        }
    }
}

/// A button opens the list, a pick closes it, and the pick is shown below.
#[component]
pub(super) fn SelectDemo(values: DemoValues) -> Element {
    let rich = values.str("option") == "true";
    let disabled = values.str("disabled") == "true";
    // demo-code: select_state start
    let fruit = use_combobox();
    let mut picked = use_signal(|| None::<Fruit>);
    // A `combobox` role takes no name from its content, so the label names it.
    let label = format!("{}-label", fruit.id());
    // demo-code: select_state end
    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            align: "flex-start",
            Combobox {
                size: values.str("size"),
                radius: values.str("radius"),
                disabled: disabled.then_some(true),
                state: fruit,
                options: Fruit::options().to_vec(),
                labelled_by: label.clone(),
                select_only: true,
                option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                    ComboboxOption {
                        // demo-code: select_wiring start
                        selected: picked() == Some(o.value),
                        onpick: move |_| {
                            picked.set(Some(o.value));
                            fruit.close();
                        },
                        // demo-code: select_wiring end
                        {row(&o, rich)}
                    }
                },
                // demo-code: select_trigger start
                Text { id: "{label}", size: "sm", "Fruit" }
                Button {
                    variant: "outlined",
                    sx: sx().width("280px"),
                    disabled,
                    attributes: [
                        fruit.a11y_attributes(),
                        vec![Attribute::new("aria-labelledby", label, None, false)],
                    ]
                    .concat(),
                    onclick: move |_| fruit.toggle(),
                    onblur: move |_| fruit.close(),
                    match picked() {
                        Some(fruit) => rsx! { "{fruit.label()}" },
                        None => rsx! { "Pick a fruit" },
                    }
                }
                // demo-code: select_trigger end
            }
            Text {
                size: "sm",
                sx: sx().color("text-dimmed"),
                match picked() {
                    Some(fruit) => rsx! { "Picked: {fruit.label()}" },
                    None => rsx! { "Nothing picked yet" },
                }
            }
        }
    }
}

// demo-code: search start
/// How long the fake search takes: long enough to see, short enough to type through.
const LATENCY: Duration = Duration::from_millis(700);

/// The fruit whose label contains `query`, case-insensitively and ignoring surrounding space.
fn matching(query: &str) -> Vec<Fruit> {
    let query = query.trim().to_lowercase();
    Fruit::options()
        .iter()
        .copied()
        .filter(|fruit| fruit.label().to_lowercase().contains(&query))
        .collect()
}
// demo-code: search end

#[cfg(test)]
mod tests {
    use super::matching;

    /// Todo 2434: a phone keyboard's trailing space must not hide the fruit.
    #[test]
    fn a_trailing_space_does_not_hide_a_match() {
        assert!(!matching("an").is_empty());
        assert_eq!(matching(" an ").len(), matching("an").len());
    }
}

/// Every keystroke starts a fake search that answers after [`LATENCY`], and a
/// newer keystroke cancels the older one by dropping its timer.
#[component]
pub(super) fn FetchingDemo(values: DemoValues) -> Element {
    let rich = values.str("option") == "true";
    let disabled = values.str("disabled") == "true";
    // demo-code: fetching_state start
    let suggestions = use_combobox();
    let mut text = use_signal(String::new);
    // One signal, not a list plus a `loading` flag: `None` *is* the search in
    // flight, so the two can never disagree.
    let mut results = use_signal(|| Some(Vec::<Fruit>::new()));
    // The answer still on its way. Replacing it drops the older one, so a slow
    // answer never overwrites a newer query's.
    let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
    use_drop(move || pending.set(None));
    let label = format!("{}-label", suggestions.id());
    // demo-code: fetching_state end

    rsx! {
        Combobox {
            size: values.str("size"),
            radius: values.str("radius"),
            disabled: disabled.then_some(true),
            state: suggestions,
            options: results().map(OptionList::from),
            labelled_by: label.clone(),
            loading_label: "Searching fruit",
            empty_label: "No fruit matches",
            empty: rsx! { Text { size: "sm", sx: sx().padding("xs"), "No fruit matches" } },
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        text.set(o.value.label());
                        suggestions.close();
                    },
                    {row(&o, rich)}
                }
            },
            // demo-code: fetching_trigger start
            Text { id: "{label}", size: "sm", "Fruit" }
            TextField {
                sx: sx().width("280px"),
                disabled,
                placeholder: "Type a fruit",
                value: text(),
                attributes: [
                    suggestions.a11y_attributes(),
                    vec![Attribute::new("aria-labelledby", label, None, false)],
                ]
                .concat(),
                onblur: move |_| suggestions.close(),
                oninput: move |next: String| {
                    text.set(next.clone());
                    suggestions.open();
                    results.set(None);
                    let answer = timer().map(|timer| {
                        timer.after(LATENCY, Box::new(move || results.set(Some(matching(&next)))))
                    });
                    pending.set(answer);
                },
            }
            // demo-code: fetching_trigger end
        }
    }
}

/// Typing filters the list; picking a suggestion fills the field.
#[component]
pub(super) fn SuggestionsDemo(values: DemoValues) -> Element {
    let rich = values.str("option") == "true";
    let disabled = values.str("disabled") == "true";
    // demo-code: suggestions_state start
    let suggestions = use_combobox();
    let mut text = use_signal(String::new);

    let matches: Vec<Fruit> = Fruit::options()
        .iter()
        .copied()
        .filter(|fruit| {
            fruit
                .label()
                .to_lowercase()
                .contains(&text().to_lowercase())
        })
        .collect();
    let label = format!("{}-label", suggestions.id());
    // demo-code: suggestions_state end

    rsx! {
        Combobox {
            size: values.str("size"),
            radius: values.str("radius"),
            disabled: disabled.then_some(true),
            state: suggestions,
            options: matches,
            labelled_by: label.clone(),
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    // demo-code: suggestion_wiring start
                    onpick: move |_| {
                        text.set(o.value.label());
                        suggestions.close();
                    },
                    // demo-code: suggestion_wiring end
                    {row(&o, rich)}
                }
            },
            // demo-code: suggestions_trigger start
            Text { id: "{label}", size: "sm", "Fruit" }
            TextField {
                sx: sx().width("280px"),
                disabled,
                placeholder: "Type a fruit",
                value: text(),
                attributes: [
                    suggestions.a11y_attributes(),
                    vec![Attribute::new("aria-labelledby", label, None, false)],
                ]
                .concat(),
                onblur: move |_| suggestions.close(),
                oninput: move |next| {
                    text.set(next);
                    suggestions.open();
                },
            }
            input { r#type: "hidden", name: "fruit", value: "{text()}" }
            // demo-code: suggestions_trigger end
        }
    }
}
