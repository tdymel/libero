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
