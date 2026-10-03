//! Render counts per scope on `/perf/*` pages (todo 821), read and reset through
//! `window.__lsxRenders()` / `window.__lsxRendersReset()`. Other pages keep the default logger.

use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{
    Accordion, AccordionOpen, Autocomplete, Badge, Box, Button, Carousel, Checkbox, ChronoField,
    ChronoPicker, CodeBlock, ColorCode, ColorPicker, Combobox, ComboboxOption, ComboboxOptionArgs,
    DataList, DataListItem, DateRange, DateRangePicker, Dialog, Fields, Flex,
    FloatingWindowOptions, Form, Image, ImageBar, ImageItem, ImageList, List, ListItem, Menu,
    MenuEntry, MenuItem, MonthPicker, MultiSelect, NotificationId, NotificationOptions,
    Notifications, NumberField, Options, Pagination, RadioGroup, RichTextEditor, ScrollArea,
    SegmentedControl, Select, Slider, SliderChangeEvent, Splitter, SpotlightAction,
    SpotlightOptions, Stepper, Switch, Tabs, Text, TextField, Textarea, ThemeSwitcher, Timeline,
    TimelineEvent, Tooltip, Tree, TreeNode, Virtualize, YearPicker, rich_text::Doc,
    spotlight_filter, use_combobox, use_menu, use_notifications, use_spotlight,
};
use libero::hooks::{
    DrawerOptions, ModalScope, PopoverOptions, use_drawer, use_element, use_floating_window,
    use_modal, use_popover,
};
use libero::theme::AutoClose;
use libero::{sx::sx, use_theme};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/perf/probe", || rsx! { ProbePage {} }),
    ("/perf/typing", || rsx! { TypingPage {} }),
    ("/perf/select", || rsx! { SelectPage {} }),
    ("/perf/multi-select", || rsx! { MultiSelectPage {} }),
    ("/perf/pagination", || rsx! { PaginationPage {} }),
    ("/perf/scroll", || rsx! { ScrollPage {} }),
    ("/perf/autocomplete", || rsx! { TimedAutocompletePage {} }),
    ("/perf/slider", || rsx! { TimedSliderPage {} }),
    ("/perf/spotlight", || rsx! { TimedSpotlightPage {} }),
    ("/perf/long-select", || rsx! { LongSelectPage {} }),
    (
        "/perf/long-multi-select",
        || rsx! { LongMultiSelectPage {} },
    ),
    ("/perf/search-select", || rsx! { SearchSelectPage {} }),
    ("/perf/menu", || rsx! { TimedMenuPage {} }),
    ("/perf/popover", || rsx! { PopoverPage {} }),
    ("/perf/tooltip", || rsx! { TooltipPage {} }),
    ("/perf/modal", || rsx! { TimedModalPage {} }),
    ("/perf/drawer", || rsx! { DrawerPage {} }),
    ("/perf/date-picker", || rsx! { DatePickerPage {} }),
    ("/perf/calendar", || rsx! { TimedCalendarPage {} }),
    ("/perf/tabs", || rsx! { TabsPage {} }),
    ("/perf/accordion", || rsx! { AccordionPage {} }),
    ("/perf/segmented", || rsx! { SegmentedPage {} }),
    ("/perf/number-field", || rsx! { NumberFieldPage {} }),
    ("/perf/big-form", || rsx! { BigFormPage {} }),
    ("/perf/color-picker", || rsx! { ColorPickerPage {} }),
    ("/perf/list", || rsx! { ListPage {} }),
    ("/perf/text-field", || rsx! { TimedTextFieldPage {} }),
    ("/perf/textarea", || rsx! { TextareaPage {} }),
    ("/perf/rich-text", || rsx! { RichTextPage {} }),
    ("/perf/button", || rsx! { ButtonPage {} }),
    ("/perf/checkbox", || rsx! { CheckboxPage {} }),
    ("/perf/switch", || rsx! { SwitchPage {} }),
    ("/perf/radio", || rsx! { RadioPage {} }),
    // Interaction timing (perf::timing): no render counter, so the timings are an app's.
    ("/timing/virtualize", || rsx! { TimedVirtualizePage {} }),
    ("/timing/scroll-area", || rsx! { TimedScrollAreaPage {} }),
    (
        "/timing/native-scroll",
        || rsx! { TimedNativeScrollPage {} },
    ),
    ("/timing/carousel", || rsx! { TimedCarouselPage {} }),
    ("/timing/tree", || rsx! { TimedTreePage {} }),
    ("/timing/modal", || rsx! { TimedModalPage {} }),
    ("/timing/menu", || rsx! { TimedMenuPage {} }),
    ("/timing/spotlight", || rsx! { TimedSpotlightPage {} }),
    ("/timing/calendar", || rsx! { TimedCalendarPage {} }),
    ("/timing/text-field", || rsx! { TimedTextFieldPage {} }),
    ("/timing/autocomplete", || rsx! { TimedAutocompletePage {} }),
    ("/timing/slider", || rsx! { TimedSliderPage {} }),
    ("/timing/splitter", || rsx! { TimedSplitterPage {} }),
    (
        "/timing/floating-window",
        || rsx! { TimedFloatingWindowPage {} },
    ),
    ("/timing/long-select", || rsx! { LongSelectPage {} }),
    (
        "/timing/long-multi-select",
        || rsx! { LongMultiSelectPage {} },
    ),
    ("/timing/search-select", || rsx! { SearchSelectPage {} }),
    ("/timing/popover", || rsx! { PopoverPage {} }),
    ("/timing/tooltip", || rsx! { TooltipPage {} }),
    ("/timing/drawer", || rsx! { DrawerPage {} }),
    ("/timing/date-picker", || rsx! { DatePickerPage {} }),
    ("/timing/tabs", || rsx! { TabsPage {} }),
    ("/timing/accordion", || rsx! { AccordionPage {} }),
    ("/timing/segmented", || rsx! { SegmentedPage {} }),
    ("/timing/number-field", || rsx! { NumberFieldPage {} }),
    ("/timing/big-form", || rsx! { BigFormPage {} }),
    ("/timing/color-picker", || rsx! { ColorPickerPage {} }),
    ("/timing/list", || rsx! { ListPage {} }),
    ("/perf/mount", || rsx! { MountPage {} }),
    ("/timing/mount", || rsx! { MountPage {} }),
    ("/timing/textarea", || rsx! { TextareaPage {} }),
    ("/timing/rich-text", || rsx! { RichTextPage {} }),
    ("/timing/button", || rsx! { ButtonPage {} }),
    ("/timing/checkbox", || rsx! { CheckboxPage {} }),
    ("/timing/switch", || rsx! { SwitchPage {} }),
    ("/timing/radio", || rsx! { RadioPage {} }),
    // Todo 2070's scenarios, each counted and timed.
    ("/perf/theme", || rsx! { ThemePage {} }),
    ("/timing/theme", || rsx! { ThemePage {} }),
    ("/perf/tree-500", || rsx! { Tree500Page {} }),
    ("/timing/tree-500", || rsx! { Tree500Page {} }),
    ("/perf/list-scroll", || rsx! { ListScrollPage {} }),
    ("/timing/list-scroll", || rsx! { ListScrollPage {} }),
    ("/perf/data-list-scroll", || rsx! { DataListScrollPage {} }),
    (
        "/timing/data-list-scroll",
        || rsx! { DataListScrollPage {} },
    ),
    ("/perf/menu-100", || rsx! { Menu100Page {} }),
    ("/timing/menu-100", || rsx! { Menu100Page {} }),
    ("/perf/tabs-20", || rsx! { Tabs20Page {} }),
    ("/timing/tabs-20", || rsx! { Tabs20Page {} }),
    ("/perf/toasts", || rsx! { ToastsPage {} }),
    ("/timing/toasts", || rsx! { ToastsPage {} }),
    ("/perf/stepper", || rsx! { StepperPage {} }),
    ("/timing/stepper", || rsx! { StepperPage {} }),
    ("/timing/pagination", || rsx! { PaginationPage {} }),
];

