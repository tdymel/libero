//! What a component costs to re-render, in nanoseconds per instance.
//!
//! Ignored by default - it is a measurement, not an assertion, and the numbers
//! only mean anything in a release build:
//!
//! ```text
//! cargo test --release -p libero --test render_cost -- --ignored --nocapture
//! ```
//!
//! `span` is a bare element with no scope; `Leaf` the cheapest possible
//! component. The ratios travel between machines, the absolutes do not - so
//! compare a change against a run of `main` on the same machine, not against a
//! number written down anywhere. The current baseline table lives in
//! `memory/performance.md`.
//!
//! Every shape renders its component in the plainest configuration that
//! compiles: what is measured is the framework overhead a caller pays for
//! reaching for the component at all, not any particular feature of it.
//!
//! A row that reads [`flip`] prices a state change instead: every round
//! re-renders with the other of two values, so a component that skips an
//! unchanged re-render still shows what a pick, a page or a drag costs.
//!
//! A row marked `memoized` came in below `Leaf`. That component takes no
//! `children`, so its props compare equal and dioxus skips the re-render
//! entirely - the number is the parent's diff, not the component's render.
//! Price those with a first render instead.

use std::sync::atomic::{AtomicBool, Ordering};

use dioxus::dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
use dioxus::prelude::*;
use libero::chrono::{NaiveDate, NaiveTime};
use libero::components::Title;
use libero::{LiberoProvider, components::*};

#[derive(Clone, PartialEq, Options)]
enum CostPane {
    One,
    Two,
}

/// A shape to measure: what to call it, and the app that renders
/// [`CHILDREN`] of it.
type Shape = (&'static str, fn() -> Element);

static FLIP: AtomicBool = AtomicBool::new(false);

/// Which of two values a state-change shape renders this round. False for the
/// first render, true in round 0, then alternating, so no round repeats the
/// render before it.
fn flip() -> bool {
    FLIP.load(Ordering::Relaxed)
}

/// Children per app. Big enough that per-render fixed costs disappear into the
/// per-item average.
const CHILDREN: usize = 200;
const ROUNDS: usize = 80;

/// One [`Shape`] per entry: a label, and the `rsx!` body to repeat.
macro_rules! shapes {
    ($($label:literal { $($item:tt)* })*) => {
        &[$((
            $label,
            {
                fn app() -> Element {
                    rsx! {
                        LiberoProvider {
                            div { for _ in 0..CHILDREN { $($item)* } }
                        }
                    }
                }
                app as fn() -> Element
            },
        )),*]
    };
}

/// One realistic three-level tree, built per instance the way a caller's would
/// be - the erasure is part of what a `Cascader` costs.
fn cost_tree() -> Vec<TreeNode<&'static str>> {
    vec![
        TreeNode::new("a", "A").children(vec![
            TreeNode::new("a1", "A1").children(vec![TreeNode::new("a1x", "A1x")]),
            TreeNode::new("a2", "A2"),
        ]),
        TreeNode::new("b", "B").children(vec![TreeNode::new("b1", "B1")]),
    ]
}

#[derive(Clone, PartialEq, Default, Fields)]
struct CostForm {
    text: String,
}

/// A field with rules never compares equal, so it re-renders with the form -
/// the pair below prices reading the form's value against a `value` prop.
#[component]
fn UnboundForm(children: Element) -> Element {
    rsx! {
        Form::<()> {
            TextField { name: "text", value: "", oninput: move |_| {}, validate: not_empty.error("r") }
            {children}
        }
    }
}

#[component]
fn BoundForm(children: Element) -> Element {
    let value = use_store(CostForm::default);
    rsx! {
        Form {
            value,
            TextField { name: CostForm::FIELDS.text(), validate: not_empty.error("r") }
            {children}
        }
    }
}

/// The cheapest component that can exist: one scope, one element, no styling.
#[component]
fn Leaf(children: Element) -> Element {
    rsx! { span { {children} } }
}

