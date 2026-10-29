//! Style API rows shared by the pickers and the fields whose dropdown holds one.

use libero::components::{
    ChronoDropdownPart, ChronoPickerPart, ColorDropdownPart, ColorPickerPart, DropdownPart, Part,
};

pub(super) fn chrono_picker_parts() -> Vec<(ChronoPickerPart, &'static str)> {
    vec![
        (
            ChronoPickerPart::Header,
            "Calendar: the row over a month, a year or a decade, with paging buttons and title.",
        ),
        (
            ChronoPickerPart::Nav,
            "Calendar: a paging button, in the header or beside the mini calendar's row.",
        ),
        (
            ChronoPickerPart::Title,
            "Calendar: the month, year or decade heading, a button that climbs a level.",
        ),
        (
            ChronoPickerPart::Months,
            "Days: the months side by side, or the mini calendar's row.",
        ),
        (
            ChronoPickerPart::Weekday,
            "Days: a weekday name over a month's columns.",
        ),
        (ChronoPickerPart::Day, "Days: a day button."),
        (
            ChronoPickerPart::Month,
            "Mini calendar: the month over a day's number.",
        ),
        (
            ChronoPickerPart::Blank,
            "Days, `columns` over 1: an empty cell instead of a neighbour's day.",
        ),
        (ChronoPickerPart::Cells, "Months and years: their grid."),
        (
            ChronoPickerPart::Cell,
            "Months and years: a month or a year button.",
        ),
        (
            ChronoPickerPart::Strip,
            "Mini calendar: the row of days between its paging buttons.",
        ),
        (
            ChronoPickerPart::Columns,
            "Digital clock, duration: the columns.",
        ),
        (
            ChronoPickerPart::Spin,
            "Digital clock, duration: one column, a spinbutton.",
        ),
        (
            ChronoPickerPart::Value,
            "Digital clock, duration: a column's value.",
        ),
        (
            ChronoPickerPart::Neighbour,
            "Digital clock, duration: the faded values above and below a column's value.",
        ),
        (
            ChronoPickerPart::Separator,
            "Digital clock: the `:` between columns.",
        ),
        (
            ChronoPickerPart::Unit,
            "Duration: the unit after each column.",
        ),
        (
            ChronoPickerPart::Readout,
            "Analog clock: the digits over the face, buttons that pick the hand. Scrolling range calendar: the picked range's two ends above the months.",
        ),
        (ChronoPickerPart::Face, "Analog clock: the face, a slider."),
        (
            ChronoPickerPart::Mark,
            "Analog clock: a number on the face.",
        ),
        (
            ChronoPickerPart::Ticks,
            "Analog clock: the ring of ticks for steps finer than the marks.",
        ),
        (ChronoPickerPart::Hand, "Analog clock: the hand."),
        (
            ChronoPickerPart::Pivot,
            "Analog clock: the dot the hand turns on.",
        ),
        (
            ChronoPickerPart::Presets,
            "Calendar with `presets`: the column of preset buttons, a row on a phone.",
        ),
        (
            ChronoPickerPart::Preset,
            "Calendar with `presets`: a preset button.",
        ),
        (
            ChronoPickerPart::WeekNumber,
            "Days with `week_numbers`: a week's number, and the empty cell over them.",
        ),
    ]
}

pub(super) fn color_picker_parts() -> Vec<(ColorPickerPart, &'static str)> {
    vec![
        (
            ColorPickerPart::Saturation,
            "The saturation and brightness panel.",
        ),
        (
            ColorPickerPart::Body,
            "The row under the panel: the sliders and the preview.",
        ),
        (
            ColorPickerPart::Sliders,
            "The column of the hue and alpha sliders.",
        ),
        (ColorPickerPart::Hue, "The hue slider."),
        (
            ColorPickerPart::Alpha,
            "The alpha slider, with `with_alpha`.",
        ),
        (ColorPickerPart::Track, "Both sliders' gradient tracks."),
        (
            ColorPickerPart::Thumb,
            "Every handle: the panel's and the sliders'.",
        ),
        (
            ColorPickerPart::Preview,
            "The current color beside the sliders, with `with_alpha`.",
        ),
        (ColorPickerPart::Swatches, "The row of preset swatches."),
        (ColorPickerPart::Swatch, "One preset swatch."),
    ]
}

const PANEL: &str = "The dropdown box itself.";
const SHEET_HANDLE: &str = "Below `sm`: the swipe strip atop the sheet.";

use DropdownPart as D;

