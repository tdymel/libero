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

/// One [`Shape`] per entry: a label, the handlers, and the `rsx!` body to repeat.
///
/// Each `let name = closure;` becomes one `use_callback` in the app, shared by
/// every copy: a closure built in render never compares equal, so passing one
/// straight as a handler or render prop would price the harness, not the
/// component.
macro_rules! shapes {
    ($($label:literal $(let $handler:ident = $callback:expr;)* { $($item:tt)* })*) => {
        &[$(Shape {
            name: $label,
            app: {
                fn app() -> Element {
                    $(let $handler = use_callback($callback);)*
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
/// be - building it is part of what a `Cascader` costs.
fn cost_tree() -> Vec<CascaderOption<String>> {
    vec![
        CascaderOption::new("a", "A").children(vec![
            CascaderOption::new("a1", "A1").children(vec![CascaderOption::new("a1x", "A1x")]),
            CascaderOption::new("a2", "A2"),
        ]),
        CascaderOption::new("b", "B").children(vec![CascaderOption::new("b1", "B1")]),
    ]
}

#[derive(Clone, PartialEq, Default, Fields)]
struct CostForm {
    text: String,
}

/// A field inside a form that re-renders with its parent - the pair below
/// prices reading the form's value against a `value` prop.
#[component]
fn UnboundForm(children: Element) -> Element {
    let oninput = use_callback(|_: String| {});
    rsx! {
        Form::<()> {
            TextField { name: "text", value: "", oninput, validate: not_empty.error("r") }
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

/// An `update` of the first notification, which redraws that notification
/// alone. With none shown, a `clear`, which redraws the host alone.
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

thread_local! {
    /// The notification [`notifications_list_round`] queued, until it hides it.
    static QUEUED: std::cell::Cell<Option<NotificationId>> = const { std::cell::Cell::new(None) };
}

/// A write to the list itself: one round queues a notification past the
/// limit, the next hides it. Either redraws the host, which compares every
/// shown notification and redraws none of them - the price of a `show` or a
/// `hide` with that many on screen.
fn notifications_list_round(dom: &mut VirtualDom) {
    dom.in_scope(ScopeId::APP, || {
        let (notify, _) =
            consume_context::<(NotificationHandle<NotificationData>, Option<NotificationId>)>();
        match QUEUED.take() {
            Some(id) => notify.hide(id),
            None => QUEUED.set(Some(notify.show("queued"))),
        }
    });
}

/// The `Notifications` rows. "host" is per host, "item" per shown
/// notification, with the host's share (the "host" row over [`CHILDREN`])
/// still in it, and "update" is one `update` with [`CHILDREN`] shown.
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
        round: notifications_list_round,
    },
    Shape {
        name: "Notifications update",
        app: notifications_app::<CHILDREN>,
        count: 1,
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
        // Every caller attribute is walked once by `styling_attributes`, which
        // merges `class`, `style` and `aria-describedby` and passes the rest
        // through. Six is more than a real call site spreads, so this is the
        // pessimistic end of what that walk costs.
        "Box+attributes" { Box { id: "i", role: "note", tabindex: "0", "aria-label": "a", "aria-describedby": "d", "data-x": "1", "x" } }
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
        "Virtualize" let item = |_: usize| rsx! { "x" }; { ScrollArea { Virtualize { count: 1, item } } }
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
        "NativeSelect" let onchange = |_: CostPane| {}; { NativeSelect { value: CostPane::One, onchange } }
        "Select" let onchange = |_: Option<CostPane>| {}; { Select { value: CostPane::One, onchange } }
        // Closed, re-rendered: every closed portal holder on the page still
        // publishes its empty slot. `NativeSelect pick` is the same change
        // with no portal.
        "Select pick" let onchange = |_: Option<CostPane>| {}; { Select { value: if flip() { CostPane::Two } else { CostPane::One }, onchange } }
        "NativeSelect pick" let onchange = |_: CostPane| {}; { NativeSelect { value: if flip() { CostPane::Two } else { CostPane::One }, onchange } }
        "MultiSelect" let onchange = |_: Vec<CostPane>| {}; { MultiSelect { value: vec![CostPane::One], onchange } }
        "Autocomplete" let oninput = |_: String| {}; { Autocomplete { value: "", options: vec![CostPane::One], oninput } }
        "TextField" let oninput = |_: String| {}; { TextField { oninput } }
        "TextField+label" let oninput = |_: String| {}; { TextField { oninput, label: "l" } }
        // Every slot filled - what the field chrome costs over a bare control.
        "TextField+slots" let oninput = |_: String| {}; { TextField { oninput, label: "l", description: "d", helper: "h", status: "e", required: true } }
        "NumberField" let onchange = |_: i32| {}; { NumberField { value: 1i32, onchange } }
        "NumberField+steppers" let onchange = |_: i32| {}; { NumberField { value: 1i32, onchange, steppers: true } }
        // A stepper press or an arrow key: the value moves every round.
        "NumberField step" let onchange = |_: i32| {}; { NumberField { value: if flip() { 2i32 } else { 1 }, onchange } }
        "NumberField+steppers step" let onchange = |_: i32| {}; { NumberField { value: if flip() { 2i32 } else { 1 }, onchange, steppers: true } }
        "Textarea" let oninput = |_: String| {}; { Textarea { oninput } }
        "PasswordField" let oninput = |_: String| {}; { PasswordField { oninput } }
        "PasswordField-toggle" let oninput = |_: String| {}; { PasswordField { oninput, reveal_button: false } }
        "PhoneField" let oninput = |_: String| {}; { PhoneField { oninput } }
        // The picker is a `ComboboxCore` shell, a button and a portal - the
        // escape hatch `reveal_button: false` is for `PasswordField`.
        "PhoneField-picker" let oninput = |_: String| {}; { PhoneField { oninput, country_select: false } }
        "TextField+frame" let oninput = |_: String| {}; { TextField { oninput, leading: rsx! { "<" }, trailing: rsx! { ">" } } }
        // Rules that capture nothing compare equal, so this one memoizes.
        "TextField+validate" let oninput = |_: String| {}; { TextField { value: "", oninput, validate: [not_empty.error("r")] } }
        "Form" { Form::<()> { "x" } }
        "Fieldset" { Fieldset::<()> { "x" } }
        // Inside a form: registration and the composite lookup.
        "Form+TextField" let oninput = |_: String| {}; { Form::<()> { TextField { name: "n", oninput } } }
        "Form+TextField+validate" { UnboundForm { "x" } }
        // The same field bound by `name`: it reads the form's signal instead.
        "Form+bound TextField" { BoundForm { "x" } }
        // The only component whose cost scales with a prop - two elements per
        // cell, so the pair below is the per-cell price.
        "PinField" let oninput = |_: String| {}; { PinField { oninput } }
        // Two entries: the dropzone is the bigger surface, and both drive the
        // same hidden input.
        "FileField" let onchange = |_: Files| {}; { FileField { onchange } }
        "FileField-dropzone" let onchange = |_: Files| {}; { FileField { onchange, variant: "dropzone" } }
        "PinField+6" let oninput = |_: String| {}; { PinField { oninput, length: 6usize } }
        // Closed: the columns are not rendered until it opens, and nothing can
        // open one from a prop. The price of an *open* cascader is a browser
        // measurement, not this table's.
        "Cascader" let onchange = |_: Option<String>| {}; { Cascader { data: cost_tree(), onchange } }
        "Checkbox" let onchange = |_: bool| {}; { Checkbox { checked: true, onchange } }
        "Checkbox+label" let onchange = |_: bool| {}; { Checkbox { checked: true, onchange, label: "l" } }
        "Radio" let onselect = |_: ()| {}; { Radio { checked: true, onselect } }
        "RadioGroup" let onchange = |_: CostPane| {}; { RadioGroup { value: CostPane::One, onchange } }
        "Chip" let onchange = |_: bool| {}; { Chip { checked: true, onchange, "x" } }
        "Switch" let onchange = |_: bool| {}; { Switch { checked: true, onchange } }
        "Switch+label" let onchange = |_: bool| {}; { Switch { checked: true, onchange, label: "l" } }
        "Slider" let oninput = |_: SliderChangeEvent| {}; { Slider { value: 50.0, oninput } }
        "Slider+label" let oninput = |_: SliderChangeEvent| {}; { Slider { value: 50.0, oninput, label: "l" } }
        // The second thumb, and what a `Tooltip` costs twice.
        "RangeSlider" let oninput = |_: SliderChangeEvent<(f64, f64)>| {}; { RangeSlider { value: (20.0, 80.0), oninput } }
        // A plain `SliderCore` over a gradient: no bar, no `Tooltip`.
        "HueSlider" let oninput = |_: SliderChangeEvent| {}; { HueSlider { value: 200.0, oninput } }
        "ColorSwatch" { ColorSwatch { color: ColorCode::hex(0x228be6) } }
        // The panel, a hue slider and nothing else - no alpha, no swatches.
        "ColorPicker" let oninput = |_: SliderChangeEvent<ColorCode>| {}; { ColorPicker { value: ColorCode::hex(0x228be6), oninput } }
        // One drag frame. The two hexes differ in hue too, so the hue slider redraws.
        "ColorPicker drag" let oninput = |_: SliderChangeEvent<ColorCode>| {}; { ColorPicker { value: ColorCode::hex(if flip() { 0x228be6 } else { 0x2f8fe0 }), oninput } }
        // A panel drag frame: saturation and value move, the hue stays.
        "ColorPicker pad" let oninput = |_: SliderChangeEvent<ColorCode>| {}; { ColorPicker { value: ColorCode::hsva(208.0, if flip() { 0.6 } else { 0.7 }, 0.8, 1.0), oninput } }
        "ColorPicker hue" let oninput = |_: SliderChangeEvent<ColorCode>| {}; { ColorPicker { value: ColorCode::hsva(if flip() { 208.0 } else { 210.0 }, 0.6, 0.8, 1.0), oninput } }
        "ColorPicker+alpha pad" let oninput = |_: SliderChangeEvent<ColorCode>| {}; { ColorPicker { value: ColorCode::hsva(208.0, if flip() { 0.6 } else { 0.7 }, 0.8, 1.0), oninput, with_alpha: true } }
        // Closed: the dropdown's picker is not rendered until it opens.
        "ColorField" let oninput = |_: SliderChangeEvent<ColorCode>| {}; { ColorField { value: ColorCode::hex(0x228be6), oninput } }
        // 42 day buttons, each with its own click handler.
        "DayPicker" let onchange = |_: Option<NaiveDate>| {}; { DayPicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange } }
        // Seven days in one row, each with a month label.
        "DayPicker pick" let onchange = |_: Option<NaiveDate>| {}; { DayPicker { value: NaiveDate::from_ymd_opt(2026, 9, if flip() { 15 } else { 14 }), onchange } }
        "DayPicker page" let onchange = |_: Option<NaiveDate>| {}; { DayPicker { value: NaiveDate::from_ymd_opt(2026, if flip() { 10 } else { 9 }, 14), onchange } }
        "DayPicker-mini" let onchange = |_: Option<NaiveDate>| {}; { DayPicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange, calendar: "mini" } }
        // Closed: the dropdown's picker is not rendered until it opens.
        "DayField" let onchange = |_: Option<NaiveDate>| {}; { DayField { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange } }
        "TimePicker" let onchange = |_: Option<NaiveTime>| {}; { TimePicker { value: NaiveTime::from_hms_opt(9, 30, 0), onchange, variant: "digital" } }
        "TimePicker pick" let onchange = |_: Option<NaiveTime>| {}; { TimePicker { value: NaiveTime::from_hms_opt(9, if flip() { 35 } else { 30 }, 0), onchange, variant: "digital" } }
        "TimePicker-analog" let onchange = |_: Option<NaiveTime>| {}; { TimePicker { value: NaiveTime::from_hms_opt(9, 30, 0), onchange, variant: "analog" } }
        "TimePicker-analog pick" let onchange = |_: Option<NaiveTime>| {}; { TimePicker { value: NaiveTime::from_hms_opt(9, if flip() { 35 } else { 30 }, 0), onchange, variant: "analog" } }
        "MonthPicker" let onchange = |_: Option<NaiveDate>| {}; { MonthPicker { value: NaiveDate::from_ymd_opt(2026, 9, 1), onchange } }
        "DateRangePicker" let onchange = |_: Option<DateRange<NaiveDate>>| {}; { DateRangePicker { onchange } }
        // A range end moving, as a hover preview does.
        "DateRangePicker end" let onchange = |_: Option<DateRange<NaiveDate>>| {}; { DateRangePicker { value: DateRange::new(NaiveDate::from_ymd_opt(2026, 9, 10).expect("a day"), NaiveDate::from_ymd_opt(2026, 9, if flip() { 15 } else { 14 })), onchange } }
        "TimeField" let onchange = |_: Option<NaiveTime>| {}; { TimeField { value: NaiveTime::from_hms_opt(9, 30, 0), onchange } }
        "SegmentedControl" let onchange = |_: CostPane| {}; { SegmentedControl { value: CostPane::One, onchange } }
        "Tabs" let onchange = |_: CostPane| {}; let panel = |_: CostPane| rsx! { "x" }; { Tabs { value: CostPane::One, onchange, panel } }
        "Accordion" let onchange = |_: AccordionOpen<CostPane>| {}; let panel = |_: CostPane| rsx! { "x" }; { Accordion { open: AccordionOpen::One(Some(CostPane::One)), onchange, panel } }
        "Accordion toggle" let onchange = |_: AccordionOpen<CostPane>| {}; let panel = |_: CostPane| rsx! { "x" }; { Accordion { open: AccordionOpen::One(Some(if flip() { CostPane::Two } else { CostPane::One })), onchange, panel } }

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
