//! The docs shell's heading focus and scroll. The hooks are a copy of
//! `docs/src/heading_focus.rs`: e2e never includes docs source (todo 951);
//! `e2e/src/docs_copy.rs` keeps the copied functions equal (1802). `page_layer_sx`, the page
//! area, the skip link and the palette hooks copy `docs/src/shell.rs`, `doc_tabs_sx`
//! `docs/src/components/doc_page.rs`.

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Button, Container, ScrollArea, ScrollAreaHandle, Tabs, VisuallyHidden, use_scroll_area,
    },
    hooks::{ElementHandle, use_element, use_fullscreen, use_local_storage, use_theme_set},
    platform::ElementApi,
    sx::{Sx, sx},
    theme::{NamedColorCss, TABS_BORDER_COLOR, ThemeSet, Z_INDEX_MODAL},
};

use crate::Routes;

/// Room above a section's heading once it is scrolled to.
const SECTION_MARGIN: f64 = 16.0;

/// Scrolls `area` back to the top whenever `location` changes, or to the pending section.
pub fn use_scroll_reset<T: Clone + PartialEq + 'static>(
    location: T,
    area: ScrollAreaHandle,
    content: ElementHandle,
    section: Signal<Option<String>>,
) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    use_effect(use_reactive!(|location| {
        if *last.peek() != location {
            last.set(location);
            match section.peek().as_deref() {
                Some(id) => scroll_to_section(area, content, id),
                None => area.scroll_to(0.0, 0.0),
            }
        }
    }));
}

fn scroll_to_section(area: ScrollAreaHandle, content: ElementHandle, id: &str) {
    let (Ok(target), Ok(main)) = (
        content.query_selector(&format!("#{id}")),
        content.query_selector("main"),
    ) else {
        return area.scroll_to(0.0, 0.0);
    };
    let (target, main) = (target.client_offset(), main.client_offset());
    spawn(async move {
        match (target.await, main.await) {
            (Ok((_, y)), Ok((_, top))) => area.scroll_to(0.0, y - top - SECTION_MARGIN),
            _ => area.scroll_to(0.0, 0.0),
        }
    });
}

/// Focuses the `main h1` inside `content` whenever `location` changes, or the pending
/// section's heading, never on the first render; waits while `inert`.
pub fn use_heading_focus<T: Clone + PartialEq + 'static>(
    location: T,
    inert: bool,
    content: ElementHandle,
    mut section: Signal<Option<String>>,
) {
    let mut last = use_hook(|| CopyValue::new(location.clone()));
    let mut pending = use_hook(|| CopyValue::new(false));
    use_effect(use_reactive!(|location, inert| {
        if *last.peek() != location {
            last.set(location);
            pending.set(true);
        }
        if *pending.peek() && !inert {
            pending.set(false);
            let heading = section
                .write()
                .take()
                .and_then(|id| content.query_selector(&focus_target(&id)).ok());
            let _ = heading
                .map_or_else(|| content.query_selector("main h1"), Ok)
                .and_then(|h| h.focus());
        }
    }));
}

/// Lands a fresh load's fragment `id` on that section, scroll and heading focus. The
/// docs read `id` above their `Router`, which drops it; here the page passes it.
pub fn use_fragment_landing(
    area: ScrollAreaHandle,
    content: ElementHandle,
    id: Option<&'static str>,
) {
    let mut done = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        if *done.peek() || !content.is_mounted() {
            return;
        }
        done.set(true);
        let Some(id) = id else {
            return;
        };
        scroll_to_section(area, content, id);
        let _ = content
            .query_selector(&focus_target(id))
            .and_then(|heading| heading.focus());
        #[cfg(target_arch = "wasm32")]
        restore_fragment(id);
    });
}

/// A section's title is its first child; a panel without one takes focus itself.
fn focus_target(id: &str) -> String {
    format!("#{id}[tabindex], #{id} > :first-child")
}

/// Puts the fragment back into the URL, so a reload lands here too (todo 1184).
#[cfg(target_arch = "wasm32")]
fn restore_fragment(id: &str) {
    let eval = document::eval(
        "const id = await dioxus.recv();
        history.replaceState(history.state, '', location.pathname + location.search + '#' + id);",
    );
    let _ = eval.send(id);
}

pub const ROUTES: Routes = &[
    ("/docs-shell/heading", || rsx! { HeadingPage {} }),
    ("/docs-shell/scroll", || rsx! { ScrollPage {} }),
    ("/docs-shell/section", || rsx! { SectionPage {} }),
    ("/docs-shell/fragment", || rsx! { FragmentPage {} }),
    ("/docs-shell/fullscreen", || rsx! { FullscreenPage {} }),
    ("/docs-shell/tabs", || rsx! { TabsPage {} }),
    ("/docs-shell/drawer", || rsx! { DrawerPage {} }),
    ("/docs-shell/palette", || rsx! { PalettePage {} }),
];

