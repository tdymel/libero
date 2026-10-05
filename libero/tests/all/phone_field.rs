//! `PhoneField`'s rendered contract: the `tel` input is the labelled control
//! and holds the *text*, a hidden input posts the E.164, and the picker is a
//! real button that only exists while `country_select` is on.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::PhoneField};

fn typed() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                country: "US",
                value: "+12133734253",
                oninput: move |_: String| {},
            }
        }
    }
}

fn posting() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                country: "US",
                name: "phone",
                value: "+12133734253",
                oninput: move |_: String| {},
            }
        }
    }
}

fn pinned() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                country: "DE",
                country_select: false,
                oninput: move |_: String| {},
            }
        }
    }
}

fn elsewhere() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField {
                label: "Mobile",
                country: "US",
                value: "+4930123456",
                oninput: move |_: String| {},
            }
        }
    }
}

/// The control is an `<input>`, which `<label for>` can name outright - so this
/// field takes `use_field()` plain rather than `labelled_by()`.
#[test]
fn the_tel_input_is_the_labelled_control() {
    let html = body(&render(typed));

    let input = attributes_of(&html, "input");
    let id = input.get("id").expect("the input carries the field's id");
    assert_eq!(input.get("type").map(String::as_str), Some("tel"));
    assert_eq!(input.get("inputmode").map(String::as_str), Some("tel"));
    assert!(
        html.contains(&format!("for=\"{id}\"")),
        "the label does not name the input:\n{html}"
    );
    assert!(
        !html.contains("aria-labelledby"),
        "a labelable control should not be named the other way round:\n{html}"
    );
}

/// The visible input holds the national text, so it must not carry the `name`:
/// it would post what is on screen instead of the value.
#[test]
fn the_hidden_input_posts_the_e164_and_the_visible_one_holds_the_text() {
    let html = body(&render(posting));

    let visible = attributes_of(&html, "input");
    assert_eq!(
        visible.get("value").map(String::as_str),
        Some("213 373 4253")
    );
    assert!(
        !visible.contains_key("name"),
        "the visible input posts the display text:\n{html}"
    );
    assert!(
        html.contains(r#"type="hidden" name="phone" value="+12133734253""#),
        "no hidden input carrying the E.164:\n{html}"
    );
}

/// An existing number renders under its own country, whatever the field was
/// asked to start on - `country` is the initial pick, not an override.
#[test]
fn a_value_from_another_country_renders_under_that_country() {
    let html = body(&render(elsewhere));

    assert_eq!(
        attributes_of(&html, "input")
            .get("value")
            .map(String::as_str),
        Some("30123456")
    );
    assert!(html.contains(">DE<"), "the picker still says US:\n{html}");
    assert!(
        html.contains(">+49<"),
        "the dial code did not follow:\n{html}"
    );
}

/// The picker is a second tab stop, and should be - it is the only way to
/// reach what it does. `type="button"` because a button in a form submits it
/// otherwise.
#[test]
fn the_picker_is_a_button_that_says_a_listbox_hangs_off_it() {
    let html = body(&render(typed));

    let button = attributes_of(&html, "button");
    assert_eq!(button.get("type").map(String::as_str), Some("button"));
    assert_eq!(
        button.get("aria-haspopup").map(String::as_str),
        Some("listbox")
    );
    assert_eq!(
        button.get("aria-expanded").map(String::as_str),
        Some("false")
    );
    // The content reads `US +1`, which names a code and not a country; the name
    // keeps it for speech input (2.5.3).
    assert_eq!(
        button.get("aria-label").map(String::as_str),
        Some("Country: United States, US +1")
    );
    // Closed, there is no list to point at and nothing to announce.
    assert!(
        !html.contains("role=\"listbox\""),
        "a closed picker rendered its list:\n{html}"
    );
    assert!(!button.contains_key("aria-disabled"), "{html}");
}

fn locked() -> Element {
    rsx! {
        LiberoProvider {
            PhoneField { label: "Mobile", country: "US", readonly: true }
        }
    }
}

/// Read-only keeps the picker a tab stop but says it does nothing (todo 558).
#[test]
fn a_read_only_picker_is_aria_disabled_and_still_focusable() {
    let html = body(&render(locked));

    let button = attributes_of(&html, "button");
    assert_eq!(
        button.get("aria-disabled").map(String::as_str),
        Some("true")
    );
    assert!(!button.contains_key("disabled"), "{html}");
}

/// `country_select: false` pins the country and is one tab stop fewer: the
/// leading slot becomes a plain `+49` with nothing to open.
#[test]
fn a_pinned_country_renders_no_picker_at_all() {
    let html = body(&render(pinned));

    assert!(
        !html.contains("<button"),
        "the pinned field still has a picker:\n{html}"
    );
    assert!(html.contains("+49"), "the dial code is not drawn:\n{html}");
}

/// Todo 559: the bare `+49` is visible text beside the input, so the input's
/// description carries it, ahead of the field's own captions (2.5.3).
#[test]
fn a_pinned_dial_code_describes_the_input() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PhoneField {
                    label: "Mobile",
                    description: "Work number",
                    country: "DE",
                    country_select: false,
                    oninput: move |_: String| {},
                }
            }
        }
    }

    let html = body(&render(app));
    let input = attributes_of(&html, "input");
    let ids = input
        .get("aria-describedby")
        .unwrap_or_else(|| panic!("the input has no description:\n{html}"));
    let texts: Vec<&str> = ids
        .split(' ')
        .map(|id| {
            let open = format!("id=\"{id}\"");
            let at = html
                .find(&open)
                .unwrap_or_else(|| panic!("no #{id}:\n{html}"));
            let from = at + html[at..].find('>').unwrap() + 1;
            let len = html[from..].find('<').unwrap();
            &html[from..from + len]
        })
        .collect();
    assert_eq!(texts, ["+49", "Work number"], "{html}");
}

