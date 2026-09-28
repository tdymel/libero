use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Anchor, Box, Burger, Button, ButtonGroup, Container, DirectionToggle, Flex,
        Header, Icon, Kbd, Pictogram, Repository, ScrollArea, SpotlightOptions, ThemeSwitcher,
        Title, spotlight_filter, use_scroll_area, use_spotlight,
    },
    hooks::{use_element, use_is_mobile},
    platform::ElementApi,
    sx::sx,
    theme::{BUTTON_HEIGHT, HEADER_HEIGHT_VAR, PAPER_BACKGROUND, Size, ThemeSet, Z_INDEX_HEADER},
};
use pictogram_icons_lucide as lucide;

use crate::{
    Route, heading_focus,
    nav::{self, DocsNav},
    site::{LOGO_INLINE, REPO},
};

#[component]
pub(crate) fn AppShell() -> Element {
    let mut open = use_signal(|| false);
    let burger = use_element();
    let content = use_element();
    // The home page is full width: the nav is a drawer at every width there.
    let route = use_route::<Route>();
    let home = route == Route::Home {};
    let section = use_context_provider(|| heading_focus::PendingSection(Signal::new(None))).0;
    // The docs search: every page, Ctrl/Cmd+K from anywhere.
    let pages = use_hook(|| nav::page_actions(section));
    let search = use_spotlight(SpotlightOptions {
        placeholder: Some("Search the docs...".into()),
        aria_label: Some("Search the docs".into()),
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &pages)
        })),
        ..Default::default()
    });
    let mobile = use_is_mobile();
    let area = use_scroll_area();
    heading_focus::use_scroll_reset(route.clone(), area, content, section);
    heading_focus::use_heading_focus(route.clone(), content, section);
    heading_focus::use_fragment_landing(area, content);
    // Any route change closes the drawer (search, pager, TLDR, body link, back); focus stays put.
    use_effect(use_reactive!(|route| {
        let _ = route;
        if *open.peek() {
            open.set(false);
        }
    }));

    rsx! {
        Flex {
            direction: "column",
            sx: sx().gap("0"),
            // Escape closes the mobile drawer (`main` is inert then) and returns focus to the burger.
            onkeydown: move |event: KeyboardEvent| {
                if open() && event.key() == Key::Escape {
                    open.set(false);
                    let _ = burger.query_selector("button").and_then(|button| button.focus());
                }
            },
            // WCAG 2.4.1 skip link, off-screen until focused. The click moves focus itself,
            // so the router never sees the fragment.
            Box {
                sx: sx()
                    .selector(
                        "& > a",
                        sx().position("fixed")
                            .top("8px")
                            .left("8px")
                            .z_index(format!("calc({} + 1)", Z_INDEX_HEADER.value()))
                            .padding("sm")
                            .border_radius("sm")
                            .background("surface")
                            .color("primary.7")
                            .transform("translateY(-200%)"),
                    )
                    .selector("& > a:focus", sx().transform("none")),
                a {
                    href: "#docs-main",
                    onclick: move |event: MouseEvent| {
                        event.prevent_default();
                        let _ = content.query_selector("main").and_then(|main| main.focus());
                    },
                    "Skip to content"
                }
            }
            Header {
                // No `color`: page surface, reads as chrome. Glass, so content scrolling
                // under the sticky bar shows through.
                publish_height: true,
                glass: true,
                // Tighter on a phone: the seven controls just miss 320px at `md`.
                sx: sx().gap("sm").breakpoint(Size::Sm, sx().gap("md")),
                // Focus target when a page link closes the drawer. `Burger` takes no
                // `onmounted`, so a `display: contents` wrapper holds the handle.
                div { display: "contents", onmounted: burger.mount(),
                    Burger {
                        open: open(),
                        "aria-controls": "docs-nav",
                        onclick: move |_| open.set(!open()),
                        // Hidden from `Sm` up, so the drawer can't open on desktop.
                        // The home page has no sidebar, so its burger stays.
                        sx: if home {
                            sx().hover(sx().background("muted.1"))
                        } else {
                            sx().hover(sx().background("muted.1"))
                                .breakpoint(Size::Sm, sx().display("none"))
                        },
                    }
                }
                // The way home: the nav has no entry for it.
                Anchor {
                    to: Route::Home {},
                    underline: "never",
                    // The logo is the one item that gives way when the row runs short.
                    sx: sx()
                        .display("flex")
                        .align_items("center")
                        .gap("md")
                        .color("inherit")
                        .min_width("0"),
                    // Inline, not `src`: Android's WebView draws the mask of `src` as a solid box.
                    Icon {
                        variant: "standard",
                        color: "primary",
                        // Wide, and the glyph nearly filling it: at icon size the bars blur together.
                        sx: sx()
                            .width("44px")
                            .breakpoint(Size::Sm, sx().width("72px"))
                            .min_width("0")
                            .height("44px")
                            .with("--lsx-icon-glyph", "84%"),
                        span {
                            display: "flex",
                            align_items: "center",
                            justify_content: "center",
                            width: "100%",
                            height: "100%",
                            dangerous_inner_html: LOGO_INLINE.as_str(),
                        }
                    }
                    Title { size: "lg", component: "span", "Libero" }
                }
                // Looks like a field, but opens Spotlight, so it stays a button. On a phone the
                // search is an icon button in the group below. Both carry the shortcut in
                // `aria-keyshortcuts`, not the name.
                Button {
                    variant: "standard",
                    aria_label: "Search",
                    "aria-keyshortcuts": "Control+K Meta+K",
                    sx: sx()
                        .display("none")
                        .color("muted.7")
                        // A placeholder's weight, not a button label's.
                        .font_weight("400")
                        // `use_field_frame`'s border step: `standard`'s transparent border left
                        // no edge on palettes whose paper is close to the page.
                        .border_color("muted.5")
                        // The icon buttons' `sm` box: one row of controls, one height.
                        .height(BUTTON_HEIGHT.value(Size::Sm))
                        .gap("sm")
                        .hover(sx().background("muted.1"))
                        .breakpoint(
                            Size::Sm,
                            sx()
                                .display("inline-flex")
                                .margin_inline_start("auto")
                                .width("240px")
                                .justify_content("flex-start")
                                .background(PAPER_BACKGROUND.value()),
                        ),
                    onclick: move |_| search.open(),
                    span {
                        display: "inline-flex",
                        width: "16px",
                        height: "16px",
                        Pictogram { icon: lucide::search::outlined }
                    }
                    "Search"
                    Kbd {
                        sx: sx().margin_left("auto"),
                        "Ctrl K"
                    }
                }
                // One joined control. The phone search is rendered, not hidden: a hidden first
                // child would still square off the next one's corners.
                ButtonGroup {
                    "aria-label": "Site",
                    size: "sm",
                    // Logical, so the controls stay at the end under RTL. From `Sm` up the
                    // search field takes the free space instead.
                    sx: sx()
                        .margin_inline_start("auto")
                        .breakpoint(Size::Sm, sx().margin_inline_start("0")),
                    if mobile() {
                        ActionIcon {
                            aria_label: "Search",
                            "aria-keyshortcuts": "Control+K Meta+K",
                            onclick: move |_| search.open(),
                            variant: "outlined",
                            color: "muted",
                            span {
                                display: "inline-flex",
                                width: "18px",
                                height: "18px",
                                Pictogram { icon: lucide::search::outlined }
                            }
                        }
                    }
                    Repository { repo: REPO }
                    // Lets a reviewer check any component right to left.
                    DirectionToggle {}
                    // In the header, so any page can be checked in every scheme and palette.
                    ThemeSwitcher { themes: ThemeSet::CATALOGUE }
                }
            }
            // The row never scrolls: the nav scrolls itself, and `ScrollArea` (not
            // `Container`) fills the rest and scrolls the page.
            Flex {
                direction: "row",
                align: "stretch",
                // Nav and page side by side at every width.
                wrap: false,
                // The drawer's containing block.
                sx: sx()
                    .position("relative")
                    .height(format!("calc(100vh - {})", HEADER_HEIGHT_VAR.value())),
                DocsNav { open, burger, drawer: home }
                ScrollArea {
                    handle: area,
                    sx: sx()
                        .flex("1")
                        .min_height("0")
                        // Default `min-width: auto` let the longest code line push the row
                        // wider instead of that block scrolling.
                        .min_width("0"),
                    // The skip link's handle on `main`, as `burger` is on the burger.
                    div { display: "contents", onmounted: content.mount(),
                        Container {
                            component: "main",
                            id: "docs-main",
                            // Focusable by the skip link only, with no ring round the page.
                            tabindex: "-1",
                            size: if home { "xl" } else { "lg" },
                            // Inert behind the open drawer. A boolean attribute: omitted when
                            // closed, since even `"false"` enables it.
                            inert: open().then_some(true),
                            // `Container`'s `height: 100%` would cap it to the viewport: nothing to scroll.
                            sx: sx()
                                .height("auto")
                                .padding("24px 16px")
                                .breakpoint(Size::Sm, sx().padding("48px 64px"))
                                .selector("&:focus", sx().outline("none")),
                            Outlet::<Route> {}
                        }
                    }
                }
            }
        }
    }
}