/// The `DropdownPart`s each list field draws.
pub(super) const COMBOBOX_DROPDOWN: &[DropdownPart] = &[
    D::Panel,
    D::Listbox,
    D::Group,
    D::GroupLabel,
    D::Option,
    D::OptionLabel,
    D::Empty,
];
pub(super) const SELECT_DROPDOWN: &[DropdownPart] = &[
    D::Panel,
    D::SheetHandle,
    D::Search,
    D::Listbox,
    D::Group,
    D::GroupLabel,
    D::Option,
    D::OptionLabel,
    D::Empty,
];
pub(super) const SUGGESTION_DROPDOWN: &[DropdownPart] =
    &[D::Panel, D::Listbox, D::Option, D::OptionLabel, D::Empty];
pub(super) const PHONE_DROPDOWN: &[DropdownPart] = &[
    D::Panel,
    D::SheetHandle,
    D::Search,
    D::Listbox,
    D::Option,
    D::CountryName,
    D::CountryDial,
    D::Empty,
];
pub(super) const CASCADER_DROPDOWN: &[DropdownPart] = &[
    D::Panel,
    D::SheetHandle,
    D::Search,
    D::Listbox,
    D::Column,
    D::Option,
    D::OptionLabel,
    D::DrillBack,
    D::PickParent,
    D::Empty,
];

/// The rows of a list dropdown, in the enum's order, for the parts a field draws.
pub(super) fn list_dropdown_parts(drawn: &[DropdownPart]) -> Vec<(DropdownPart, &'static str)> {
    DropdownPart::ALL
        .iter()
        .copied()
        .filter(|part| drawn.contains(part))
        .map(|part| {
            let description = match part {
                DropdownPart::Panel => PANEL,
                DropdownPart::SheetHandle => SHEET_HANDLE,
                DropdownPart::Search => "The search box above the rows.",
                DropdownPart::Listbox => "The scrolling list of rows.",
                DropdownPart::Group => "A group of rows that share a label, from an `OptionList`.",
                DropdownPart::GroupLabel => "A group's heading.",
                DropdownPart::Option => "A row.",
                DropdownPart::OptionLabel => "A row's `span { \"data-slot\": \"label\" }`, which ends in an ellipsis.",
                DropdownPart::Empty => "The text shown when the query matches nothing.",
                DropdownPart::Column => "One level's column.",
                DropdownPart::DrillBack => "On a narrow screen, the header over a child level that goes back to its parent.",
                DropdownPart::PickParent => "On a narrow screen with `any_level`, the row that picks the parent.",
                DropdownPart::CountryName => "A row's country name.",
                DropdownPart::CountryDial => "A row's dial code.",
            };
            (part, description)
        })
        .collect()
}

/// A picker field's dropdown rows in its enum's order, each matched to its description
/// by slot; a part without one is left out, which the unit test catches.
fn picker_dropdown_rows<D: Part, P: Part>(
    picker: Vec<(P, &'static str)>,
) -> Vec<(D, &'static str)> {
    D::ALL
        .iter()
        .filter_map(|&part| {
            let description = match part.slot() {
                slot if slot == DropdownPart::Panel.slot() => PANEL,
                slot if slot == DropdownPart::SheetHandle.slot() => SHEET_HANDLE,
                slot => picker.iter().find(|(row, _)| row.slot() == slot)?.1,
            };
            Some((part, description))
        })
        .collect()
}

/// The date fields' dropdown: the box and its sheet strip, then every `ChronoPicker` part.
pub(super) fn chrono_dropdown_parts() -> Vec<(ChronoDropdownPart, &'static str)> {
    picker_dropdown_rows(chrono_picker_parts())
}

/// `ColorField`'s dropdown: the box and its sheet strip, then every `ColorPicker` part.
pub(super) fn color_dropdown_parts() -> Vec<(ColorDropdownPart, &'static str)> {
    picker_dropdown_rows(color_picker_parts())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_one_row_each<P: Part + PartialEq + std::fmt::Debug>(rows: &[(P, &str)]) {
        assert_eq!(rows.len(), P::ALL.len());
        for part in P::ALL {
            let count = rows.iter().filter(|(row, _)| row == part).count();
            assert_eq!(count, 1, "{part:?}");
        }
    }

    /// Todo 4883: every part of a dropdown or picker enum has exactly one docs row.
    #[test]
    fn every_part_has_exactly_one_row() {
        assert_one_row_each(&chrono_picker_parts());
        assert_one_row_each(&color_picker_parts());
        assert_one_row_each(&chrono_dropdown_parts());
        assert_one_row_each(&color_dropdown_parts());
        assert_one_row_each(&list_dropdown_parts(DropdownPart::ALL));
    }
}
