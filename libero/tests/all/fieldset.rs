//! Todo 3873: a disabled `Fieldset` renders no native `disabled`, so every control
//! inside disables itself: an input natively, so a submit leaves it out, a button softly.

use crate::common::{body, render, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    chrono::NaiveDate,
    components::{
        ActionIcon, Autocomplete, Button, Cascader, CascaderOption, Checkbox, Chip, ColorCode,
        ColorField, ColorPicker, ColorSwatch, DateField, DatePicker, DateRangeField,
        DateRangePicker, DateTimeField, Fieldset, FileField, MonthPicker, MultiSelect,
        NativeSelect, NavLink, NavigationBar, NavigationBarItem, NumberField, Options, Pagination,
        PasswordField, PhoneField, PinField, Radio, RadioGroup, RangeSlider, Rating,
        SegmentedControl, Select, Slider, Switch, TagsField, TextField, Textarea, TimeField,
        TimePicker, YearPicker,
    },
};

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
}

/// Each case renders its controls inside a disabled `Fieldset`.
macro_rules! disabled_group {
    ($($name:literal => { $($body:tt)* }),* $(,)?) => {
        [$((
            $name,
            (|| rsx! {
                LiberoProvider {
                    Fieldset::<()> { label: "Group", disabled: true, $($body)* }
                }
            }) as fn() -> Element,
        )),*]
    };
}

type Case = (&'static str, fn() -> Element);

const CASES: [Case; 39] = disabled_group! {
    "TextField" => { TextField { label: "A", name: "a", value: "x" } },
    "PasswordField" => { PasswordField { label: "A", name: "a", value: "x" } },
    "Textarea" => { Textarea { label: "A", name: "a", value: "x" } },
    "NumberField" => { NumberField::<u32> { label: "A", name: "a", value: Some(3) } },
    "PinField" => { PinField { label: "A", name: "a", value: "12", length: 4usize } },
    "PhoneField" => { PhoneField { label: "A", name: "a", value: Some("+4912345".to_string()) } },
    "TagsField" => { TagsField { label: "A", name: "a", value: vec!["x".to_string()] } },
    "Autocomplete" => { Autocomplete { label: "A", name: "a", value: "x", options: vec!["x".to_string()] } },
    "Checkbox" => { Checkbox { label: "A", name: "a", checked: true } },
    "Switch" => { Switch { label: "A", name: "a", checked: true } },
    "Chip" => { Chip { name: "a", checked: true, "A" } },
    "Radio" => { Radio { label: "A", name: "a", value: "x", checked: true } },
    "RadioGroup" => { RadioGroup::<Plan> { label: "A", name: "a", value: Some(Plan::Pro) } },
    "NativeSelect" => { NativeSelect::<Plan> { label: "A", name: "a", value: Some(Plan::Pro) } },
    "Select" => { Select::<Plan> { label: "A", name: "a", value: Some(Plan::Pro) } },
    "MultiSelect" => { MultiSelect { label: "A", name: "a", options: vec!["x".to_string()], value: vec!["x".to_string()] } },
    "SegmentedControl" => { SegmentedControl::<Plan> { label: "A", name: "a", value: Some(Plan::Pro) } },
    "Cascader" => {
        Cascader::<String> {
            label: "A",
            name: "a",
            data: vec![CascaderOption::new("x", "X")],
            value: Some("x".to_string()),
        }
    },
    "Rating" => { Rating { label: "A", name: "a", value: 3.0 } },
    "Slider" => { Slider { label: "A", name: "a", value: 3.0 } },
    "RangeSlider" => { RangeSlider { label: "A", name: "a", value: (2.0, 4.0) } },
    "ColorField" => { ColorField { label: "A", name: "a", value: ColorCode::rgba(1, 2, 3, 1.0) } },
    "FileField" => { FileField { label: "A", name: "a" } },
    "DateField" => { DateField { label: "A", name: "a", value: NaiveDate::from_ymd_opt(2026, 1, 2) } },
    "DateRangeField" => { DateRangeField { label: "A", name: "a" } },
    "DateTimeField" => { DateTimeField { label: "A", name: "a" } },
    "TimeField" => { TimeField { label: "A", name: "a" } },
    "DatePicker" => { DatePicker { name: "a", value: NaiveDate::from_ymd_opt(2026, 1, 2) } },
    "DateRangePicker" => { DateRangePicker { name: "a" } },
    "MonthPicker" => { MonthPicker { name: "a" } },
    "YearPicker" => { YearPicker { name: "a" } },
    "TimePicker" => { TimePicker { name: "a" } },
    "ColorPicker" => {
        ColorPicker {
            value: ColorCode::rgba(1, 2, 3, 1.0),
            swatches: vec![ColorCode::rgba(1, 2, 3, 1.0)],
        }
    },
    "Button" => { Button { onclick: |_| {}, "A" } },
    "ActionIcon" => { ActionIcon { aria_label: "A", onclick: |_| {}, "x" } },
    "Chip button" => { Chip { onclick: |_| {}, "A" } },
    "ColorSwatch" => { ColorSwatch { color: ColorCode::rgba(1, 2, 3, 1.0), aria_label: "A", onclick: |_| {} } },
    "Pagination" => { Pagination { total: 5, page: 2, aria_label: "Pages", onchange: |_| {} } },
    "Navigation" => {
        NavLink { to: "/", nested: rsx! { NavLink { to: "/b", "B" } }, "A" }
        NavigationBar { "aria-label": "Main",
            NavigationBarItem { onclick: |_| {}, icon: rsx! { "H" }, "Home" }
        }
    },
};

/// The controls that would still act or post, by tag.
fn live_controls(html: &str) -> Vec<String> {
    let mut live = Vec::new();
    for tag in ["input", "select", "textarea"] {
        for attributes in tags_with(html, &format!("<{tag}")) {
            if !attributes.contains_key("disabled") {
                live.push(format!("<{tag}> {attributes:?}"));
            }
        }
    }
    // A field's own buttons may disable natively; the soft form is pinned by e2e `soft_disabled::`.
    for attributes in tags_with(html, "<button") {
        let soft = attributes.get("aria-disabled").is_some_and(|v| v == "true");
        if !soft && !attributes.contains_key("disabled") {
            live.push(format!("<button> {attributes:?}"));
        }
    }
    live
}

#[test]
fn a_disabled_fieldset_disables_every_control_inside_and_renders_no_native_disabled() {
    let mut failures = Vec::new();
    for (name, app) in CASES {
        let html = body(&render(app));
        let fieldset = &tags_with(&html, "<fieldset")[0];
        assert!(!fieldset.contains_key("disabled"), "{name}: {fieldset:?}");
        assert_eq!(
            fieldset.get("aria-disabled").map(String::as_str),
            Some("true")
        );
        let live = live_controls(&html);
        if !live.is_empty() {
            failures.push(format!("{name}:\n  {}", live.join("\n  ")));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
