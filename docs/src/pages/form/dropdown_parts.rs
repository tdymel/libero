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
            "Analog clock: the digits over the face, buttons that pick the hand.",
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
    D::Search,
    D::Listbox,
    D::Option,
    D::CountryName,
    D::CountryDial,
];
pub(super) const CASCADER_DROPDOWN: &[DropdownPart] = &[
    D::Panel,
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

/// The date fields' dropdown: the box, then every `ChronoPicker` part.
pub(super) fn chrono_dropdown_parts() -> Vec<(ChronoDropdownPart, &'static str)> {
    let picker = chrono_picker_parts()
        .into_iter()
        .map(|(_, description)| description);
    ChronoDropdownPart::ALL
        .iter()
        .copied()
        .zip(std::iter::once(PANEL).chain(picker))
        .collect()
}

/// `ColorField`'s dropdown: the box, then every `ColorPicker` part.
pub(super) fn color_dropdown_parts() -> Vec<(ColorDropdownPart, &'static str)> {
    let picker = color_picker_parts()
        .into_iter()
        .map(|(_, description)| description);
    ColorDropdownPart::ALL
        .iter()
        .copied()
        .zip(std::iter::once(PANEL).chain(picker))
        .collect()
}
