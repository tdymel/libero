//! `Menu`, for the overlay archetype.

use dioxus::prelude::*;
use libero::components::{
    Button, FileField, FileTriggerArgs, Files, Flex, Menu, MenuEntry, MenuItem, MenuPart, Parts,
    Text, use_menu,
};
use libero::hooks::Side;
use libero::sx::sx;
use libero::theme::{MENU_MAX_HEIGHT, Size};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/menu", || rsx! { MenuPage {} }),
    ("/menu-open-on-mount", || rsx! { MenuOpenOnMountPage {} }),
    ("/menu-items-late", || rsx! { MenuItemsLatePage {} }),
    ("/menu-items-shrink", || rsx! { MenuItemsShrinkPage {} }),
    ("/menu-submenu-reopen", || rsx! { MenuSubmenuReopenPage {} }),
    ("/menu-submenu-low", || rsx! { MenuSubmenuLowPage {} }),
    ("/menu-choices", || rsx! { MenuChoicesPage {} }),
    ("/menu-row", || rsx! { MenuRowPage {} }),
    ("/menu-keep-open", || rsx! { MenuKeepOpenPage {} }),
    ("/menu-parts", || rsx! { MenuPartsPage {} }),
    ("/menu-tall/below", || rsx! { MenuTallPage { top: "100px" } }),
    (
        "/menu-tall/above",
        || rsx! { MenuTallPage { top: "calc(100vh - 100px)" } },
    ),
    ("/menu-fill", || rsx! { MenuFillPage {} }),
    ("/menu-docked", || rsx! { MenuDockedPage {} }),
    ("/menu-phone", || rsx! { MenuPhonePage {} }),
    ("/menu-phone-docked", || rsx! { MenuPhoneDockedPage {} }),
    ("/menu-file", || rsx! { MenuFilePage {} }),
];

/// A dot in `currentColor`, so it takes its slot's colour.
fn dot() -> Element {
    rsx! {
        svg { width: "16", height: "16", view_box: "0 0 16 16",
            circle { cx: "8", cy: "8", r: "6", fill: "currentColor" }
        }
    }
}

/// Todos 4110, 4112: leading icons, a red item and a submenu, a phone sheet below `sm`.
#[component]
fn MenuPhonePage() -> Element {
    let menu = use_menu();
    let mut last = use_signal(String::new);
    let pick = move |name: &'static str| move |_| last.set(name.to_string());
    let items = vec![
        MenuItem::new("Rename")
            .leading(dot())
            .onselect(pick("Rename"))
            .into(),
        MenuItem::new("Share")
            .leading(dot())
            .submenu(vec![MenuItem::new("Email").onselect(pick("Email")).into()])
            .into(),
        MenuEntry::Separator,
        MenuItem::new("Delete")
            .leading(dot())
            .color("error")
            .onselect(pick("Delete"))
            .into(),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", padding: "16px",
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
            Text { id: "last", "{last}" }
            button { id: "outside", r#type: "button", "Elsewhere" }
        }
    }
}