/// The scrolling page. Inert behind the open drawer, the area and not only `main`: its region
/// stop stayed in the Tab order under the drawer (5310). Omitted closed: even `"false"` enables it.
fn page_area(open: bool, area: ScrollAreaHandle, page: Element) -> Element {
    rsx! {
        ScrollArea {
            handle: area,
            inert: open.then_some(true),
            // The overflowing page is a region tab stop: it needs a name.
            aria_label: "Page",
            sx: sx()
                .flex("1")
                .min_height("0")
                // Default `min-width: auto` let the longest code line push the row
                // wider instead of that block scrolling.
                .min_width("0")
                .and(page_layer_sx()),
            {page}
        }
    }
}

/// `main` takes focus from the skip link only and no keyboard use: without Container's inset ring (5329).
fn landing_sx() -> Sx {
    sx().selector("&:focus", sx().outline("none").box_shadow("none"))
}

/// The skip link's press. An open drawer leaves `main` inert, so it closes first and focus
/// follows once the page is live again, the order of 1693 (5310).
fn use_skip_to_main(mut open: Signal<bool>, main: ElementHandle) -> Callback<()> {
    let mut pending = use_signal(|| false);
    use_effect(move || {
        if pending() && !open() {
            pending.set(false);
            let _ = main.focus();
        }
    });
    use_callback(move |()| {
        if *open.peek() {
            open.set(false);
        }
        pending.set(true);
    })
}

/// WCAG 2.4.1 skip link, off-screen until focused. The click moves focus itself,
/// so the router never sees the fragment.
fn skip_link(skip: Callback<()>) -> Element {
    rsx! {
        // Logical: the shown link sits at the inline start, so RTL keeps it off the header tools.
        VisuallyHidden { focusable: true,
            sx: sx()
                .border_radius("sm")
                .selector("& > a", sx().display("block").padding("sm").color("primary.7")),
            a {
                href: "#docs-main",
                onclick: move |event: MouseEvent| {
                    event.prevent_default();
                    skip.call(());
                },
                "Skip to content"
            }
        }
    }
}

/// The shell's skip link, burger and page area, with a stop after them to Tab to.
#[component]
fn DrawerPage() -> Element {
    let mut open = use_signal(|| false);
    let main_area = use_element();
    let skip = use_skip_to_main(open, main_area);
    let area = use_scroll_area();
    rsx! {
        div { display: "flex", flex_direction: "column", height: "400px",
            {skip_link(skip)}
            Button { id: "burger", onclick: move |_| open.set(!open()), "Menu" }
            div { display: "flex", flex: "1", min_height: "0",
                {page_area(open(), area, rsx! {
                    Container {
                        component: "main",
                        id: "docs-main",
                        tabindex: "-1",
                        onmounted: main_area.mount(),
                        sx: landing_sx(),
                        div { height: "1500px", "The page" }
                    }
                })}
            }
            Button { id: "after", "After the page" }
        }
    }
}

const PALETTE_KEY: &str = "docs-palette";

/// The palette kept from the reader's last pick, for `LiberoProvider`: the library keeps the
/// scheme and direction, the app its palette (5304, 5308). An unknown name gets the default.
fn use_kept_palette() -> ThemeSet {
    let kept = use_local_storage(PALETTE_KEY, String::new);
    use_hook(|| {
        let name = kept.value();
        let name = name.peek();
        ThemeSet::CATALOGUE
            .iter()
            .find(|set| set.name() == name.as_str())
            .map_or_else(ThemeSet::default, |set| (*set).clone())
    })
}

/// Stores a palette the reader picks; a load that picked nothing writes nothing.
fn use_keep_palette() {
    let themes = use_theme_set();
    let mut kept = use_local_storage(PALETTE_KEY, String::new);
    let mut last = use_hook(|| CopyValue::new(themes.name()));
    use_effect(move || {
        let name = themes.name();
        if *last.peek() != name {
            last.set(name);
            kept.set(name.to_string());
        }
    });
}

/// The docs' `App` and shell: a provider that starts on the kept palette, a picker under it.
#[component]
fn PalettePage() -> Element {
    let themes = use_kept_palette();
    rsx! {
        LiberoProvider { themes, PalettePicker {} }
    }
}

