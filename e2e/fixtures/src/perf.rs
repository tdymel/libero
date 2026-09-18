//! Render counts in the browser (todo 821). On a `/perf/*` page `install`
//! counts dioxus-core's `render` span per scope name; the test reads and resets
//! the counts through `window.__lsxRenders()` / `window.__lsxRendersReset()`.
//! Every other page keeps dioxus's default logger.

use dioxus::prelude::*;
use libero::components::{
    Badge, Flex, MultiSelect, Options, Pagination, ScrollArea, Select, TextField, Virtualize,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/perf/probe", || rsx! { ProbePage {} }),
    ("/perf/typing", || rsx! { TypingPage {} }),
    ("/perf/select", || rsx! { SelectPage {} }),
    ("/perf/multi-select", || rsx! { MultiSelectPage {} }),
    ("/perf/pagination", || rsx! { PaginationPage {} }),
    ("/perf/scroll", || rsx! { ScrollPage {} }),
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