#[cfg(not(target_arch = "wasm32"))]
pub fn install() {}

/// Before `dioxus::launch`, which keeps a subscriber already set.
#[cfg(target_arch = "wasm32")]
pub fn install() {
    use std::{cell::RefCell, collections::BTreeMap, fmt::Debug};

    use tracing::{
        Subscriber,
        field::{Field, Visit},
        span::{Attributes, Id},
    };
    use tracing_subscriber::{
        Layer,
        filter::{LevelFilter, filter_fn},
        layer::{Context, SubscriberExt},
    };
    use wasm_bindgen::{JsValue, closure::Closure};

    thread_local! {
        static COUNTS: RefCell<BTreeMap<String, u32>> = const { RefCell::new(BTreeMap::new()) };
    }

    struct Counter;

    impl<S: Subscriber> Layer<S> for Counter {
        fn on_new_span(&self, attrs: &Attributes<'_>, _: &Id, _: Context<'_, S>) {
            struct Scope(Option<String>);
            impl Visit for Scope {
                fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
                    if field.name() == "scope" {
                        self.0 = Some(format!("{value:?}"));
                    }
                }
            }
            let mut scope = Scope(None);
            attrs.record(&mut scope);
            if let Some(name) = scope.0 {
                COUNTS.with_borrow_mut(|counts| *counts.entry(name).or_default() += 1);
            }
        }
    }

    let Some(window) = web_sys::window() else {
        return;
    };
    // The one switch: every other route keeps dioxus's logger, so its console
    // pass and its timing stay as they were.
    if !window
        .location()
        .pathname()
        .is_ok_and(|path| path.starts_with("/perf/"))
    {
        return;
    }

    // dioxus's own default: its console layer at DEBUG in a debug build.
    let level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    let console = tracing_wasm::WASMLayer::new(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(level)
            .build(),
    )
    .with_filter(LevelFilter::from_level(level));
    let counter = Counter.with_filter(filter_fn(|meta| meta.is_span() && meta.name() == "render"));
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(console).with(counter),
    );

    let read = Closure::<dyn Fn() -> String>::new(|| {
        COUNTS.with_borrow(|counts| {
            let fields: Vec<String> = counts
                .iter()
                .map(|(name, count)| format!("{name:?}:{count}"))
                .collect();
            format!("{{{}}}", fields.join(","))
        })
    });
    let reset = Closure::<dyn Fn()>::new(|| COUNTS.with_borrow_mut(BTreeMap::clear));
    for (name, closure) in [
        ("__lsxRenders", read.as_ref()),
        ("__lsxRendersReset", reset.as_ref()),
    ] {
        let _ = js_sys::Reflect::set(&window, &JsValue::from_str(name), closure);
    }
    read.forget();
    reset.forget();
}

