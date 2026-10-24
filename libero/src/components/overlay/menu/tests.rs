use super::MenuPart;
use crate::components::common::Part;

/// The slot names are public: a rename here is a breaking change.
#[test]
fn the_part_table_is_stable() {
    let table: Vec<_> = MenuPart::ALL
        .iter()
        .map(|part| (part.slot(), part.selector()))
        .collect();

    assert_eq!(
        table,
        [
            ("item", "& [data-slot='item']"),
            ("group-label", "& [data-slot='group-label']"),
            ("check", "& [data-slot='item'] > [data-slot='check']"),
            ("leading", "& [data-slot='item'] > [data-slot='leading']"),
            ("label", "& [data-slot='item'] > [data-slot='label']"),
            ("trailing", "& [data-slot='item'] > [data-slot='trailing']"),
            ("shortcut", "& [data-slot='item'] > [data-slot='shortcut']"),
            ("chevron", "& [data-slot='item'] > [data-slot='chevron']"),
        ]
    );
}

/// Todo 3840: a group with a blank label renders bare rows, not an unnamed `role=group`.
#[test]
fn a_blank_group_label_renders_bare_rows() {
    use dioxus::prelude::*;

    use super::{Menu, MenuEntry, MenuItem, use_menu};

    fn app() -> Element {
        let menu = use_menu();
        use_hook(|| menu.open());
        rsx! {
            crate::LiberoProvider {
                Menu {
                    state: menu,
                    items: vec![
                        MenuEntry::Group {
                            label: "  ".into(),
                            items: vec![MenuItem::new("Copy").onselect(|_| {}).into()],
                        },
                    ],
                    crate::components::Button { attributes: menu.a11y_attributes(), "Menu" }
                }
            }
        }
    }
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains(">Copy<"), "{html}");
    // The stylesheet names `role="group"`, so look for the label it points at.
    assert!(!html.contains("menu-group-"), "{html}");
}

/// `app`'s SSR output after its effects ran.
fn render_open(app: fn() -> dioxus::prelude::Element) -> String {
    let mut dom = dioxus::prelude::VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// Todo 4111: a trigger's own id is kept, named once, and labels the menu.
#[test]
fn a_trigger_keeps_its_own_id_inside_the_menu_and_beside_it() {
    use dioxus::prelude::*;

    use super::{Menu, MenuItem, use_menu};
    use crate::components::common::attr;

    fn inside() -> Element {
        let menu = use_menu();
        use_hook(|| menu.open());
        rsx! {
            crate::LiberoProvider {
                Menu {
                    state: menu,
                    items: vec![MenuItem::new("Upload").onselect(|_| {}).into()],
                    crate::components::Button {
                        attributes: menu.a11y_attributes_on(vec![attr("id", "avatar")]),
                        "Avatar"
                    }
                }
            }
        }
    }
    fn beside() -> Element {
        let menu = use_menu();
        use_hook(|| menu.open());
        rsx! {
            crate::LiberoProvider {
                crate::components::Button {
                    attributes: menu.a11y_attributes_on(vec![attr("id", "avatar")]),
                    onclick: move |_| menu.toggle(),
                    "Avatar"
                }
                Menu {
                    state: menu,
                    items: vec![MenuItem::new("Upload").onselect(|_| {}).into()],
                }
            }
        }
    }
    for (name, app) in [("inside", inside as fn() -> Element), ("beside", beside)] {
        let html = render_open(app);
        assert_eq!(html.matches("id=\"avatar\"").count(), 1, "{name}: {html}");
        assert!(!html.contains("-trigger"), "{name}: {html}");
        assert!(
            html.contains("aria-labelledby=\"avatar\""),
            "{name}: {html}"
        );
        assert!(html.contains("aria-haspopup=\"menu\""), "{name}: {html}");
    }
}

/// Todo 4111: a name is not followed through two `aria-labelledby` hops, so the menu
/// takes the trigger's own.
#[test]
fn a_labelled_trigger_lends_the_menu_its_labels() {
    use dioxus::prelude::*;

    use super::{Menu, MenuItem, use_menu};
    use crate::components::common::attr;

    fn app() -> Element {
        let menu = use_menu();
        use_hook(|| menu.open());
        let attributes = vec![
            attr("id", "avatar"),
            attr("aria-labelledby", "photo-label avatar"),
        ];
        rsx! {
            crate::LiberoProvider {
                Menu {
                    state: menu,
                    items: vec![MenuItem::new("Upload").onselect(|_| {}).into()],
                    crate::components::Button { attributes: menu.a11y_attributes_on(attributes), "Change" }
                }
            }
        }
    }
    let html = render_open(app);
    assert_eq!(html.matches("id=\"avatar\"").count(), 1, "{html}");
    assert_eq!(
        html.matches("aria-labelledby=\"photo-label avatar\"")
            .count(),
        2,
        "{html}"
    );
}

/// Todo 4111: without an id, `a11y_attributes_on` adds the menu's own.
#[test]
fn a_trigger_without_an_id_gets_the_menus() {
    use dioxus::prelude::*;

    use super::{Menu, MenuItem, use_menu};
    use crate::components::common::attr;

    fn app() -> Element {
        let menu = use_menu();
        use_hook(|| menu.open());
        rsx! {
            crate::LiberoProvider {
                Menu {
                    state: menu,
                    items: vec![MenuItem::new("Upload").onselect(|_| {}).into()],
                    crate::components::Button {
                        attributes: menu.a11y_attributes_on(vec![attr("title", "Avatar")]),
                        "Avatar"
                    }
                }
            }
        }
    }
    let html = render_open(app);
    let trigger = html
        .split("id=\"")
        .find_map(|rest| rest.split('"').next().filter(|id| id.ends_with("-trigger")))
        .expect(&html)
        .to_string();
    assert!(
        html.contains(&format!("aria-labelledby=\"{trigger}\"")),
        "{html}"
    );
}

/// Todo 4112: a coloured item paints its label and leading slot through the row's var.
#[test]
fn a_coloured_item_carries_its_text_colour() {
    use dioxus::prelude::*;

    use super::{Menu, MenuItem, use_menu};

    fn app() -> Element {
        let menu = use_menu();
        use_hook(|| menu.open());
        rsx! {
            crate::LiberoProvider {
                Menu {
                    state: menu,
                    items: vec![
                        MenuItem::new("Rename").onselect(|_| {}).into(),
                        MenuItem::new("Delete").color("error").onselect(|_| {}).into(),
                    ],
                    crate::components::Button { attributes: menu.a11y_attributes(), "Menu" }
                }
            }
        }
    }
    let html = render_open(app);
    assert_eq!(html.matches("data-colored=").count(), 1, "{html}");
    let style = html
        .split("--lsx-menu-item-color:")
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .expect(&html);
    // The error hue's text shade, not its plain fill shade.
    assert!(style.contains("error"), "{style}");
}
