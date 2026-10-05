//! `ButtonGroup`: bordered and borderless rows, a column, wrapped items, RTL.

use dioxus::prelude::*;
use libero::components::{ActionIcon, Button, ButtonGroup, DirectionToggle, Flex, ThemeSwitcher};
use libero::theme::ThemeSet;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/button-group", || rsx! { ButtonGroupPage {} }),
    ("/button-group/rtl", || {
        rsx! {
            div { dir: "rtl", ButtonGroupPage {} }
        }
    }),
];

#[component]
fn ButtonGroupPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", align: "start",
            ButtonGroup { id: "outlined", "aria-label": "Outlined", variant: "outlined", radius: "lg",
                Button { id: "o1", "One" }
                Button { id: "o2", "Two" }
                ActionIcon { id: "o3", aria_label: "Three", "3" }
            }
            ButtonGroup { id: "filled", "aria-label": "Filled", variant: "filled",
                Button { id: "f1", "One" }
                Button { id: "f2", "Two" }
            }
            // A `display: contents` wrapper, as `Repository`'s, and `ThemeSwitcher`'s own pair.
            ButtonGroup { id: "mixed", "aria-label": "Mixed", size: "sm",
                span { display: "contents", Button { id: "m1", "Wrapped" } }
                DirectionToggle { id: "m2" }
                ThemeSwitcher { id: "m3", themes: ThemeSet::CATALOGUE }
            }
            ButtonGroup { id: "disabled", "aria-label": "Disabled", variant: "outlined", disabled: true,
                Button { id: "d1", "Off" }
                Button { id: "d2", disabled: false, "On" }
            }
            ButtonGroup { id: "vertical", "aria-label": "Vertical", variant: "outlined", orientation: "vertical",
                Button { id: "v1", "Top" }
                Button { id: "v2", "Middle" }
                Button { id: "v3", "Bottom" }
            }
            // After the rest: the baseline's tab walk reaches `#v2` first.
            // The pair leads: its chevron seam needs a divider of its own.
            ButtonGroup { id: "pair", "aria-label": "Pair", size: "sm", variant: "standard",
                ThemeSwitcher { id: "p1", themes: ThemeSet::CATALOGUE }
                Button { id: "p2", "After" }
            }
            // A hidden first child still squares its neighbour; a lone child is round.
            ButtonGroup { id: "hidden-in", "aria-label": "Hidden in", variant: "outlined",
                span { display: "none", Button { id: "h1", "Hidden" } }
                Button { id: "h2", "Shown" }
            }
            ButtonGroup { id: "hidden-out", "aria-label": "Hidden out", variant: "outlined",
                Button { id: "h3", "Shown" }
            }
            // Set-width items in a column: an icon, a wrapped icon, the pair.
            ButtonGroup { id: "vertical-mixed", "aria-label": "Vertical mixed", variant: "outlined", orientation: "vertical",
                Button { id: "vm1", "A wider label" }
                ActionIcon { id: "vm2", aria_label: "Two", "2" }
                span { display: "contents", ActionIcon { id: "vm3", aria_label: "Three", "3" } }
                // In the Tooltip's `max-content` wrapper (todo 2321).
                ActionIcon { id: "vm5", aria_label: "Five", tooltip: true, "5" }
                ThemeSwitcher { id: "vm4", themes: ThemeSet::CATALOGUE }
            }
            ButtonGroup { id: "tipped", "aria-label": "Tipped", variant: "outlined",
                Button { id: "t1", "Label" }
                ActionIcon { id: "t2", aria_label: "Tip", tooltip: true, "T" }
            }
        }
    }
}
