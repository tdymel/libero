//! What a component costs to re-render, in nanoseconds per instance.
//!
//! Ignored by default - it is a measurement, not an assertion, and the numbers
//! only mean anything in a release build:
//!
//! ```text
//! cargo test --release -p libero --test render_cost -- --ignored --nocapture
//! ```
//!
//! One component on its own - every row whose label is that name, or starts
//! with it and then ` `, `+` or `-`, plus the two controls. Commas for more:
//!
//! ```text
//! RENDER_COST=Notifications cargo test --release -p libero --test render_cost -- --ignored --nocapture
//! ```
//!
//! `span` is a bare element with no scope; `Leaf` the cheapest possible
//! component. The ratios travel between machines, the absolutes do not - so
//! compare a change against a run of `main` on the same machine, not against a
//! number written down anywhere. The current baseline table lives in
//! `.agents/brain/codebase/performance/component-table.md`.
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

/// A row to measure: what to call it, the app, how many instances the app
/// renders (the time is divided by it), and what one round changes.
#[derive(Clone, Copy)]
struct Shape {
    name: &'static str,
    app: fn() -> Element,
    count: usize,
    round: fn(&mut VirtualDom),
}

/// The round of every [`shapes!`] row: the parent re-renders.
fn rerender_app(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
}

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
        &[$(Shape {
            name: $label,
            app: {
                fn app() -> Element {
                    rsx! {
                        LiberoProvider {
                            div { for _ in 0..CHILDREN { $($item)* } }
                        }
                    }
                }
                app as fn() -> Element
            },
            count: CHILDREN,
            round: rerender_app,
        }),*]
    };
}

/// An open floating window. The window itself is crate-private, so the row
/// goes through the hook, opened on mount. `children` so the host re-renders
/// with its parent every round, which re-registers the portal entry and
/// re-renders the window through `PortalOutlet`.
#[component]
fn CostWindow(title: String, children: Element) -> Element {
    let window = libero::hooks::use_floating_window(
        FloatingWindowOptions {
            title: Some(title),
            resizable: true,
            ..Default::default()
        },
        move |_| children.clone(),
    );
    use_hook(|| window.open());
    rsx! {}
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

/// A `Notifications` host showing `N` notifications, all in one stack.
///
/// A singleton, so it gets rows of its own instead of [`CHILDREN`] copies,
/// each of which would portal nine stacks. Its props compare equal, so its
/// parent re-rendering skips it: what redraws it is a store write, which
/// [`notifications_round`] makes.
fn notifications_app<const N: usize>() -> Element {
    let notify = use_notifications();
    let first = use_hook(|| {
        let ids: Vec<_> = (0..N).map(|i| notify.show(format!("n{i}"))).collect();
        ids.first().copied()
    });
    use_context_provider(|| (notify, first));
    rsx! {
        LiberoProvider { Notifications { limit: N } }
    }
}

/// An `update` of the first notification, which redraws the host and every
/// shown notification - they all read the store. With none shown, a `clear`,
/// which redraws the host alone.
fn notifications_round(dom: &mut VirtualDom) {
    dom.in_scope(ScopeId::APP, || {
        let (notify, first) =
            consume_context::<(NotificationHandle<NotificationData>, Option<NotificationId>)>();
        match first {
            Some(id) => notify.update(id, if flip() { "a" } else { "b" }),
            None => notify.clear(),
        }
    });
}

/// The `Notifications` rows. "host" is per host, "item" per shown
/// notification, with the host's share (the "host" row over [`CHILDREN`])
/// still in it.
const NOTIFICATION_SHAPES: &[Shape] = &[
    Shape {
        name: "Notifications host",
        app: notifications_app::<0>,
        count: 1,
        round: notifications_round,
    },
    Shape {
        name: "Notifications item",
        app: notifications_app::<CHILDREN>,
        count: CHILDREN,
        round: notifications_round,
    },
];

/// Whether `RENDER_COST` asks for this row. Unset or empty measures them all,
/// and the controls always run - every row is read against `Leaf`.
fn wanted(label: &str) -> bool {
    let filter = std::env::var("RENDER_COST").unwrap_or_default();
    let names: Vec<&str> = filter
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    if names.is_empty() || label == "span" || label == "Leaf" {
        return true;
    }
    names.iter().any(|name| {
        label
            .get(..name.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(name))
            && matches!(
                label[name.len()..].chars().next(),
                None | Some(' ' | '+' | '-')
            )
    })
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
        .map(|shape| {
            let mut dom = VirtualDom::new(shape.app);
            dom.rebuild(&mut NoOpMutations);
            (shape, dom, u64::MAX)
        })
        .collect();

    for round in 0..ROUNDS {
        FLIP.store(round % 2 == 0, Ordering::Relaxed);
        for (shape, dom, best) in doms.iter_mut() {
            (shape.round)(dom);
            let started = std::time::Instant::now();
            dom.render_immediate(&mut NoOpMutations);
            *best = (*best).min(started.elapsed().as_nanos() as u64);
        }
    }

    doms.into_iter()
        .map(|(shape, _, best)| (shape.name, best as f64 / shape.count as f64))
        .collect()
}

#[test]
#[ignore = "a measurement; needs --release to mean anything"]
fn render_cost_per_component() {
    let shapes = shapes! {
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
        // `children: Element`, so it can never compare equal and always
        // re-renders with its parent - inherent to the wrapper design.
        "Skeleton" { Skeleton { "x" } }
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
        "Accordion" { Accordion { open: AccordionOpen::One(Some(CostPane::One)), onchange: move |_| {}, panel: |_: CostPane| rsx! { "x" } } }
        "Accordion toggle" { Accordion { open: AccordionOpen::One(Some(if flip() { CostPane::Two } else { CostPane::One })), onchange: move |_| {}, panel: |_: CostPane| rsx! { "x" } } }

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

        // Resting: an open window re-rendered with an unchanged title.
        "FloatingWindow" { CostWindow { title: "w", "x" } }
        // A drag frame writes the window's own position signal, which
        // re-renders the whole window, children included. Flipping a prop
        // prices the same full window render; the drag itself cannot be
        // driven here.
        "FloatingWindow drag" { CostWindow { title: if flip() { "a" } else { "b" }, "x" } }
        "Overlay" { Overlay {} }
        "Paper" { Paper { "x" } }
        "Dialog" { Dialog { "x" } }
        "Tooltip" { Tooltip { label: rsx! { "t" }, "x" } }

        "FocusTrap" { FocusTrap { "x" } }
        "VisuallyHidden" { VisuallyHidden { "x" } }
    };
    let shapes: Vec<Shape> = shapes
        .iter()
        .chain(NOTIFICATION_SHAPES)
        .filter(|shape| wanted(shape.name))
        .copied()
        .collect();
    assert!(
        shapes.len() > 2,
        "RENDER_COST matches no row besides the controls"
    );
    let measured = measure(&shapes);

    let cost = |wanted: &str| {
        measured
            .iter()
            .find(|(name, _)| *name == wanted)
            .map(|(_, ns)| *ns)
    };
    let leaf = cost("Leaf").expect("the control is always measured");

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
    // A single-component run skips the ones it did not measure.
    for (name, ceiling) in [("Box", 3.0), ("Text", 5.0), ("Button", 6.0)] {
        let Some(ns) = cost(name) else { continue };
        assert!(
            ns < leaf * ceiling,
            "{name} regressed: {ns:.0} ns, {:.1}x Leaf",
            ns / leaf
        );
    }
}
