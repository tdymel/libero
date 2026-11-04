//! Todo 5638: a light and a dark `ThemeScope` side by side, a `Select` inside the dark one
//! whose portalled list must carry the scope. Todo 5681: `Sx::dark` inside a scope follows the
//! scope, whichever scheme the page pins. Todo 5710: a drawer opened inside reads the scope's
//! theme through `use_theme`.

use dioxus::prelude::*;
use libero::{
    LiberoContext, ThemeScope,
    components::{Box, Flex, Paper, Select},
    hooks::{DrawerOptions, ModalScope, use_drawer, use_theme},
    sx::sx,
    theme::{Theme, ThemeSet},
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/theme-scope", || rsx! { ThemeScopePage {} }),
    ("/theme-scope/dark-page", || rsx! { DarkPage {} }),
];

/// The same page under a pinned dark theme of its own name, which stores no scheme.
#[component]
fn DarkPage() -> Element {
    static OCEAN: Theme = Theme::DARK;
    let context = use_context::<LiberoContext>();
    use_effect(move || {
        context.set_theme_set(ThemeSet::new().named("ocean", &OCEAN));
        context.set_active_theme("ocean");
    });
    rsx! { ThemeScopePage {} }
}

#[component]
fn ThemeScopePage() -> Element {
    let mut fruit = use_signal(|| Some(Fruit::Banana));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Box { id: "probe-page", sx: probe(), "Page" }
            div { id: "light",
                ThemeScope { theme: "light",
                    Paper { sx: sx().padding("md"), "Light" }
                    Box { id: "probe-light", sx: probe(), "Light probe" }
                }
            }
            div { id: "dark",
                ThemeScope { theme: "dark",
                    Box { id: "probe-dark", sx: probe(), "Dark probe" }
                    Select {
                        label: "Fruit",
                        value: fruit(),
                        onchange: move |next| fruit.set(next),
                    }
                    DrawerOpener {}
                }
            }
        }
    }
}

#[component]
fn DrawerOpener() -> Element {
    let options = DrawerOptions {
        aria_label: Some("Theme".into()),
        ..Default::default()
    };
    let drawer = use_drawer(options, |_: ModalScope<()>| rsx! { DrawerTheme {} });
    rsx! {
        button { id: "open-drawer", onclick: move |_| {
                drawer.open();
            }, "Open drawer" }
    }
}

/// The surface `use_theme` gives inside the portalled drawer.
#[component]
fn DrawerTheme() -> Element {
    let surface = use_theme().surface.to_string();
    rsx! { span { id: "drawer-theme", "{surface}" } }
}

/// Red, green in a dark scheme.
fn probe() -> libero::sx::Sx {
    sx().background_color("#ff0000").dark(sx().background_color("#008000"))
}