/// The known-good case (todo 463): a closure handler prop re-renders its child
/// on every parent render, a `use_callback` one lets it skip.
#[component]
fn ProbePage() -> Element {
    let mut renders = use_signal(|| 0u32);
    let stable = use_callback(|()| {});

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            button { id: "rerender", onclick: move |_| renders += 1, "Rerender {renders}" }
            ClosureChild { onpress: move |()| {} }
            StableChild { onpress: stable }
        }
    }
}

#[component]
fn ClosureChild(onpress: EventHandler<()>) -> Element {
    rsx! { span { "closure" } }
}

#[component]
fn StableChild(onpress: EventHandler<()>) -> Element {
    rsx! { span { "stable" } }
}

/// A field whose value lives in its own scope, beside content that must not
/// redraw per keystroke.
#[component]
fn TypingPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Typing {}
            for i in 0..10 {
                Badge { key: "{i}", "Tag {i}" }
            }
        }
    }
}

#[component]
fn Typing() -> Element {
    let mut value = use_signal(String::new);
    let oninput = use_callback(move |next: String| value.set(next));
    rsx! {
        TextField { label: "Search", value: value(), oninput }
        span { id: "typed", "data-typed": "{value}" }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    Damson,
    Elderberry,
}

#[component]
fn SelectPage() -> Element {
    let mut value = use_signal(|| None::<Fruit>);
    let onchange = use_callback(move |next: Option<Fruit>| value.set(next));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select::<Fruit> { label: "Fruit", placeholder: "Pick a fruit", value: value(), onchange }
            span { id: "picked", "data-picked": "{value:?}" }
        }
    }
}

#[component]
fn MultiSelectPage() -> Element {
    let mut value = use_signal(Vec::<Fruit>::new);
    let onchange = use_callback(move |next: Vec<Fruit>| value.set(next));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            MultiSelect::<Fruit> { label: "Fruits", placeholder: "Pick fruits", value: value(), onchange }
            span { id: "picked", "data-picked": "{value:?}" }
        }
    }
}

#[component]
fn PaginationPage() -> Element {
    let mut page = use_signal(|| 1u32);
    let onchange = use_callback(move |next: u32| page.set(next));
    rsx! {
        Flex { direction: "column", gap: "md",
            Pagination { total: 20, page: page(), aria_label: "Pages", onchange }
            span { id: "page", "data-page": "{page}" }
        }
    }
}

#[component]
fn ScrollPage() -> Element {
    let item = use_callback(|i: usize| {
        rsx! {
            div { "data-row": i, style: "height: 20px", "Row {i}" }
        }
    });
    rsx! {
        div { id: "pane", style: "height: 200px; width: 240px",
            ScrollArea { "aria-label": "Rows",
                Virtualize { count: 1000, item_size: Some(20.0), item }
            }
        }
    }
}

#[component]
fn TimedVirtualizePage() -> Element {
    let item = use_callback(|i: usize| {
        rsx! {
            div { "data-row": i, style: "height: 24px", "Row {i}" }
        }
    });
    rsx! {
        div { id: "pane", style: "height: 400px; width: 320px",
            ScrollArea { "aria-label": "Rows",
                Virtualize { count: 10_000, item_size: Some(24.0), item }
            }
        }
    }
}

/// Plain content, nothing windowed.
#[component]
fn TimedScrollAreaPage() -> Element {
    rsx! {
        div { id: "pane", style: "height: 400px; width: 320px",
            ScrollArea { "aria-label": "Lines",
                for i in 0..300 {
                    p { key: "{i}", style: "margin: 0; height: 24px", "Line {i}" }
                }
            }
        }
    }
}

