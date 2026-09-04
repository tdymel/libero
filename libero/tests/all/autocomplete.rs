use crate::common::{body, render};

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

    #[test]
    fn clearable_offers_the_x_only_while_the_field_holds_text() {
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

        assert!(body(&render(typed)).contains(r#"aria-label="Clear""#));
        assert!(!body(&render(blank)).contains(r#"aria-label="Clear""#));
    }

    /// The caller's own trailing content keeps its place when the x joins it.
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
        assert!(!html.contains(r#"aria-label="Clear""#), "{html}");
    }
}
