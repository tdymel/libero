use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{MultiSelect, NativeSelect, Options, Select, SelectionArgs},
};

#[derive(Clone, PartialEq, Options)]
enum Pick {
    First,
    Second,
}

#[test]
fn select_renders_its_options_and_marks_the_current_one() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect { value: Pick::Second, onchange: move |_| {} }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<select"));
    assert_eq!(body.matches("<option").count(), 2);
    // The selection is on the `<option>`, not a `value` on the `<select>`:
    // that is what SSR can express, and it needs no second render to appear.
    assert!(body.contains("<option value=\"1\" selected"));
    assert!(!body.contains("<select value="));
}

/// `value: None` is a real state - the field has not been filled in yet - so
/// it selects an entry no one can pick rather than silently taking the first.
#[test]
fn a_select_without_a_value_shows_its_placeholder() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect {
                    value: None::<Pick>,
                    placeholder: "Choose one",
                    // Annotated: with `value: None` there is nothing else for
                    // `T` to be inferred from.
                    onchange: move |_: Pick| {},
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("Choose one"));
    assert_eq!(body.matches("<option").count(), 3);
    // Disabled and hidden, so it cannot be picked back once a value is set.
    assert!(body.contains("hidden"));
    assert!(!body.contains("<option value=\"0\" selected"));
}

#[test]
fn a_disabled_select_renders_the_attribute() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect {
                    value: Pick::First,
                    disabled: true,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("disabled"));
}

/// `Select` and `MultiSelect` as SSR sees them: closed, since the open state
/// lives in the component and no test can click. The open list is the
/// Combobox's, and `combobox_highlight` covers it.
mod select_listbox {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug, Options)]
    enum Fruit {
        Apple,
        Banana,
        Cherry,
    }

    #[test]
    fn the_trigger_is_a_combobox_named_by_the_label_and_shows_the_selection() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select { label: "Fruit", value: Fruit::Banana, onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains(r#"role="combobox""#), "{html}");
        assert!(html.contains(r#"aria-haspopup="listbox""#), "{html}");
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains(r#"tabindex="0""#), "{html}");
        assert!(
            html.contains("Banana"),
            "the selection is not drawn:\n{html}"
        );
        assert!(
            !html.contains("Apple"),
            "a closed list rendered its rows:\n{html}"
        );

        let label_id = html
            .split(r#"<label id=""#)
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("the label carries an id");
        assert!(
            html.contains(&format!(r#"aria-labelledby="{label_id}""#)),
            "the trigger is not named by its label:\n{html}"
        );
    }

    #[test]
    fn nothing_selected_shows_the_placeholder() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: None::<Fruit>,
                        placeholder: "Pick a fruit",
                        onchange: move |_: Option<Fruit>| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("data-placeholder"), "{html}");
        assert!(html.contains("Pick a fruit"), "{html}");
    }

    #[test]
    fn clearable_offers_the_x_only_while_something_is_selected() {
        fn picked() -> Element {
            rsx! {
                LiberoProvider {
                    Select { value: Fruit::Apple, clearable: true, onchange: move |_| {} }
                }
            }
        }
        fn empty() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: None::<Fruit>,
                        clearable: true,
                        onchange: move |_: Option<Fruit>| {},
                    }
                }
            }
        }

        assert!(body(&render(picked)).contains(r#"aria-label="Clear""#));
        assert!(!body(&render(empty)).contains(r#"aria-label="Clear""#));
    }

    #[test]
    fn a_disabled_select_leaves_the_tab_order() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select { value: Fruit::Apple, disabled: true, onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains(r#"aria-disabled="true""#), "{html}");
        assert!(!html.contains(r#"tabindex="0""#), "{html}");
    }

    #[test]
    fn a_multi_select_draws_each_value_in_the_order_it_was_picked() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { value: vec![Fruit::Cherry, Fruit::Apple], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        let cherry = html.find("Cherry").expect("Cherry is drawn");
        let apple = html.find("Apple").expect("Apple is drawn");
        assert!(cherry < apple, "the chips lost the pick order:\n{html}");
        assert!(
            !html.contains("Banana"),
            "an unpicked value is drawn:\n{html}"
        );
        assert!(
            html.contains("multiple"),
            "the trigger lost its `multiple` state:\n{html}"
        );
    }

    #[test]
    fn every_chip_carries_a_labelled_remove_button_and_an_id_to_point_at() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { value: vec![Fruit::Cherry, Fruit::Apple], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(
            html.contains(r#"aria-label="Remove Cherry""#)
                && html.contains(r#"aria-label="Remove Apple""#),
            "a chip lost its remove button:\n{html}"
        );
        assert!(
            html.matches(r#"tabindex="-1""#).count() == 2,
            "the remove buttons are tab stops, or are missing:\n{html}"
        );
        // The ids `aria-activedescendant` points at once the cursor moves.
        assert!(
            html.matches(r#"data-slot="chip""#).count() == 2,
            "a chip lost its wrapper:\n{html}"
        );
        assert!(
            !html.contains("aria-activedescendant"),
            "a closed select with no cursor still names a descendant:\n{html}"
        );
    }

    #[test]
    fn the_chips_follow_the_field_one_step_down_the_size_scale() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { size: "lg", value: vec![Fruit::Cherry], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(
            html.contains("size-md"),
            "an `lg` field did not draw an `md` chip:\n{html}"
        );
    }

    /// The chip's inner design is the caller's, remove control included - the
    /// component adds nothing of its own around what `selection` returns.
    #[test]
    fn a_custom_selection_draws_only_what_the_caller_drew() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect {
                        value: vec![Fruit::Cherry],
                        onchange: move |_| {},
                        selection: move |args: SelectionArgs<Fruit>| rsx! {
                            span { onclick: move |_| args.remove.call(()), "{args.value.label()}!" }
                        },
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("Cherry!"), "{html}");
        assert!(
            !html.contains("aria-label=\"Remove Cherry\""),
            "the default x survived a custom selection:\n{html}"
        );
    }

    /// `searchable` must not leak a dropdown into a closed select. The core
    /// keeps an *open* list alive through a query that matches nothing - that
    /// is what the header slot is for - and a closed one draws neither.
    #[test]
    fn a_closed_searchable_select_draws_no_search_box() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: Fruit::Apple,
                        searchable: true,
                        search_placeholder: "Find a fruit",
                        onchange: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(
            !html.contains("Find a fruit"),
            "a closed select drew its search box:\n{html}"
        );
        assert!(!html.contains(r#"role="listbox""#), "{html}");
        // Closed, the trigger is still the combobox - the role only moves to
        // the search box while one exists.
        assert!(html.contains(r#"role="combobox""#), "{html}");
    }
}