/// The same lines in a plain scrolling `div`: the floor under the scroll rows.
#[component]
fn TimedNativeScrollPage() -> Element {
    rsx! {
        div { id: "pane", style: "height: 400px; width: 320px",
            div { style: "height: 100%; overflow: auto",
                for i in 0..300 {
                    p { key: "{i}", style: "margin: 0; height: 24px", "Line {i}" }
                }
            }
        }
    }
}

#[component]
fn TimedCarouselPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Carousel {
                aria_label: "Slides",
                indicators: true,
                slides: (1..=5)
                    .map(|n| rsx! {
                        Text { "Slide {n}" }
                    })
                    .collect(),
            }
        }
    }
}

/// Four branches of 30 leaves, the default row render (chevron, icon).
#[component]
fn TimedTreePage() -> Element {
    let data = (0..4)
        .map(|b| {
            TreeNode::new(format!("b{b}"), format!("Folder {b}")).children(
                (0..30)
                    .map(|l| TreeNode::new(format!("b{b}/{l}"), format!("File {l}.rs")))
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    rsx! {
        Flex { direction: "column", max_width: "320px",
            Tree { aria_label: "Files", data }
        }
    }
}

#[component]
fn TimedModalPage() -> Element {
    let prompt = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog { title: "Unsaved changes", size: "sm",
                Text { "notes.md has changes you have not saved." }
                Button { variant: "text", onclick: move |_| s.close(), "Keep editing" }
                Button { variant: "filled", onclick: move |_| s.close(), "Discard" }
            }
        }
    });
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "open",
                onclick: move |_| {
                    prompt.open();
                },
                "Close editor"
            }
            for i in 0..20 {
                Text { key: "{i}", "Paragraph {i} of the page behind the dialog." }
            }
        }
    }
}

#[component]
fn TimedMenuPage() -> Element {
    let menu = use_menu();
    let items: Vec<MenuEntry> = [
        "New",
        "Open",
        "Save",
        "Save as",
        "Rename",
        "Duplicate",
        "Share",
        "Delete",
    ]
    .into_iter()
    .map(|label| MenuItem::new(label).onselect(|_| {}).into())
    .collect();
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Menu {
                state: menu,
                items,
                Button { id: "open", variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

/// Sixty actions in ten groups.
#[component]
fn TimedSpotlightPage() -> Element {
    let all = use_hook(|| {
        (0..60)
            .map(|i| {
                SpotlightAction::new(format!("Action {i}"))
                    .group(format!("Group {}", i / 6))
                    .description("What the action does")
            })
            .collect::<Vec<_>>()
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        aria_label: Some("Command palette".into()),
        limit: Some(60),
        ..Default::default()
    });
    rsx! {
        Button { id: "open", onclick: move |_| spotlight.open(), "Open the palette" }
    }
}

#[component]
fn TimedCalendarPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 18));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ChronoPicker {
                value: day(),
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                onchange: move |next: Option<NaiveDate>| day.set(next),
            }
        }
    }
}

#[component]
fn TimedTextFieldPage() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TextField { label: "Note", value: value(), oninput: move |next| value.set(next) }
        }
    }
}

/// A hundred options, filtered as the field is typed in.
#[component]
fn TimedAutocompletePage() -> Element {
    let mut value = use_signal(String::new);
    let options = use_hook(|| (0..100).map(|i| format!("City {i}")).collect::<Vec<_>>());
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Autocomplete {
                label: "City",
                options: options.clone(),
                value: value(),
                oninput: move |next| value.set(next),
            }
        }
    }
}

#[component]
fn TimedSliderPage() -> Element {
    let mut value = use_signal(|| 50.0f64);
    rsx! {
        div { style: "width: 400px; padding: 40px 16px",
            Slider {
                aria_label: "Volume",
                value: Some(value()),
                oninput: move |e: SliderChangeEvent<f64>| value.set(e.value()),
            }
        }
    }
}

#[component]
fn TimedSplitterPage() -> Element {
    rsx! {
        div { style: "height: 240px; width: 480px",
            Splitter {
                initial_size: 50.0,
                aria_label: "Resize panes",
                panel_a: rsx! { Text { "Pane A" } },
                panel_b: rsx! { Text { "Pane B" } },
            }
        }
    }
}

#[component]
fn TimedFloatingWindowPage() -> Element {
    let window = use_floating_window(
        FloatingWindowOptions {
            title: Some("Inspector".into()),
            resizable: true,
            ..Default::default()
        },
        |_| {
            rsx! {
                div { style: "width: 320px",
                    Text { "Drag the title bar, or focus it and use the arrow keys." }
                }
            }
        },
    );
    rsx! {
        Button { id: "open", variant: "outlined", onclick: move |_| window.open(), "Inspector" }
    }
}

fn cities(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("City {i}")).collect()
}

/// Fifty options, the trigger a combobox.
#[component]
fn LongSelectPage() -> Element {
    let mut value = use_signal(|| None::<String>);
    let options = use_hook(|| cities(50));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select::<String> { label: "City", options: options.clone(), value: value(), onchange: move |next| value.set(next) }
        }
    }
}