static PINNED: libero::theme::Theme = libero::theme::Theme {
    phone_field: libero::theme::PhoneFieldDefaults {
        country_select: false,
        ..libero::theme::Theme::DEFAULT.phone_field
    },
    ..libero::theme::Theme::DEFAULT
};

/// An unset `country_select` takes the theme's.
#[test]
fn an_unset_country_select_follows_the_theme() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { themes: &PINNED,
                PhoneField { label: "Mobile", country: "DE", oninput: move |_: String| {} }
            }
        }
    }

    let html = body(&render(app));
    assert!(!html.contains("<button"), "{html}");
    assert!(html.contains("+49"), "{html}");
}

/// Todo 724: the country's name comes from the localization, a code it lacks
/// keeps the English name, and `country_label` wins over both.
#[test]
fn the_country_name_follows_the_localization() {
    use libero::localization::{Localization, PhoneFieldLabels};
    static SPARSE: Localization = Localization {
        phone_field: PhoneFieldLabels {
            country_names: &[("AT", "Österreich")],
            ..PhoneFieldLabels::GERMAN
        },
        ..Localization::GERMAN
    };
    fn field(localization: &'static Localization, country: &'static str) -> Element {
        rsx! {
            LiberoProvider { localization,
                PhoneField { label: "Mobil", country, oninput: move |_: String| {} }
            }
        }
    }
    let label = |app: fn() -> Element| {
        let html = body(&render(app));
        attributes_of(&html, "button")
            .get("aria-label")
            .cloned()
            .unwrap_or_else(|| panic!("no picker name:\n{html}"))
    };

    assert_eq!(
        label(|| field(&Localization::GERMAN, "DE")),
        "Land: Deutschland, DE +49"
    );
    assert_eq!(label(|| field(&SPARSE, "US")), "Land: United States, US +1");
    assert_eq!(
        label(|| rsx! {
            LiberoProvider { localization: &Localization::GERMAN,
                PhoneField {
                    label: "Mobil",
                    country: "DE",
                    country_label: move |iso: String| format!("[{iso}]"),
                    oninput: move |_: String| {},
                }
            }
        }),
        "Land: [DE], DE +49"
    );
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::{attributes_of, body};
    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{LiberoProvider, components::PhoneField};

    thread_local! {
        /// The country the phone field below is mounted on, and a prop that
        /// changes under it - the demo's controls in one flag.
        static PHONE_COUNTRY: std::cell::Cell<&'static str> = const { std::cell::Cell::new("DE") };
        static PHONE_TOGGLE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    fn phone_app() -> Element {
        let mut value = use_signal(String::new);
        rsx! {
            LiberoProvider {
                PhoneField {
                    label: "Mobile",
                    name: "phone",
                    country: PHONE_COUNTRY.get(),
                    description: PHONE_TOGGLE.get().then(|| "Deliveries only".to_string()),
                    value: value(),
                    oninput: move |next: String| value.set(next),
                }
            }
        }
    }

    thread_local! {
        /// Every value the recorded field below emitted.
        static EMITTED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    /// A saved US number under a field whose prop says Germany; `oninput` only records.
    fn foreign_app() -> Element {
        rsx! {
            LiberoProvider {
                PhoneField {
                    label: "Mobile",
                    name: "phone",
                    country: PHONE_COUNTRY.get(),
                    value: "+12133734253",
                    oninput: move |next: String| EMITTED.with_borrow_mut(|seen| seen.push(next)),
                }
            }
        }
    }

    /// Todo 2364: the mount leaves a value from another country alone and emits nothing;
    /// only a later change of the `country` prop moves the number, once.
    #[test]
    fn a_foreign_value_is_not_rewritten_on_mount() {
        PHONE_COUNTRY.set("DE");
        EMITTED.with_borrow_mut(Vec::clear);
        let mut page = Page::mount(foreign_app);
        page.render();
        assert_eq!(
            EMITTED.with_borrow(Vec::clone),
            Vec::<String>::new(),
            "the mount emitted"
        );
        let html = dioxus_ssr::render(&page.dom);
        assert!(
            html.contains("value=\"+12133734253\""),
            "the posted number moved: {html}"
        );
        assert_eq!(phone_text(&html), "213 373 4253");

        PHONE_COUNTRY.set("FR");
        page.dom.mark_dirty(dioxus::core::ScopeId::APP);
        page.render();
        page.render();
        assert_eq!(
            EMITTED.with_borrow(Vec::clone),
            ["+332133734253"],
            "the prop change"
        );
    }

    /// Todo 2369: a typed or pasted number with its own dial code moves the picker and keeps
    /// only the national part in the input, from either prefix and in any digits.
    #[test]
    fn a_typed_dial_code_moves_the_country_through_the_input() {
        for (typed, text, e164) in [
            ("+49 171 1234567", "1711234567", "+491711234567"),
            ("0049 30 123456", "30123456", "+4930123456"),
            ("＋４９ ３０ １２３", "30123", "+4930123"),
        ] {
            PHONE_COUNTRY.set("US");
            PHONE_TOGGLE.set(false);
            let mut page = Page::mount(phone_app);
            let input = input_listener(&page.rec);
            page.dom
                .runtime()
                .handle_event("input", Event::new(input_event(typed), true), input);
            page.render();
            let html = dioxus_ssr::render(&page.dom);
            assert_eq!(phone_text(&html), text, "{typed}: the text");
            assert!(
                html.contains(&format!("value=\"{e164}\"")),
                "{typed}: the E.164 in {html}"
            );
            assert!(
                html.contains("Country: Germany, DE +49"),
                "{typed}: the picker in {html}"
            );
        }
    }

    /// The text the `tel` input is showing, which is not what it posts.
    fn phone_text(html: &str) -> String {
        attributes_of(&body(html), "input")
            .get("value")
            .cloned()
            .unwrap_or_default()
    }

    /// Todo 87b: typing `30 123456` into a German field and then leaving it - or
    /// re-rendering it, which is what toggling a demo control does - used to bring
    /// the number back as `30123456`. Germany has no one fixed grouping, so there
    /// is nothing to regroup into and the text the user typed stays. The US, which
    /// has one, still regroups on the way out.
    #[test]
    fn a_plan_without_a_fixed_shape_keeps_the_text_the_user_typed() {
        for (country, typed, after_blur) in [
            ("DE", "30 123456", "30 123456"),
            ("US", "2133734253", "213 373 4253"),
        ] {
            PHONE_COUNTRY.set(country);
            PHONE_TOGGLE.set(false);
            let Page {
                mut dom,
                rec: mut find,
            } = Page::build(phone_app);
            // Every render is written back into `find`: the second pass, which is
            // what picks up the component CSS, replaces the nodes, and an id read
            // before it addresses an element that is gone.
            dom.render_immediate(&mut find);

            let input = input_listener(&find);
            // The `tel` input is the element that listens for both, and a blur
            // sent anywhere else would prove nothing.
            assert!(
                find.blur.contains(&input),
                "the tel input has no blur listener: {:?}",
                find.blur
            );
            dom.runtime()
                .handle_event("input", Event::new(input_event(typed), true), input);
            dom.render_immediate(&mut find);
            assert_eq!(
                phone_text(&dioxus_ssr::render(&dom)),
                typed,
                "{country}: the keystroke"
            );

            // A re-render with nothing else touched - what a demo control does.
            PHONE_TOGGLE.set(true);
            dom.mark_dirty(dioxus::core::ScopeId::APP);
            dom.render_immediate(&mut find);
            let html = dioxus_ssr::render(&dom);
            assert!(
                html.contains("Deliveries only"),
                "the re-render did nothing"
            );
            assert_eq!(phone_text(&html), typed, "{country}: the re-render");

            let input = input_listener(&find);
            dom.runtime()
                .handle_event("blur", Event::new(focus_event(), false), input);
            dom.render_immediate(&mut find);
            let html = dioxus_ssr::render(&dom);
            assert_eq!(phone_text(&html), after_blur, "{country}: the blur");
            // Whatever the text is doing, the hidden input posts E.164.
            let digits: String = typed.chars().filter(char::is_ascii_digit).collect();
            let dial = match country {
                "DE" => "49",
                _ => "1",
            };
            assert!(
                html.contains(&format!("value=\"+{dial}{digits}\"")),
                "the E.164 moved: {html}"
            );
        }
    }
}
