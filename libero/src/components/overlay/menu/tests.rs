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