#[component]
fn PalettePicker() -> Element {
    use_keep_palette();
    let themes = use_theme_set();
    let name = themes.name();
    let last = ThemeSet::CATALOGUE[ThemeSet::CATALOGUE.len() - 1];
    rsx! {
        p { id: "palette", "{name}" }
        Button { id: "pick", onclick: move |_| themes.set(last.clone()), "{last.name()}" }
    }
}

/// The page area is a stacking context (`ScrollArea`): a drawn fullscreen inside lifts
/// it above the header, else the header covers the box's Exit (5040).
fn page_layer_sx() -> Sx {
    sx().selector(
        "&:has([data-fullscreen='drawn'])",
        sx().z_index(Z_INDEX_MODAL.value()),
    )
}

/// The tab row. Equal tabs that never shrink below icon and label: the tablist scrolls instead.
/// Child combinators, or the Tabs page's demo strip matches too.
fn doc_tabs_sx() -> Sx {
    sx().selector(
        "& > [role=tablist] > [role=tab]",
        sx().flex("1 0 0")
            .min_width("max-content")
            .white_space("nowrap"),
    )
    // A row that scrolls shows a shadow at each edge with tabs beyond it: the covers
    // scroll with the content (`local`) and hide the fixed shadows at either end.
    // In the page's own surface: the paper is lighter than the dark page (todo 4858).
    .selector(
        "& > [role=tablist]",
        sx().with(
            "background-image",
            format!(
                "linear-gradient(to right, {page} 30%, transparent), \
                 linear-gradient(to left, {page} 30%, transparent), \
                 radial-gradient(farthest-side at 0 50%, color-mix(in srgb, currentColor 30%, transparent), transparent), \
                 radial-gradient(farthest-side at 100% 50%, color-mix(in srgb, currentColor 30%, transparent), transparent)",
                page = NamedColorCss::SURFACE.value(),
            ),
        )
        .with("background-repeat", "no-repeat")
        .with("background-position", "0 0, 100% 0, 0 0, 100% 0")
        .with("background-size", "2rem 100%, 2rem 100%, 0.75rem 100%, 0.75rem 100%")
        .with("background-attachment", "local, local, scroll, scroll"),
    )
    // The Tabs line is a background on a phone (todo 5357): this row's replaces it, so it re-adds it on top.
    .media(
        "(pointer: coarse)",
        sx().selector(
            "& > [role=tablist]",
            sx().with(
                "background-image",
                format!(
                    "linear-gradient({line}, {line}), \
                     linear-gradient(to right, {page} 30%, transparent), \
                     linear-gradient(to left, {page} 30%, transparent), \
                     radial-gradient(farthest-side at 0 50%, color-mix(in srgb, currentColor 30%, transparent), transparent), \
                     radial-gradient(farthest-side at 100% 50%, color-mix(in srgb, currentColor 30%, transparent), transparent)",
                    line = TABS_BORDER_COLOR.value(),
                    page = NamedColorCss::SURFACE.value(),
                ),
            )
            .with("background-position", "0 calc(100% - 6px), 0 0, 100% 0, 0 0, 100% 0")
            .with("background-size", "100% 1px, 2rem 100%, 2rem 100%, 0.75rem 100%, 0.75rem 100%")
            .with("background-attachment", "scroll, local, local, scroll, scroll"),
        ),
    )
    // Tight columns drop the icons and tighten the tabs, so four fit (todo 5305). The sidebar
    // keeps the column narrow up to this width, a phone's is 375px or 320px.
    .media(
        "(max-width: 64rem)",
        sx().selector(
            "& > [role=tablist] > [role=tab]",
            sx().padding_inline("xs").font_size("1rem"),
        )
        .selector(
            "& > [role=tablist] > [role=tab] > [data-slot=icon]",
            sx().display("none"),
        ),
    )
    // 16px text in DejaVu needs a 362px phone (5339): this tier ends past it, with 375px still at 16px.
    .media(
        "(max-width: 23rem)",
        sx().selector("& > [role=tablist] > [role=tab]", sx().font_size("0.875rem")),
    )
    // DejaVu, a wide fallback, needs this at 320px: tighter padding would read as one run of words.
    .media(
        "(max-width: 20.5rem)",
        sx().selector("& > [role=tablist] > [role=tab]", sx().font_size("0.8125rem")),
    )
}

/// A docs page's four tabs at the docs page's column width: 1rem of padding each side.
#[component]
fn TabsPage() -> Element {
    let mut tab = use_signal(|| "Usage".to_string());
    let tabs = ["Usage", "Properties", "Style API", "Accessibility"].map(str::to_string);
    rsx! {
        div { width: "calc(100vw - 2rem)",
            Tabs {
                aria_label: "Documentation sections",
                options: tabs.to_vec(),
                value: tab(),
                onchange: move |next| tab.set(next),
                panel: move |selected: String| rsx! { "{selected}" },
                size: "lg",
                full_width: true,
                sx: doc_tabs_sx(),
            }
        }
    }
}