#[component]
fn LongMultiSelectPage() -> Element {
    let mut value = use_signal(Vec::<String>::new);
    let options = use_hook(|| cities(50));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            MultiSelect::<String> { label: "Cities", options: options.clone(), value: value(), onchange: move |next| value.set(next) }
        }
    }
}

/// A searchable select of three hundred, typed into once open.
#[component]
fn SearchSelectPage() -> Element {
    let mut value = use_signal(|| None::<String>);
    let options = use_hook(|| cities(300));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Select::<String> {
                label: "City",
                searchable: true,
                options: options.clone(),
                value: value(),
                onchange: move |next| value.set(next),
            }
        }
    }
}

#[component]
fn PopoverPage() -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    let options = PopoverOptions::new(theme.popover.gap, theme.popover.padding).dismiss(true);
    let popover = use_popover(anchor, opened(), options);
    popover.on_dismiss(move || opened.set(false));
    let floating = *popover.floating();
    popover.show(opened().then(|| {
        rsx! {
            Box {
                role: "dialog",
                aria_label: "Details",
                tabindex: "-1",
                attributes: popover.floating_events(),
                style: popover.style(),
                onmounted: floating.mount(),
                sx: sx().background("surface").padding("8px").z_index("var(--lsx-z-index-popover)"),
                Text { "Popover content" }
                Button { "Inside" }
            }
        }
    }));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "open",
                onmounted: anchor.mount(),
                onclick: move |_| opened.toggle(),
                attributes: popover.anchor_events(),
                aria_haspopup: "dialog",
                aria_expanded: "{opened()}",
                "Details"
            }
        }
    }
}

#[component]
fn TooltipPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Tooltip { label: rsx! { "Saves the draft" }, open_delay: 0, close_delay: 0,
                Button { id: "open", "Save" }
            }
            p { id: "away", style: "height: 200px", "Away" }
        }
    }
}

#[component]
fn DrawerPage() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            aria_label: Some("Menu".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                for line in 0..20 {
                    Text { key: "{line}", "Entry {line}" }
                }
                Button { variant: "text", onclick: move |_| s.close(), "Close" }
            }
        },
    );
    rsx! {
        Button {
            id: "open",
            onclick: move |_| {
                nav.open();
            },
            "Open menu"
        }
    }
}

#[component]
fn DatePickerPage() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 18));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ChronoField::<NaiveDate> {
                label: "Due date",
                today: NaiveDate::from_ymd_opt(2026, 3, 18),
                value: day(),
                onchange: move |next| day.set(next),
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Section {
    Account,
    Profile,
    Billing,
    Security,
    Alerts,
    Devices,
}

fn section_body(section: Section) -> Element {
    rsx! {
        for line in 0..8 {
            Text { key: "{line}", "{section:?} line {line} of the panel." }
        }
    }
}

#[component]
fn TabsPage() -> Element {
    let mut section = use_signal(|| Section::Account);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "520px",
            Tabs {
                aria_label: "Settings",
                value: section(),
                onchange: move |next| section.set(next),
                panel: section_body,
            }
        }
    }
}

#[component]
fn AccordionPage() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(None::<Section>));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "520px",
            Accordion {
                open: open(),
                onchange: move |next| open.set(next),
                panel: section_body,
            }
        }
    }
}

#[component]
fn SegmentedPage() -> Element {
    let mut section = use_signal(|| Section::Account);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "640px",
            SegmentedControl { label: "Section", value: section(), onchange: move |next| section.set(next) }
        }
    }
}

#[component]
fn NumberFieldPage() -> Element {
    let mut value = use_signal(|| None::<i64>);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            NumberField { label: "Amount", steppers: true, value: value(), onchange: move |next| value.set(next) }
        }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Settings {
    t0: String,
    t1: String,
    t2: String,
    t3: String,
    t4: String,
    t5: String,
    t6: String,
    t7: String,
    t8: String,
    t9: String,
    s0: bool,
    s1: bool,
    s2: bool,
    s3: bool,
    s4: bool,
    s5: bool,
    s6: bool,
    s7: bool,
    s8: bool,
    s9: bool,
    c0: bool,
    c1: bool,
    c2: bool,
    c3: bool,
    c4: bool,
    c5: bool,
    c6: bool,
    c7: bool,
    c8: bool,
    c9: bool,
}