/// Nanoseconds to re-render one instance of each shape.
///
/// The variants are interleaved and the *minimum* per variant is kept: a
/// sequential best-of-N drifts enough between variants to invert small
/// differences.
fn measure(shapes: &[Shape]) -> Vec<(&'static str, f64)> {
    FLIP.store(false, Ordering::Relaxed);
    let mut doms: Vec<_> = shapes
        .iter()
        .map(|(name, app)| {
            let mut dom = VirtualDom::new(*app);
            dom.rebuild(&mut NoOpMutations);
            (*name, dom, u64::MAX)
        })
        .collect();

    for round in 0..ROUNDS {
        FLIP.store(round % 2 == 0, Ordering::Relaxed);
        for (_, dom, best) in doms.iter_mut() {
            dom.mark_dirty(ScopeId::APP);
            let started = std::time::Instant::now();
            dom.render_immediate(&mut NoOpMutations);
            *best = (*best).min(started.elapsed().as_nanos() as u64);
        }
    }

    doms.into_iter()
        .map(|(name, _, best)| (name, best as f64 / CHILDREN as f64))
        .collect()
}

#[test]
#[ignore = "a measurement; needs --release to mean anything"]
fn render_cost_per_component() {
    let measured = measure(shapes! {
        // Controls. Everything below is read as a multiple of `Leaf`.
        "span" { span { "x" } }
        "Leaf" { Leaf { "x" } }

        "Box" { Box { "x" } }
        "Flex" { Flex { "x" } }
        "GridZone" { GridZone { GridItem { "x" } } }
        "Center" { Center { "x" } }
        "Container" { Container { "x" } }
        "AspectRatio" { AspectRatio { "x" } }
        "Divider" { Divider {} }
        "Float" { Float { "x" } }
        "Header" { Header { "x" } }
        "ScrollArea" { ScrollArea { "x" } }
        "Virtualize" { ScrollArea { Virtualize { count: 1, item: move |_| rsx! { "x" } } } }
        "Sidebar" { Sidebar { "x" } }
        "Splitter" { Splitter { initial_size: 50.0, panel_a: rsx! { div { "l" } }, panel_b: rsx! { div { "r" } } } }

        "Text" { Text { "x" } }
        "Title" { Title { "x" } }
        "Kbd" { Kbd { "x" } }
        "Mark" { Mark { "x" } }
        "Code" { Code { source: "let x = 1;" } }
        "CodeBlock" { CodeBlock { source: "let x = 1;" } }

        "Button" { Button { "x" } }
        "ActionIcon" { ActionIcon { aria_label: "a", "x" } }
        "NativeSelect" { NativeSelect { value: CostPane::One, onchange: move |_| {} } }
        "Select" { Select { value: CostPane::One, onchange: move |_| {} } }
        "MultiSelect" { MultiSelect { value: vec![CostPane::One], onchange: move |_| {} } }
        "Autocomplete" { Autocomplete { value: "", options: vec![CostPane::One], oninput: move |_| {} } }
        "TextField" { TextField { oninput: move |_| {} } }
        "TextField+label" { TextField { oninput: move |_| {}, label: "l" } }
        // Every slot filled - what the field chrome costs over a bare control.
        "TextField+slots" { TextField { oninput: move |_| {}, label: "l", description: "d", helper: "h", status: "e", required: true } }
        "NumberField" { NumberField { value: 1i32, onchange: move |_| {} } }
        "NumberField+steppers" { NumberField { value: 1i32, onchange: move |_| {}, steppers: true } }
        "Textarea" { Textarea { oninput: move |_| {} } }
        "PasswordField" { PasswordField { oninput: move |_| {} } }
        "PasswordField-toggle" { PasswordField { oninput: move |_| {}, reveal_button: false } }
        "PhoneField" { PhoneField { oninput: move |_| {} } }
        // The picker is a `ComboboxCore` shell, a button and a portal - the
        // escape hatch `reveal_button: false` is for `PasswordField`.
        "PhoneField-picker" { PhoneField { oninput: move |_| {}, country_select: false } }
        "TextField+frame" { TextField { oninput: move |_| {}, leading: rsx! { "<" }, trailing: rsx! { ">" } } }
        // Rules never compare equal, so this one always re-renders with its parent.
        "TextField+validate" { TextField { value: "", oninput: move |_| {}, validate: [not_empty.error("r")] } }
        "Form" { Form::<()> { "x" } }
        "Fieldset" { Fieldset::<()> { "x" } }
        // Inside a form: registration and the composite lookup.
        "Form+TextField" { Form::<()> { TextField { name: "n", oninput: move |_| {} } } }
        "Form+TextField+validate" { UnboundForm { "x" } }
        // The same field bound by `name`: it reads the form's signal instead.
        "Form+bound TextField" { BoundForm { "x" } }
        // The only component whose cost scales with a prop - two elements per
        // cell, so the pair below is the per-cell price.
        "PinField" { PinField { oninput: move |_| {} } }
        // Two entries: the dropzone is the bigger surface, and both drive the
        // same hidden input.
        "FileField" { FileField { onchange: move |_| {} } }
        "FileField-dropzone" { FileField { onchange: move |_| {}, variant: "dropzone" } }
        "PinField+6" { PinField { oninput: move |_| {}, length: 6usize } }
        // Closed: the columns are not rendered until it opens, and nothing can
        // open one from a prop. The price of an *open* cascader is a browser
        // measurement, not this table's.
        "Cascader" { Cascader { data: cost_tree(), onchange: move |_: CascaderPick<&'static str>| {} } }
        "Checkbox" { Checkbox { checked: true, onchange: move |_| {} } }
        "Checkbox+label" { Checkbox { checked: true, onchange: move |_| {}, label: "l" } }
        "Radio" { Radio { checked: true, onselect: move |_| {} } }
        "RadioGroup" { RadioGroup { value: CostPane::One, onchange: move |_| {} } }
        "Chip" { Chip { checked: true, onchange: move |_| {}, "x" } }
        "Switch" { Switch { checked: true, onchange: move |_| {} } }
        "Switch+label" { Switch { checked: true, onchange: move |_| {}, label: "l" } }
        "Slider" { Slider { value: 50.0, oninput: move |_| {} } }
        "Slider+label" { Slider { value: 50.0, oninput: move |_| {}, label: "l" } }
        // The second thumb, and what a `Tooltip` costs twice.
        "RangeSlider" { RangeSlider { value: (20.0, 80.0), oninput: move |_| {} } }
        // A plain `SliderCore` over a gradient: no bar, no `Tooltip`.
        "HueSlider" { HueSlider { value: 200.0, oninput: move |_| {} } }
        "ColorSwatch" { ColorSwatch { color: ColorCode::hex(0x228be6) } }
        // The panel, a hue slider and nothing else - no alpha, no swatches.
        "ColorPicker" { ColorPicker { value: ColorCode::hex(0x228be6), oninput: move |_| {} } }
        // One drag frame.
        "ColorPicker drag" { ColorPicker { value: ColorCode::hex(if flip() { 0x228be6 } else { 0x2f8fe0 }), oninput: move |_| {} } }
        // Closed: the dropdown's picker is not rendered until it opens.
        "ColorField" { ColorField { value: ColorCode::hex(0x228be6), oninput: move |_| {} } }
        // 42 day buttons, each with its own click handler.
        "DayPicker" { DayPicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {} } }
        // Seven days in one row, each with a month label.
        "DayPicker pick" { DayPicker { value: NaiveDate::from_ymd_opt(2026, 9, if flip() { 15 } else { 14 }), onchange: move |_| {} } }
        "DayPicker page" { DayPicker { value: NaiveDate::from_ymd_opt(2026, if flip() { 10 } else { 9 }, 14), onchange: move |_| {} } }
        "DayPicker-mini" { DayPicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, calendar: "mini" } }
        // Closed: the dropdown's picker is not rendered until it opens.
        "DayField" { DayField { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {} } }
        "TimePicker" { TimePicker { value: NaiveTime::from_hms_opt(9, 30, 0), onchange: move |_| {}, variant: "digital" } }
        "TimePicker pick" { TimePicker { value: NaiveTime::from_hms_opt(9, if flip() { 35 } else { 30 }, 0), onchange: move |_| {}, variant: "digital" } }
        "TimePicker-analog" { TimePicker { value: NaiveTime::from_hms_opt(9, 30, 0), onchange: move |_| {}, variant: "analog" } }
        "TimePicker-analog pick" { TimePicker { value: NaiveTime::from_hms_opt(9, if flip() { 35 } else { 30 }, 0), onchange: move |_| {}, variant: "analog" } }
        "MonthPicker" { MonthPicker { value: NaiveDate::from_ymd_opt(2026, 9, 1), onchange: move |_| {} } }
        "DateRangePicker" { DateRangePicker { onchange: move |_| {} } }
        // A range end moving, as a hover preview does.
        "DateRangePicker end" { DateRangePicker { value: DateRange::new(NaiveDate::from_ymd_opt(2026, 9, 10).expect("a day"), NaiveDate::from_ymd_opt(2026, 9, if flip() { 15 } else { 14 })), onchange: move |_| {} } }
        "TimeField" { TimeField { value: NaiveTime::from_hms_opt(9, 30, 0), onchange: move |_| {} } }
        "SegmentedControl" { SegmentedControl { value: CostPane::One, onchange: move |_| {} } }
        "Tabs" { Tabs { value: CostPane::One, onchange: move |_| {}, panel: |_: CostPane| rsx! { "x" } } }

        "Icon" { Icon { "x" } }
        "Image" { Image { src: "/x.png" } }
        "QrCode" { QrCode { data: "x", aria_label: "a" } }
        "List" { List { ListItem { "x" } } }
        "DataList" { DataList { DataListItem { label: rsx! { "l" }, "x" } } }
        "Table" { Table { data: vec![1u32], columns: vec![column("N").value(|n: &u32| *n)] } }

        "Anchor" { Anchor { to: "https://example.com", "x" } }
        "NavLink" { NavLink { to: "https://example.com", "x" } }
        "Tree" { Tree { aria_label: "a", data: vec![TreeNode::new("a", "Alpha".to_string())] } }
        "TreeItem" { TreeItem { "x" } }

        "Overlay" { Overlay {} }
        "Paper" { Paper { "x" } }
        "Dialog" { Dialog { "x" } }
        "Tooltip" { Tooltip { label: rsx! { "t" }, "x" } }

        "FocusTrap" { FocusTrap { "x" } }
        "VisuallyHidden" { VisuallyHidden { "x" } }
    });

    let cost = |wanted: &str| {
        measured
            .iter()
            .find(|(name, _)| *name == wanted)
            .unwrap_or_else(|| panic!("{wanted} was not measured"))
            .1
    };
    let leaf = cost("Leaf");

    println!();
    for (name, ns) in &measured {
        let memoized = if *ns < leaf && *name != "span" {
            "  memoized"
        } else {
            ""
        };
        println!("{name:<22} {ns:>8.0} {:>6.2}x{memoized}", ns / leaf);
    }
    println!();

    if cfg!(debug_assertions) {
        println!("debug build - numbers are not comparable to the baseline");
        return;
    }

    // Tripwires, not targets. They catch a component growing a scope or an
    // uncached per-render build, and stay quiet for ordinary drift.
    for (name, ceiling) in [("Box", 3.0), ("Text", 5.0), ("Button", 6.0)] {
        assert!(
            cost(name) < leaf * ceiling,
            "{name} regressed: {:.0} ns, {:.1}x Leaf",
            cost(name),
            cost(name) / leaf
        );
    }
}
