//! `ThemeSwitcher`, the toggle alone.

use dioxus::prelude::*;
use libero::LiberoContext;
use libero::components::{Button, Flex, MenuPart, Parts, Text, ThemeSwitcher};
use libero::sx::sx;
use libero::theme::{HexColor, Theme, ThemeSet};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/theme-switcher", || rsx! { ThemeSwitcherPage {} }),
    ("/theme-switcher/themes", || rsx! { ThemesPage {} }),
    ("/theme-switcher/system", || rsx! { SystemPage {} }),
    ("/theme-switcher/named", || rsx! { NamedPage {} }),
];

/// Todo 1837: a theme beyond the pair, then a pinned scheme of the pair.
#[component]
fn NamedPage() -> Element {
    static SEPIA: Theme = Theme {
        primary: HexColor::new(0x70_4214),
        ..Theme::DEFAULT
    };
    let context = use_context::<LiberoContext>();
    let named = context.clone();
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            Button {
                id: "named",
                onclick: move |_| {
                    named.set_theme_set(ThemeSet::new().named("sepia", &SEPIA));
                    named.set_active_theme("sepia");
                },
                "Sepia"
            }
            Button { id: "dark", onclick: move |_| context.set_active_theme(ThemeSet::DARK), "Dark" }
        }
    }
}

/// The toggle with the system entry in its cycle.
#[component]
fn SystemPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            ThemeSwitcher { id: "scheme", with_system: true }
            Text { id: "page-text", "Page text" }
        }
    }
}

/// The split button: the toggle and the theme picker's chevron. `menu_parts`
/// reaches the portaled menu's labels.
#[component]
fn ThemesPage() -> Element {
    rsx! {
        ThemeSwitcher {
            id: "split",
            themes: ThemeSet::CATALOGUE,
            menu_parts: Parts::new().part(MenuPart::Label, sx().font_style("italic")),
        }
    }
}

/// Text beside the toggle, so a scheme change has something to recolour.
#[component]
fn ThemeSwitcherPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            ThemeSwitcher { id: "scheme" }
            Text { id: "page-text", "Page text" }
        }
    }
}