/// The panel refuses the Fullscreen API, as an iframe or WebView does; marks it once set.
const REFUSE: &str = "const panel = document.querySelector('#panel');
    panel.requestFullscreen = () => Promise.reject(new TypeError('refused'));
    panel.dataset.refused = 'true';";

/// The docs shell's layers: a sticky header over the page `ScrollArea`, a panel inside
/// that falls back to the drawn fullscreen.
#[component]
fn FullscreenPage() -> Element {
    let panel = use_element();
    let fullscreen = use_fullscreen(panel);
    use_effect(|| {
        document::eval(REFUSE);
    });
    let drawn = match fullscreen.is_drawn() {
        true => "position: fixed; inset: 0; z-index: var(--lsx-z-index-modal);",
        false => "",
    };
    rsx! {
        div { display: "flex", flex_direction: "column", height: "100vh",
            header {
                position: "sticky",
                top: "0",
                z_index: "var(--lsx-z-index-header)",
                height: "96px",
                flex_shrink: "0",
                background: "Canvas",
                a { href: "#", "Home" }
            }
            div { display: "flex", position: "relative", flex: "1", min_height: "0",
                ScrollArea { id: "page-area", "aria-label": "Page", sx: page_layer_sx().flex("1"),
                    div {
                        id: "panel",
                        onmounted: panel.mount(),
                        style: "{drawn} padding: 16px; background: Canvas;",
                        ..fullscreen.attributes(),
                        Button { id: "toggle", onclick: move |_| fullscreen.toggle(),
                            if fullscreen.is_fullscreen() { "Exit fullscreen" } else { "Fullscreen" }
                        }
                    }
                    div { height: "1500px" }
                }
            }
        }
    }
}

/// Two "pages" behind one signal: the hook only needs a value that changes.
#[component]
fn HeadingPage() -> Element {
    let mut page = use_signal(|| "A");
    let content = use_element();
    let section = use_signal(|| None);
    use_heading_focus(page(), false, content, section);
    rsx! {
        nav { aria_label: "Pages",
            Button { id: "to-a", onclick: move |_| page.set("A"), "Page A" }
            Button { id: "to-b", onclick: move |_| page.set("B"), "Page B" }
        }
        div { onmounted: content.mount(),
            main {
                h1 { tabindex: "-1", "Page {page}" }
            }
        }
    }
}

/// Only the reset: the web's heading focus alone would scroll it back up.
#[component]
fn ScrollPage() -> Element {
    let mut page = use_signal(|| "A");
    let area = use_scroll_area();
    let content = use_element();
    use_scroll_reset(page(), area, content, use_signal(|| None));
    rsx! {
        Button { id: "to-b", onclick: move |_| page.set("B"), "Page B" }
        div { height: "200px",
            ScrollArea { id: "page-area", handle: area, "aria-label": "Page",
                h1 { "Page {page}" }
                div { height: "2000px" }
            }
        }
    }
}

/// Page B's `#far` section below the fold, landed on as the docs' `SectionLink` does.
#[component]
fn SectionPage() -> Element {
    let mut page = use_signal(|| "A");
    let mut section = use_signal(|| None);
    let area = use_scroll_area();
    let content = use_element();
    use_scroll_reset(page(), area, content, section);
    use_heading_focus(page(), false, content, section);
    rsx! {
        Button {
            id: "to-far",
            onclick: move |_| {
                section.set(Some("far".to_string()));
                page.set("B");
            },
            "Page B, far section"
        }
        div { height: "300px",
            ScrollArea { id: "page-area", handle: area, "aria-label": "Page",
                div { display: "contents", onmounted: content.mount(),
                    main {
                        h1 { tabindex: "-1", "Page {page}" }
                        div { height: "1500px" }
                        if page() == "B" {
                            section { id: "far",
                                h2 { tabindex: "-1", "Far" }
                                div { height: "1500px" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A `#far` section below the fold, landed on at the first render (todo 1175).
#[component]
fn FragmentPage() -> Element {
    let area = use_scroll_area();
    let content = use_element();
    use_fragment_landing(area, content, Some("far"));
    rsx! {
        div { height: "300px",
            ScrollArea { id: "page-area", handle: area, "aria-label": "Page",
                div { display: "contents", onmounted: content.mount(),
                    main {
                        h1 { tabindex: "-1", "Fragment" }
                        div { height: "1500px" }
                        section { id: "far",
                            h2 { tabindex: "-1", "Far" }
                            div { height: "1500px" }
                        }
                    }
                }
            }
        }
    }
}
