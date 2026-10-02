use crate::common::{body, clear_buttons, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Autocomplete};

/// `Autocomplete` as SSR sees it: closed. The open state lives in the
/// component - there is no `state` prop to force it open with, the same limit
/// `select_listbox` documents - so the suggestions themselves are out of
/// reach here. What is testable is the control, which is where this component
/// differs from `Select`: an `<input>` that keeps its own text.
mod autocomplete_suggestions {
    use super::*;

    const CITIES: [&str; 3] = ["Amsterdam", "Berlin", "Copenhagen"];

    fn cities() -> Vec<String> {
        CITIES.iter().map(|city| city.to_string()).collect()
    }

    #[test]
    fn the_control_is_an_input_that_announces_a_list_of_suggestions() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        label: "City",
                        value: "Ber",
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("<input"), "{html}");
        assert!(html.contains(r#"role="combobox""#), "{html}");
        assert!(html.contains(r#"aria-autocomplete="list""#), "{html}");
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        // The browser's own dropdown would sit on top of ours.
        assert!(html.contains(r#"autocomplete="off""#), "{html}");
        assert!(
            html.contains(r#"value="Ber""#),
            "the text is not drawn:\n{html}"
        );
    }

    /// The whole point of the component: the value is the text, so a closed
    /// field draws no rows at all - not even the ones that match.
    #[test]
    fn a_closed_field_draws_none_of_its_suggestions() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        value: "Ber",
                        placeholder: "Where to?",
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains(r#"placeholder="Where to?""#), "{html}");
        for city in CITIES {
            assert!(
                !html.contains(&format!(">{city}<")),
                "a closed list rendered {city}:\n{html}"
            );
        }
    }

    fn typed() -> Element {
        rsx! {
            LiberoProvider {
                Autocomplete {
                    value: "Ber",
                    clearable: true,
                    options: cities(),
                    oninput: move |_| {},
                }
            }
        }
    }

    #[test]
    fn clearable_offers_the_x_only_while_the_field_holds_text() {
        fn blank() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        value: "",
                        clearable: true,
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }

        assert_eq!(clear_buttons(&body(&render(typed))).len(), 1);
        assert_eq!(clear_buttons(&body(&render(blank))).len(), 0);
    }

    /// The caller's own trailing content keeps its place when the x joins it.
    #[test]
    fn a_described_trailing_slot_leaves_the_clear_button_out() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        value: "Ber",
                        clearable: true,
                        options: cities(),
                        oninput: move |_| {},
                        trailing: rsx! { "km" },
                        describe_trailing: true,
                    }
                }
            }
        }
        let html = body(&render(app));
        let input = crate::common::attributes_of(&html, "input");
        let id = &input["id"];

        assert_eq!(input["aria-describedby"], format!("{id}-trailing"));
        // The id sits on the caller's text alone, not on the x beside it.
        assert!(
            html.contains(&format!(r#"<span id="{id}-trailing">km</span>"#)),
            "{html}"
        );
        assert_eq!(clear_buttons(&html).len(), 1, "{html}");
    }

    #[test]
    fn a_disabled_field_offers_no_clear_button() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        value: "Ber",
                        clearable: true,
                        disabled: true,
                        options: cities(),
                        oninput: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("disabled"), "{html}");
        assert_eq!(clear_buttons(&body(&render(typed))).len(), 1, "the control");
        assert_eq!(clear_buttons(&html).len(), 0, "{html}");
    }
}
