//! Fixtures for the two keyboard-group archetypes: `RadioSet` (`RadioGroup`,
//! `SegmentedControl`) and `RovingTabindex` (`Menubar`).

use dioxus::prelude::*;
use libero::components::{
    Button, Flex, MenuEntry, MenuItem, Menubar, MenubarMenu, Options, RadioGroup, SegmentedControl,
};

/// A control on each side of a radio group, so "Tab leaves the group" and
/// "Shift+Tab leaves the group" land somewhere real. At the document's edge
/// Chromium parks Shift+Tab on a stop of its own for one press, and the
/// assertion would be reading that instead of the component.
#[component]
fn Between(children: Element) -> Element {
    rsx! {
        Button { id: "before", variant: "outlined", "Before" }
        {children}
        Button { id: "after", variant: "outlined", "After" }
    }
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
    Team,
}

/// Starts on the **second** option, so "Tab enters at the checked radio" can
/// fail: with the first one checked it is indistinguishable from "Tab enters at
/// the first radio".
#[component]
pub fn RadioGroupPage() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                RadioGroup {
                    label: "Plan",
                    value: plan(),
                    onchange: move |next| plan.set(Some(next)),
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

/// Starts on the middle segment, for the same reason as `RadioGroupPage`.
///
/// `readonly` exists for the planted defect only: a read-only strip refuses the
/// arrows, which is exactly "the arrows do nothing" as a keyboard user meets
/// it.
#[component]
pub fn SegmentedControlPage(#[props(default)] readonly: bool) -> Element {
    let mut alignment = use_signal(|| Alignment::Center);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Between {
                SegmentedControl {
                    label: "Alignment",
                    readonly,
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                }
            }
        }
    }
}

/// Three menus, the middle one with a submenu row and a separator, so the
/// open-menu state has the shapes a real bar has.
#[component]
pub fn MenubarPage() -> Element {
    let item = |name: &str| -> MenuEntry { MenuItem::new(name).onselect(|_| {}).into() };
    let menus = vec![
        MenubarMenu::new(
            "File",
            vec![
                item("New"),
                item("Open"),
                MenuEntry::Separator,
                item("Save"),
            ],
        ),
        MenubarMenu::new(
            "Edit",
            vec![
                item("Undo"),
                item("Redo"),
                MenuItem::new("Find")
                    .submenu(vec![item("Find next")])
                    .into(),
            ],
        ),
        MenubarMenu::new("View", vec![item("Zoom in"), item("Zoom out")]),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Menubar { aria_label: "Editor", menus }
        }
    }
}