/// Todo 4121: the docs demo's docked menu, trigger-wide from `sm` up and a sheet below it.
#[component]
fn MenuPhoneDockedPage() -> Element {
    let menu = use_menu();

    rsx! {
        Flex { direction: "column", gap: "md", padding: "16px",
            Menu {
                state: menu,
                items: vec![MenuItem::new("Rename").onselect(|_| {}).into()],
                sx: sx()
                    .breakpoint(
                        Size::Sm,
                        sx()
                            .width("var(--lsx-popover-anchor-width)")
                            .max_width("none")
                            .margin_top("var(--lsx-popover-gap)")
                            .box_shadow("none"),
                    ),
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

/// Todo 4111: a `FileField` trigger opens a menu under the field's id; a second
/// trigger sits beside its `Menu` with its own id.
#[component]
fn MenuFilePage() -> Element {
    let mut photo = use_signal(Files::default);
    let mut opened = use_signal(|| 0);
    let menu = use_menu();
    let beside = use_menu();

    rsx! {
        Flex { direction: "column", gap: "md", padding: "16px",
            FileField {
                label: "Photo",
                value: photo(),
                onchange: move |files| photo.set(files),
                trigger: move |t: FileTriggerArgs| {
                    let open = t.open;
                    rsx! {
                        Menu {
                            state: menu,
                            items: vec![
                                MenuItem::new("Change photo")
                                    .onselect(move |_| {
                                        opened += 1;
                                        open.call(());
                                    })
                                    .into(),
                            ],
                            Button { attributes: menu.a11y_attributes_on(t.attributes), "Change" }
                        }
                    }
                },
            }
            Text { id: "opened", "{opened}" }
            Text { id: "picked", {photo().one().map(|file| file.name()).unwrap_or_default()} }
            Button {
                attributes: beside.a11y_attributes_on(vec![Attribute::new("id", "tools", None, false)]),
                onclick: move |_| beside.toggle(),
                "Tools"
            }
            Menu {
                state: beside,
                items: vec![MenuItem::new("Ruler").onselect(|_| {}).into()],
            }
            // Where Tab out of the beside menu lands (4511).
            Button { id: "after-tools", "After" }
        }
    }
}

/// minmax 3893: a round trigger at 100% of a sized box fills it, and the menu anchors on it.
#[component]
fn MenuFillPage() -> Element {
    let menu = use_menu();
    let items = vec![MenuItem::new("Copper").onselect(|_| {}).into()];

    rsx! {
        div { id: "fill-box", width: "120px", height: "120px", margin: "40px",
            Menu {
                state: menu,
                items,
                Button {
                    attributes: menu.a11y_attributes(),
                    sx: sx().width("100%").height("100%").border_radius("50%"),
                    "Ore"
                }
            }
        }
    }
}

/// minmax 3892: a row docked at the bottom opens a menu as wide as the viewport, flush on
/// the row's top edge, by `sx` alone.
#[component]
fn MenuDockedPage() -> Element {
    let menu = use_menu();
    let items = vec![
        MenuItem::new("Slot 1").onselect(|_| {}).into(),
        MenuItem::new("Slot 2").onselect(|_| {}).into(),
    ];

    rsx! {
        div { id: "dock", position: "fixed", left: "0", right: "0", bottom: "0", height: "52px",
            Menu {
                state: menu,
                items,
                side: Side::Top,
                radius: "0",
                sx: sx()
                    .left("0")
                    .right("0")
                    .width("auto")
                    .max_width("none")
                    .top("auto")
                    .bottom("52px")
                    .box_shadow("none"),
                Button {
                    attributes: menu.a11y_attributes(),
                    sx: sx().width("100%").height("100%").border_radius("0"),
                    "Queue"
                }
            }
        }
    }
}

/// Forty items and no theme cap, the trigger at `top`: only the room on the
/// side it lands on holds the box (1739).
#[component]
fn MenuTallPage(top: &'static str) -> Element {
    let menu = use_menu();
    let items = (1..=40)
        .map(|n| MenuItem::new(format!("Item {n}")).onselect(|_| {}).into())
        .collect::<Vec<_>>();

    rsx! {
        div { position: "absolute", left: "16px", top,
            Menu {
                state: menu,
                items,
                sx: sx().var(MENU_MAX_HEIGHT, "100vh"),
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

/// `parts` styles the labels on every level, `sx` every level's box.
#[component]
fn MenuPartsPage() -> Element {
    let menu = use_menu();
    let items = vec![
        MenuItem::new("Share")
            .submenu(vec![MenuItem::new("Email").onselect(|_| {}).into()])
            .into(),
    ];

    rsx! {
        Menu {
            state: menu,
            items,
            parts: Parts::new().part(MenuPart::Label, sx().font_style("italic")),
            sx: sx().letter_spacing("2px"),
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}

/// One item that keeps the menu open, one that closes it.
#[component]
fn MenuKeepOpenPage() -> Element {
    let menu = use_menu();
    let mut count = use_signal(|| 0);
    let items = vec![
        MenuItem::new("Zoom in")
            .keep_open()
            .onselect(move |_| count += 1)
            .into(),
        MenuItem::new("Save").onselect(|_| {}).into(),
    ];

    rsx! {
        p { id: "count", "{count}" }
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "View" }
        }
    }
}

/// Radio items with one checked, a toggle with a shortcut, a label longer than
/// a phone is wide, and a disabled item with a shortcut.
#[component]
fn MenuChoicesPage() -> Element {
    let menu = use_menu();
    let mut sort = use_signal(|| "Name");
    let mut grid = use_signal(|| false);
    let choice = move |label: &'static str| {
        MenuItem::new(label)
            .radio(sort() == label)
            .onselect(move |_| sort.set(label))
            .into()
    };
    let items = vec![
        MenuEntry::Group {
            label: "Sort by".into(),
            items: vec![choice("Name"), choice("Date"), choice("Size")],
        },
        MenuEntry::Separator,
        MenuItem::new("Show grid")
            .checkbox(grid())
            .shortcut("Control+G")
            .onselect(move |_| grid.toggle())
            .into(),
        MenuItem::new("Export every selected row as a comma separated values file")
            .onselect(|_| {})
            .into(),
        MenuItem::new("Print")
            .shortcut("Control+P")
            .disabled(true)
            .onselect(|_| {})
            .into(),
    ];

    rsx! {
        p { id: "sort", "{sort}" }
        p { id: "grid", "{grid}" }
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "View" }
        }
    }
}

/// A group, a separator, a disabled item and a submenu - the shapes a menu's
/// accessibility tree can take.
#[component]
fn MenuPage() -> Element {
    let menu = use_menu();
    let items = vec![
        MenuEntry::Group {
            label: "Edit".into(),
            items: vec![
                MenuItem::new("Cut").onselect(|_| {}).into(),
                MenuItem::new("Paste")
                    .disabled(true)
                    .onselect(|_| {})
                    .into(),
            ],
        },
        MenuEntry::Separator,
        MenuItem::new("Save").onselect(|_| {}).into(),
        MenuItem::new("Share")
            .submenu(vec![MenuItem::new("Email").onselect(|_| {}).into()])
            .into(),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

/// The docs demo's row: the trigger and the text beside it, centred.
#[component]
fn MenuRowPage() -> Element {
    let menu = use_menu();
    let items = vec![MenuItem::new("Save").onselect(|_| {}).into()];

    rsx! {
        Flex { direction: "row", align: "center", gap: "md",
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
            Text { id: "menu-row-text", size: "sm", "Last chosen: nothing yet" }
        }
    }
}

/// Todo 408: a menu opened before it mounts still has to move focus in.
#[component]
fn MenuOpenOnMountPage() -> Element {
    let menu = use_menu();
    use_hook(|| menu.open());
    let items = vec![
        MenuItem::new("Save").onselect(|_| {}).into(),
        MenuItem::new("Share").onselect(|_| {}).into(),
    ];

    rsx! {
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}

/// Todo 1789: opened empty, its items land a render later, as a load would.
#[component]
fn MenuItemsLatePage() -> Element {
    let menu = use_menu();
    let mut loaded = use_signal(|| false);
    use_effect(move || {
        if menu.is_open() && !*loaded.peek() {
            loaded.set(true);
        }
    });
    let items: Vec<MenuEntry> = match loaded() {
        true => vec![
            MenuItem::new("Save").onselect(|_| {}).into(),
            MenuItem::new("Share").onselect(|_| {}).into(),
        ],
        false => Vec::new(),
    };

    rsx! {
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}

/// Todo 3554: choosing the last item, which stays open, leaves a single item.
#[component]
fn MenuItemsShrinkPage() -> Element {
    let menu = use_menu();
    let mut shrunk = use_signal(|| false);
    let mut items: Vec<MenuEntry> = vec![MenuItem::new("Save").onselect(|_| {}).into()];
    if !shrunk() {
        items.push(MenuItem::new("Share").onselect(|_| {}).into());
        items.push(
            MenuItem::new("Shrink")
                .keep_open()
                .onselect(move |_| shrunk.set(true))
                .into(),
        );
    }

    rsx! {
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}

/// Todo 3641: a phone-wide menu near the bottom leaves its submenu no room on either side.
#[component]
fn MenuSubmenuLowPage() -> Element {
    let menu = use_menu();
    let items = vec![
        MenuItem::new("Export every selected row as a comma separated values file")
            .onselect(|_| {})
            .into(),
        MenuItem::new("Share")
            .submenu(vec![
                MenuItem::new("Copy link").onselect(|_| {}).into(),
                MenuItem::new("Email").onselect(|_| {}).into(),
                MenuItem::new("Messages").onselect(|_| {}).into(),
            ])
            .into(),
    ];

    rsx! {
        div { position: "absolute", left: "16px", top: "calc(100vh - 180px)",
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

/// A closed menu drops its submenu levels: a reopen must start them closed and
/// show the items changed meanwhile.
#[component]
fn MenuSubmenuReopenPage() -> Element {
    let menu = use_menu();
    let mut renamed = use_signal(|| false);
    let mut picked = use_signal(String::new);
    let label = if renamed() { "Post" } else { "Email" };
    let items = vec![
        MenuItem::new("Share")
            .submenu(vec![
                MenuItem::new(label)
                    .onselect(move |_| picked.set(label.to_string()))
                    .into(),
            ])
            .into(),
    ];

    rsx! {
        // Above the trigger, so the open levels do not cover it.
        Flex { direction: "column", gap: "md", max_width: "320px",
            button { id: "rename", onclick: move |_| renamed.set(true), "Rename" }
            p { id: "picked", "{picked}" }
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}