/// Thirty bound fields in one `Form`: ten text fields, ten switches, ten checkboxes.
#[component]
fn BigFormPage() -> Element {
    let value = use_store(Settings::default);
    let f = Settings::FIELDS;
    let texts = [
        f.t0(),
        f.t1(),
        f.t2(),
        f.t3(),
        f.t4(),
        f.t5(),
        f.t6(),
        f.t7(),
        f.t8(),
        f.t9(),
    ];
    let switches = [
        f.s0(),
        f.s1(),
        f.s2(),
        f.s3(),
        f.s4(),
        f.s5(),
        f.s6(),
        f.s7(),
        f.s8(),
        f.s9(),
    ];
    let checks = [
        f.c0(),
        f.c1(),
        f.c2(),
        f.c3(),
        f.c4(),
        f.c5(),
        f.c6(),
        f.c7(),
        f.c8(),
        f.c9(),
    ];
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Form { value,
                for (i, name) in texts.into_iter().enumerate() {
                    TextField { key: "t{i}", label: "Text {i}", name }
                }
                for (i, name) in switches.into_iter().enumerate() {
                    Switch { key: "s{i}", id: "s{i}", label: "Switch {i}", name }
                }
                for (i, name) in checks.into_iter().enumerate() {
                    Checkbox { key: "c{i}", id: "c{i}", label: "Check {i}", name }
                }
            }
        }
    }
}

#[component]
fn ColorPickerPage() -> Element {
    let mut color = use_signal(|| "#1c7ed6".parse::<ColorCode>().unwrap());
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            ColorPicker {
                value: color(),
                saturation_label: "Saturation",
                hue_label: "Hue",
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
            }
        }
    }
}

#[component]
fn TextareaPage() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Textarea { label: "Note", value: value(), oninput: move |next| value.set(next) }
        }
    }
}

#[component]
fn RichTextPage() -> Element {
    let mut doc = use_signal(Doc::new);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "520px",
            RichTextEditor { label: "Notes", value: doc(), onchange: move |next| doc.set(next) }
        }
    }
}

/// A press flips the label, so each one changes what the page shows.
#[component]
fn ButtonPage() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "press", onclick: move |_| on.toggle(), "Pressed {on}" }
        }
    }
}

#[component]
fn CheckboxPage() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Checkbox { id: "check", label: "Remember me", checked: on(), onchange: move |next| on.set(next) }
        }
    }
}

#[component]
fn SwitchPage() -> Element {
    let mut on = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Switch { id: "switch", label: "Alerts", checked: on(), onchange: move |next| on.set(next) }
        }
    }
}

#[component]
fn RadioPage() -> Element {
    let mut section = use_signal(|| Some(Section::Account));
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            RadioGroup { label: "Section", value: section(), onchange: move |next| section.set(Some(next)) }
        }
    }
}

/// Forty items, sorted by a button and filtered by a field.
#[component]
fn ListPage() -> Element {
    let mut descending = use_signal(|| false);
    let mut query = use_signal(String::new);
    let all = use_hook(|| cities(40));
    let mut shown: Vec<&String> = all
        .iter()
        .filter(|c| c.contains(query().as_str()))
        .collect();
    if descending() {
        shown.reverse();
    }
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "sort", onclick: move |_| descending.toggle(), "Sort" }
            TextField { label: "Filter", value: query(), oninput: move |next| query.set(next) }
            List {
                for city in shown {
                    ListItem { key: "{city}", "{city}" }
                }
            }
        }
    }
}

/// The scheme toggle over a page of themed controls: every `use_theme()` reader restyles.
#[component]
fn ThemePage() -> Element {
    let mut section = use_signal(|| Section::Account);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "640px",
            div { id: "switcher", ThemeSwitcher {} }
            Tabs {
                aria_label: "Settings",
                value: section(),
                onchange: move |next| section.set(next),
                panel: section_body,
            }
            Flex { gap: "sm",
                for i in 0..12 {
                    Button { variant: if i % 2 == 0 { "filled" } else { "outlined" }, "Action {i}" }
                    Badge { "Tag {i}" }
                }
            }
            for i in 0..6 {
                TextField { label: "Field {i}", value: "", oninput: |_| {} }
                Checkbox { label: "Check {i}", checked: i % 2 == 0, onchange: |_| {} }
                Switch { label: "Switch {i}", checked: i % 2 == 1, onchange: |_| {} }
            }
        }
    }
}

