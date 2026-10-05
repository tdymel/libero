use super::{ChronoPickerPart, ColorPickerPart};
use crate::components::common::parts_enum;

parts_enum! {
    /// The inner parts of a field's portaled dropdown, for its `dropdown_parts`
    /// prop, and of [`Combobox`](super::Combobox)'s, for its `parts`. Matched from
    /// the dropdown box at any depth: rows sit in groups and columns.
    pub enum DropdownPart {
        /// The dropdown box itself.
        Panel = "dropdown" => "&",
        /// The search box above the rows: a searchable `Select` or `MultiSelect`,
        /// a searchable `Cascader`, `PhoneField`'s country list.
        Search = "search" => "& [data-slot='search']",
        /// A scrolling `role="listbox"`; one per column on a `Cascader`.
        Listbox = "listbox" => "& [data-slot='listbox']",
        /// A `role="group"` of rows that share a label.
        Group = "group" => "& [data-slot='group']",
        /// The label atop a `Group`, which names it.
        GroupLabel = "group-label" => "& [data-slot='group-label']",
        /// A [`ComboboxOption`](super::ComboboxOption) row.
        Option = "option" => "& [data-slot='option']",
        /// A row's `span { "data-slot": "label" }`, which ellipsises.
        OptionLabel = "label" => "& [data-slot='option'] > [data-slot='label']",
        /// The text shown when a query matches nothing.
        Empty = "nothing-found" => "& [data-slot='nothing-found']",
        /// `Cascader`: one level's column.
        Column = "column" => "& [data-slot='column']",
        /// `Cascader`, narrow: the header over a child level, back to its parent.
        DrillBack = "drill-back" => "& [data-slot='drill-back']",
        /// `Cascader`, narrow with `any_level`: the row that picks the parent.
        PickParent = "pick-parent" => "& [data-slot='pick-parent']",
        /// `PhoneField`: a row's country name.
        CountryName = "name" => "& [data-slot='option'] [data-slot='name']",
        /// `PhoneField`: a row's dial code.
        CountryDial = "dial" => "& [data-slot='option'] [data-slot='dial']",
    }
}

/// A picker's selector, rebased from the picker's root onto the dropdown box that holds it.
fn under_picker(selector: &str) -> String {
    selector
        .split(", ")
        .map(|alternative| alternative.replacen('&', "& > *", 1))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Declares a picker field's dropdown enum: [`DropdownPart::Panel`], then the
/// listed parts of the picker it holds, matched through the dropdown box.
macro_rules! picker_dropdown_parts {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $picker:ident { $($variant:ident),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        $vis enum $name {
            /// The dropdown box itself.
            Panel,
            $(
                #[doc = concat!("The picker's [`", stringify!($picker), "::", stringify!($variant), "`].")]
                $variant,
            )+
        }

        impl crate::components::common::Part for $name {
            const ALL: &'static [Self] = &[Self::Panel, $(Self::$variant),+];

            fn slot(self) -> &'static str {
                match self {
                    Self::Panel => DropdownPart::Panel.slot(),
                    $(Self::$variant => $picker::$variant.slot()),+
                }
            }

            fn selector(self) -> &'static str {
                static REBASED: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
                    [$($picker::$variant),+]
                        .into_iter()
                        .map(|part| under_picker(part.selector()))
                        .collect()
                });
                match self {
                    Self::Panel => DropdownPart::Panel.selector(),
                    // `Panel` is 0, the picker's parts follow in order.
                    part => &REBASED[part as usize - 1],
                }
            }
        }
    };
}

picker_dropdown_parts! {
    /// The inner parts of a date or time field's dropdown, for its `dropdown_parts`
    /// prop: the box, then the [`ChronoPicker`](super::ChronoPicker)'s parts.
    pub enum ChronoDropdownPart: ChronoPickerPart {
        Header, Nav, Title, Months, Weekday, Day, Month, Blank, Cells, Cell, Strip,
        Columns, Spin, Value, Neighbour, Separator, Unit, Readout, Face, Mark, Ticks, Hand, Pivot,
    }
}

picker_dropdown_parts! {
    /// The inner parts of [`ColorField`](super::ColorField)'s dropdown, for its
    /// `dropdown_parts` prop: the box, then the [`ColorPicker`](super::ColorPicker)'s parts.
    pub enum ColorDropdownPart: ColorPickerPart {
        Saturation, Body, Sliders, Hue, Alpha, Track, Thumb, Preview, Swatches, Swatch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::{Part, part_table};

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<DropdownPart>(),
            [
                ("dropdown", "&"),
                ("search", "& [data-slot='search']"),
                ("listbox", "& [data-slot='listbox']"),
                ("group", "& [data-slot='group']"),
                ("group-label", "& [data-slot='group-label']"),
                ("option", "& [data-slot='option']"),
                ("label", "& [data-slot='option'] > [data-slot='label']"),
                ("nothing-found", "& [data-slot='nothing-found']"),
                ("column", "& [data-slot='column']"),
                ("drill-back", "& [data-slot='drill-back']"),
                ("pick-parent", "& [data-slot='pick-parent']"),
                ("name", "& [data-slot='option'] [data-slot='name']"),
                ("dial", "& [data-slot='option'] [data-slot='dial']"),
            ]
        );
    }

    /// Every picker part is listed, in the picker's order, after the box, and each
    /// alternative is the picker's own one step below the box.
    fn assert_mirrors<D: Part + std::fmt::Debug, P: Part>() {
        let slots: Vec<_> = D::ALL.iter().map(|part| part.slot()).collect();
        let mut expected = vec!["dropdown"];
        expected.extend(P::ALL.iter().map(|part| part.slot()));
        assert_eq!(slots, expected);
        for (dropdown, picker) in D::ALL[1..].iter().zip(P::ALL) {
            let ours: Vec<_> = dropdown.selector().split(", ").collect();
            let theirs: Vec<_> = picker.selector().split(", ").collect();
            assert_eq!(ours.len(), theirs.len(), "{dropdown:?}");
            for (ours, theirs) in ours.iter().zip(theirs) {
                let rest = theirs
                    .strip_prefix('&')
                    .expect("a picker selector starts at `&`");
                assert_eq!(ours.strip_prefix("& > *"), Some(rest), "{dropdown:?}");
            }
        }
    }

    #[test]
    fn the_picker_dropdowns_mirror_their_pickers() {
        assert_mirrors::<ChronoDropdownPart, ChronoPickerPart>();
        assert_mirrors::<ColorDropdownPart, ColorPickerPart>();
    }

    /// `Panel`'s bare `&` styles the box itself, not a descendant.
    #[test]
    fn the_panel_rule_lands_on_the_box() {
        let css = crate::css::Stylesheet::from(
            &crate::sx::sx().selector(DropdownPart::Panel.selector(), crate::sx::sx().color("red")),
        );
        let css = css.as_str();
        assert!(css.contains("{color:red;}"), "{css}");
        assert!(!css.contains(" {color"), "{css}");
    }

    #[test]
    fn a_picker_selector_steps_through_the_box_on_every_alternative() {
        assert_eq!(
            ColorDropdownPart::Thumb.selector(),
            "& > * > [data-slot='saturation'] > [data-slot='thumb'], \
             & > * > [data-slot='body'] > [data-slot='sliders'] > * > [data-slot='track'] > * > [data-slot='thumb']"
        );
        assert_eq!(
            ChronoDropdownPart::Day.selector(),
            "& > * [data-slot='day']"
        );
    }
}
