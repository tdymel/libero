//! `Chip`, in every kind its docs page shows: tag, filter, action, link.

use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Button, Chip, ColorCode, ColorSwatch, Fieldset, Flex, Icon, Image, PhoneField,
    Tree, TreeItem, TreeNode, TreeNodeRenderArgs,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/chip", || rsx! { ChipPage {} }),
    ("/chip/fieldset", || rsx! { ChipFieldsetPage {} }),
    ("/chip/readonly", || rsx! { ChipReadonlyPage {} }),
    ("/chip/removable", || rsx! { ChipRemovablePage {} }),
    ("/chip/icons", || rsx! { ChipIconsPage {} }),
    ("/chip/new-tab", || rsx! { ChipNewTabPage {} }),
    (
        "/chip/trailing-badge",
        || rsx! { ChipTrailingPage { control: false } },
    ),
    (
        "/chip/trailing-control",
        || rsx! { ChipTrailingPage { control: true } },
    ),
];

/// Todo 629: a readonly filter chip and a readonly form chip in a `<form>`;
/// neither toggles, and the form one still posts.
#[component]
fn ChipReadonlyPage() -> Element {
    let mut emitted = use_signal(String::new);
    rsx! {
        form { id: "chip-form", "data-emitted": emitted(),
            Flex { direction: "row", gap: "md",
                Chip {
                    id: "locked",
                    readonly: true,
                    checked: true,
                    onchange: move |next| emitted.set(format!("locked:{next}")),
                    "Locked"
                }
                Chip {
                    id: "locked-form",
                    name: "tags",
                    value: "rust",
                    readonly: true,
                    checked: true,
                    onchange: move |next| emitted.set(format!("locked-form:{next}")),
                    "rust"
                }
            }
        }
    }
}

/// Todo 661: an `onclick` chip and a `to` chip with `trailing` - a count
/// (fine) or a button (nested interactive, warned in debug).
#[component]
fn ChipTrailingPage(control: bool) -> Element {
    let trailing = move || match control {
        true => rsx! { button { r#type: "button", "x" } },
        false => rsx! { span { "3" } },
    };
    rsx! {
        Flex { direction: "row", gap: "md",
            Chip { id: "action-trailing", onclick: move |_| {}, trailing: trailing(), "Action" }
            Chip { id: "link-trailing", to: "/chip", trailing: trailing(), "Link" }
        }
    }
}

/// A selected chip with a remove x, which has to follow its label in forced
/// colours (todo 646), beside an unselected one.
#[component]
fn ChipRemovablePage() -> Element {
    let mut emitted = use_signal(String::new);
    rsx! {
        Flex { direction: "row", gap: "md", "data-emitted": emitted(),
            Chip {
                id: "removable",
                checked: true,
                onchange: move |next| emitted.set(format!("removable:{next}")),
                trailing: rsx! {
                    ActionIcon { id: "remove", aria_label: "Remove wasm", size: "xs", onclick: move |_| emitted.set("remove".into()),
                        svg { view_box: "0 0 24 24", path { d: "M6 6l12 12M18 6L6 18", stroke: "currentColor" } }
                    }
                },
                "wasm"
            }
            // Todo 686: an x with its own fill.
            Chip {
                id: "removable-filled",
                checked: true,
                onchange: move |_| {},
                trailing: rsx! {
                    ActionIcon { aria_label: "Remove rust", size: "xs", variant: "filled", color: "error", onclick: move |_| {},
                        svg { view_box: "0 0 24 24", path { d: "M6 6l12 12M18 6L6 18", stroke: "currentColor" } }
                    }
                },
                "rust"
            }
            Chip { id: "plain", checked: false, onchange: move |_| {}, "js" }
        }
    }
}

/// Todo 674: an icon among the children and one in `icon` both keep the gap
/// and sit on the text's centre line, as the docs header chips need.
#[component]
fn ChipIconsPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md",
            Chip { id: "in-children", size: "sm", variant: "outlined",
                Icon { variant: "standard", size: "sm", color: "inherit", Glyph {} }
                "Source"
            }
            Chip { id: "in-prop", size: "sm", variant: "outlined",
                icon: rsx! { Icon { variant: "standard", size: "sm", color: "inherit", Glyph {} } },
                "Source"
            }
        }
    }
}

