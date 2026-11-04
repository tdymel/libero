//! Todo 5638: a light and a dark `ThemeScope` side by side, a `Select` inside the dark one
//! whose portalled list must carry the scope.

use dioxus::prelude::*;
use libero::{
    ThemeScope,
    components::{Flex, Paper, Select},
    sx::sx,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[("/theme-scope", || rsx! { ThemeScopePage {} })];

#[component]
fn ThemeScopePage() -> Element {
    let mut fruit = use_signal(|| Some(Fruit::Banana));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { id: "light",
                ThemeScope { theme: "light",
                    Paper { sx: sx().padding("md"), "Light" }
                }
            }
            div { id: "dark",
                ThemeScope { theme: "dark",
                    Select {
                        label: "Fruit",
                        value: fruit(),
                        onchange: move |next| fruit.set(next),
                    }
                }
            }
        }
    }
}
