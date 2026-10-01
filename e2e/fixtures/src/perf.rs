//! Render counts per scope on `/perf/*` pages (todo 821), read and reset through
//! `window.__lsxRenders()` / `window.__lsxRendersReset()`. Other pages keep the default logger.

use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{
    Autocomplete, Badge, Button, Carousel, ChronoPicker, Dialog, Flex, FloatingWindowOptions, Menu,
    MenuEntry, MenuItem, MultiSelect, Options, Pagination, ScrollArea, Select, Slider,
    SliderChangeEvent, Splitter, SpotlightAction, SpotlightOptions, Table, Text, TextField, Tree,
    TreeNode, Virtualize, column, spotlight_filter, use_menu, use_spotlight,
};
use libero::hooks::{ModalScope, use_floating_window, use_modal};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/perf/probe", || rsx! { ProbePage {} }),
    ("/perf/typing", || rsx! { TypingPage {} }),
    ("/perf/select", || rsx! { SelectPage {} }),
    ("/perf/multi-select", || rsx! { MultiSelectPage {} }),
    ("/perf/pagination", || rsx! { PaginationPage {} }),
    ("/perf/scroll", || rsx! { ScrollPage {} }),
    ("/perf/autocomplete", || rsx! { TimedAutocompletePage {} }),
    // Interaction timing (perf::timing): no render counter, so the timings are an app's.
    (
        "/timing/table",
        || rsx! { TimedTablePage { rows: 10_000, windowed: true } },
    ),
    (
        "/timing/table-plain",
        || rsx! { TimedTablePage { rows: 300, windowed: false } },
    ),
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

#[derive(Clone, PartialEq)]
struct Stock {
    name: &'static str,
    count: u32,
}

/// A selectable, sortable table under a 300px cap; `windowed` draws only the rows in view.
#[component]
fn TimedTablePage(rows: u32, windowed: bool) -> Element {
    let mut selection = use_signal(Vec::<String>::new);
    let data: Vec<Stock> = (1..=rows)
        .map(|count| Stock {
            name: ["Apple", "Banana", "Cherry", "Date"][count as usize % 4],
            count,
        })
        .collect();
    rsx! {
        div { id: "pane", style: "width: 480px",
            Table {
                caption: "Stock",
                max_height: "300px",
                virtual_row_height: windowed.then_some(40.0),
                selectable: true,
                selection: selection(),
                onselectionchange: move |next| selection.set(next),
                data,
                columns: vec![
                    column("Name").value(|stock: &Stock| stock.name.to_string()).sortable(),
                    column("Count").value(|stock: &Stock| stock.count).sortable().row_header(),
                ],
                row_key: |stock: &Stock| stock.count.to_string(),
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