#[component]
fn Glyph() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "8" } }
    }
}

/// Todo 672: the new-tab icon stays whole when the label is cut.
#[component]
fn ChipNewTabPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md",
            Chip { id: "new-tab", to: "https://example.com", target: "_blank", "External" }
            Chip {
                id: "new-tab-long",
                to: "https://example.com",
                target: "_blank",
                style: "max-width: 120px",
                span { style: "min-width: 0; overflow: hidden; text-overflow: ellipsis",
                    "Versandkostenberechnungsgrundlage"
                }
            }
        }
    }
}

/// Button-rooted controls in a disabled `Fieldset`: the browser disables the
/// `<button>`, so each has to look disabled too (todo 499).
#[component]
fn ChipFieldsetPage() -> Element {
    rsx! {
        Fieldset::<()> { label: "Actions", disabled: true,
            Flex { direction: "row", gap: "md",
                Chip { id: "fs-chip", onclick: move |_| {}, "Action" }
                Button { id: "fs-button", onclick: move |_| {}, "Save" }
                ActionIcon { id: "fs-icon", aria_label: "Delete", onclick: move |_| {},
                    svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "8" } }
                }
                // Todo 514: the other button-rooted components.
                ColorSwatch { id: "fs-swatch", color: ColorCode::hex(0x40c057), onclick: move |_| {}, aria_label: "Green" }
                TreeItem { id: "fs-tree-item", "Item" }
                div { id: "fs-phone", PhoneField { label: "Phone", value: "", oninput: move |_| {} } }
                div { id: "fs-image", width: "32px", height: "32px",
                    Image { src: DOT, alt: "Dot", zoomable: true }
                }
            }
        }
        // A row the tree disables dims itself; its `TreeItem` must not dim twice.
        Tree {
            aria_label: "Rows",
            data: vec![TreeNode::new("on", "On"), TreeNode::new("off", "Off").disabled(true)],
            render_node: move |args: TreeNodeRenderArgs<&'static str>| rsx! {
                TreeItem { id: "tree-{args.id}", "{args.data}" }
            },
        }
    }
}

const DOT: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 2 2'%3E%3Ccircle cx='1' cy='1' r='1'/%3E%3C/svg%3E";

/// Every chip reports what it last emitted into `data-emitted`.
#[component]
fn ChipPage() -> Element {
    let mut rust = use_signal(|| true);
    let mut css = use_signal(|| false);
    let mut small = use_signal(|| false);
    let mut emitted = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px", "data-emitted": emitted(),
            Flex { direction: "row", gap: "md", wrap: "wrap",
                Chip { id: "tag", "Tag" }
                Chip {
                    id: "rust",
                    checked: rust(),
                    onchange: move |next| {
                        rust.set(next);
                        emitted.set(format!("rust:{next}"));
                    },
                    "rust"
                }
                Chip {
                    id: "css",
                    variant: "outlined",
                    checked: css(),
                    onchange: move |next| {
                        css.set(next);
                        emitted.set(format!("css:{next}"));
                    },
                    "css"
                }
                Chip { id: "named", name: "tags", value: "html", "html" }
                Chip {
                    id: "off",
                    disabled: true,
                    checked: false,
                    onchange: move |next| emitted.set(format!("off:{next}")),
                    "Unavailable"
                }
            }
            Flex { direction: "row", gap: "md", wrap: "wrap",
                Chip {
                    id: "action",
                    onclick: move |_| emitted.set("action".to_string()),
                    "Action"
                }
                Chip { id: "link", to: "/chip", "Link" }
                Chip { id: "dead-link", to: "/chip", disabled: true, "Dead link" }
                // The other variants, so axe sees every look's text.
                Chip { id: "tonal", variant: "tonal", name: "look", value: "tonal", "tonal" }
                Chip { id: "elevated", variant: "elevated", name: "look", value: "elevated", "elevated" }
                Chip { id: "text", variant: "text", name: "look", value: "text", "text" }
                Chip {
                    id: "small",
                    size: "xs",
                    checked: small(),
                    onchange: move |next| {
                        small.set(next);
                        emitted.set(format!("small:{next}"));
                    },
                    "small"
                }
            }
        }
    }
}