/// Ten branches of fifty leaves: 510 rows once one opens.
#[component]
fn Tree500Page() -> Element {
    let data = (0..10)
        .map(|b| {
            TreeNode::new(format!("b{b}"), format!("Folder {b}")).children(
                (0..50)
                    .map(|l| TreeNode::new(format!("b{b}/{l}"), format!("File {l}.rs")))
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    rsx! {
        Flex { direction: "column", max_width: "320px",
            Tree { aria_label: "Files", data }
        }
    }
}

/// A focusable scroller a key pages through.
const SCROLLER_STYLE: &str = "height: 400px; width: 360px; overflow: auto";

#[component]
fn ListScrollPage() -> Element {
    rsx! {
        div { id: "scroller", tabindex: "0", style: SCROLLER_STYLE,
            List {
                for city in cities(300) {
                    ListItem { key: "{city}", "{city}" }
                }
            }
        }
    }
}

#[component]
fn DataListScrollPage() -> Element {
    rsx! {
        div { id: "scroller", tabindex: "0", style: SCROLLER_STYLE,
            DataList { orientation: "horizontal",
                for i in 0..300 {
                    DataListItem { key: "{i}", label: rsx! { "Term {i}" }, "Description {i}" }
                }
            }
        }
    }
}

#[component]
fn Menu100Page() -> Element {
    let menu = use_menu();
    let items: Vec<MenuEntry> = (0..100)
        .map(|i| MenuItem::new(format!("Action {i}")).onselect(|_| {}).into())
        .collect();
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Menu {
                state: menu,
                items,
                Button { id: "open", variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

#[component]
fn Tabs20Page() -> Element {
    let mut tab = use_signal(|| "City 0".to_string());
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "640px",
            Tabs::<String> {
                aria_label: "Cities",
                options: cities(20),
                value: tab(),
                onchange: move |next| tab.set(next),
                panel: |city: String| rsx! {
                    for line in 0..8 {
                        Text { key: "{line}", "{city} line {line} of the panel." }
                    }
                },
            }
        }
    }
}

/// `#show` adds a toast that stays, `#hide` closes the newest.
#[component]
fn ToastsPage() -> Element {
    let notify = use_notifications();
    let mut shown = use_signal(Vec::<NotificationId>::new);
    rsx! {
        Notifications { limit: 10 }
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "show",
                onclick: move |_| {
                    let n = shown.peek().len();
                    let id = notify.show_with(
                        format!("Saved draft {n}"),
                        NotificationOptions {
                            auto_close: Some(AutoClose::Never),
                            ..Default::default()
                        },
                    );
                    shown.write().push(id);
                },
                "Show"
            }
            Button {
                id: "hide",
                onclick: move |_| {
                    if let Some(id) = shown.write().pop() {
                        notify.hide(id);
                    }
                },
                "Hide"
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Phase {
    Cart,
    Account,
    Shipping,
    Billing,
    Payment,
    Review,
    Confirm,
    Receipt,
    Survey,
}

/// Nine steps, moved by `#next` and `#back`.
#[component]
fn StepperPage() -> Element {
    let mut at = use_signal(|| 0usize);
    let all = Phase::options();
    let value = all[at()];
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "720px",
            Stepper {
                id: "stepper",
                value: Some(value),
                options: all.to_vec(),
                panel: section_body_phase,
            }
            Flex { gap: "sm",
                Button { id: "back", onclick: move |_| at.with_mut(|at| *at = at.saturating_sub(1)), "Back" }
                Button { id: "next", onclick: move |_| at.with_mut(|at| *at = (*at + 1).min(8)), "Next" }
            }
        }
    }
}

fn section_body_phase(phase: Phase) -> Element {
    rsx! {
        for line in 0..8 {
            Text { key: "{line}", "{phase:?} line {line} of the step." }
        }
    }
}

type Mount = fn() -> Element;

/// The mount survey's cases (todo 2031): `#m-<stem>` mounts one under `#mounted`, `#unmount`
/// drops it. `empty` mounts nothing, the floor under every row.
const MOUNTS: &[(&str, Mount)] = &[
    ("empty", || rsx! {}),
    ("list-300", || rsx! { MountList {} }),
    ("tree-1000", || rsx! { MountTree {} }),
    ("timeline-100", || rsx! { MountTimeline {} }),
    ("select-300", || rsx! { MountSelect {} }),
    ("combobox-300", || rsx! { MountCombobox {} }),
    ("accordion-50", || rsx! { MountAccordion {} }),
    ("tabs-20", || rsx! { MountTabs {} }),
    ("image-list-30", || rsx! { MountImageList {} }),
    ("code-block-500", || rsx! { MountCodeBlock {} }),
    ("calendar", || rsx! { MountCalendar {} }),
    ("month-picker", || rsx! { MountMonthPicker {} }),
    ("year-picker", || rsx! { MountYearPicker {} }),
    ("range-picker", || rsx! { MountRangePicker {} }),
    ("data-list-100", || rsx! { MountDataList {} }),
    ("menu-100", || rsx! { MountMenu {} }),
];

#[component]
fn MountPage() -> Element {
    let mut case = use_signal(|| None::<usize>);
    rsx! {
        Flex { direction: "column", gap: "md",
            div {
                for (i, (stem, _)) in MOUNTS.iter().enumerate() {
                    button { key: "{stem}", id: "m-{stem}", onclick: move |_| case.set(Some(i)), "{stem}" }
                }
                button { id: "unmount", onclick: move |_| case.set(None), "Unmount" }
            }
            div { id: "mounted", "data-case": case().map_or("", |i| MOUNTS[i].0), style: "max-width: 640px",
                if let Some(i) = case() {
                    {MOUNTS[i].1()}
                }
            }
        }
    }
}

#[component]
fn MountList() -> Element {
    rsx! {
        List {
            for city in cities(300) {
                ListItem { key: "{city}", "{city}" }
            }
        }
    }
}

/// Twenty branches of fifty leaves.
#[component]
fn MountTree() -> Element {
    let data = (0..20)
        .map(|b| {
            TreeNode::new(format!("b{b}"), format!("Folder {b}")).children(
                (0..50)
                    .map(|l| TreeNode::new(format!("b{b}/{l}"), format!("File {l}.rs")))
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    rsx! { Tree { aria_label: "Files", data } }
}

#[component]
fn MountTimeline() -> Element {
    let items = (0..100)
        .map(|i| {
            TimelineEvent::new(format!("Event {i}")).content(rsx! { "What happened at step {i}." })
        })
        .collect::<Vec<_>>();
    rsx! { Timeline { active: 40, items } }
}

#[component]
fn MountSelect() -> Element {
    let mut value = use_signal(|| None::<String>);
    rsx! {
        Select::<String> { label: "City", options: cities(300), value: value(), onchange: move |next| value.set(next) }
    }
}

#[component]
fn MountCombobox() -> Element {
    let state = use_combobox();
    let mut picked = use_signal(|| "City 0".to_string());
    rsx! {
        Combobox {
            state,
            options: cities(300),
            option: move |row: ComboboxOptionArgs<String>| {
                let value = row.value.clone();
                rsx! {
                    ComboboxOption {
                        selected: picked() == row.value,
                        onpick: move |_| {
                            picked.set(value.clone());
                            state.close();
                        },
                        "{row.value}"
                    }
                }
            },
            Button {
                attributes: state.a11y_attributes(),
                onclick: move |_| state.toggle(),
                onblur: move |_| state.close(),
                "{picked}"
            }
        }
    }
}

#[component]
fn MountAccordion() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(None::<String>));
    rsx! {
        Accordion::<String> {
            options: cities(50),
            open: open(),
            onchange: move |next| open.set(next),
            panel: |city: String| rsx! {
                for line in 0..8 {
                    Text { key: "{line}", "{city} line {line} of the panel." }
                }
            },
        }
    }
}

#[component]
fn MountTabs() -> Element {
    let mut tab = use_signal(|| "City 0".to_string());
    rsx! {
        Tabs::<String> {
            aria_label: "Cities",
            options: cities(20),
            value: tab(),
            onchange: move |next| tab.set(next),
            panel: |city: String| rsx! {
                for line in 0..8 {
                    Text { key: "{line}", "{city} line {line} of the panel." }
                }
            },
        }
    }
}

/// An inline picture, so no request is timed.
const PIXEL: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='4' height='3'%3E%3Crect width='4' height='3' fill='%23889'/%3E%3C/svg%3E";

#[component]
fn MountImageList() -> Element {
    let items = (0..30)
        .map(|i| {
            ImageItem::new(rsx! { Image { src: PIXEL, alt: "Picture {i}", fit: "cover" } })
                .bar(ImageBar::new(rsx! { "Picture {i}" }))
        })
        .collect::<Vec<_>>();
    rsx! { ImageList { cols: 3u8, items } }
}

#[component]
fn MountCodeBlock() -> Element {
    let source = use_hook(|| {
        (0..500)
            .map(|i| format!("let value_{i} = compute({i}, \"line\"); // step {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    });
    rsx! { CodeBlock { source, language: "rust", line_numbers: true } }
}

#[component]
fn MountCalendar() -> Element {
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 3, 18));
    rsx! {
        ChronoPicker { value: day(), today: NaiveDate::from_ymd_opt(2026, 3, 18), onchange: move |next: Option<NaiveDate>| day.set(next) }
    }
}

#[component]
fn MountMonthPicker() -> Element {
    let mut month = use_signal(|| None::<NaiveDate>);
    rsx! {
        MonthPicker { value: month(), today: NaiveDate::from_ymd_opt(2026, 3, 18), onchange: move |v| month.set(v) }
    }
}

#[component]
fn MountYearPicker() -> Element {
    let mut year = use_signal(|| None::<NaiveDate>);
    rsx! {
        YearPicker { value: year(), today: NaiveDate::from_ymd_opt(2026, 3, 18), onchange: move |v| year.set(v) }
    }
}

#[component]
fn MountRangePicker() -> Element {
    let mut range = use_signal(|| None::<DateRange<NaiveDate>>);
    rsx! {
        DateRangePicker { value: range(), today: NaiveDate::from_ymd_opt(2026, 3, 18), onchange: move |v| range.set(v) }
    }
}

#[component]
fn MountDataList() -> Element {
    rsx! {
        DataList { orientation: "horizontal",
            for i in 0..100 {
                DataListItem { key: "{i}", label: rsx! { "Term {i}" }, "Description {i}" }
            }
        }
    }
}

#[component]
fn MountMenu() -> Element {
    let menu = use_menu();
    let items: Vec<MenuEntry> = (0..100)
        .map(|i| MenuItem::new(format!("Action {i}")).onselect(|_| {}).into())
        .collect();
    rsx! {
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}
