use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Kbd,
    theme::{KbdDefaults, Size, Theme},
};

#[test]
fn kbd_renders_its_key() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Kbd { "Ctrl" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<kbd"));
    assert!(body(&html).contains("Ctrl"));
}

static LARGE_KEYS: Theme = Theme {
    kbd: KbdDefaults {
        size: Size::Lg,
        ..Theme::DEFAULT.kbd
    },
    ..Theme::DEFAULT
};

#[test]
fn an_unsized_kbd_takes_the_themes_default_size() {
    fn app() -> Element {
        rsx! { LiberoProvider { theme: &LARGE_KEYS, Kbd { "Ctrl" } } }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "kbd")["data-state"], "size-lg");
}
