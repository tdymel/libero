//! The docs site's header row, a copy of `shell_header`'s parts in `docs/src/shell.rs`
//! (todo 951); `e2e/src/docs_copy.rs` keeps the copied functions equal.

use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Box, Burger, Button, ButtonGroup, DirectionToggle, Header, Kbd, Pictogram,
        Repository, SpotlightHandle, SpotlightOptions, ThemeSwitcher, spotlight_filter,
        use_spotlight,
    },
    sx::{Sx, sx},
    theme::{
        ACTION_ICON_RADIUS, BUTTON_HEIGHT, PAPER_BACKGROUND, Size, TITLE_FONT_FAMILY, ThemeSet,
        TitleDefaults,
    },
};
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[("/docs-header", || rsx! { DocsHeaderPage {} })];

const REPO: &str = "tdymel/libero";

/// A `ButtonGroup` item's start as when it is the first: round corners, no seam overlap.
fn first_item_sx() -> Sx {
    sx().border_start_start_radius(ACTION_ICON_RADIUS.overridable())
        .border_end_start_radius(ACTION_ICON_RADIUS.overridable())
        .margin_inline_start("0")
}

#[component]
fn DocsHeaderPage() -> Element {
    let seeded = use_resource(|| async {
        document::eval(
            "sessionStorage.setItem('libero-repo-stars:https://api.github.com/repos/tdymel/libero', '1234'); \
             window.fetch = () => Promise.reject(new TypeError('blocked')); return true;",
        )
        .await
        .is_ok()
    });
    let mut open = use_signal(|| false);
    let search = use_spotlight(SpotlightOptions {
        aria_label: Some("Search the docs".into()),
        actions: Some(Callback::new(|query: String| spotlight_filter(&query, &[]))),
        ..Default::default()
    });
    rsx! {
        if seeded() == Some(true) {
            Header {
                id: "docs-header",
                publish_height: true,
                glass: true,
                sx: header_sx(),
                Burger {
                    id: "docs-burger",
                    open: open(),
                    "aria-controls": "docs-nav",
                    onclick: move |_| open.set(!open()),
                    sx: burger_sx(false),
                }
                // The logo and name, as wide as the docs' own.
                Box {
                    component: "span",
                    sx: sx()
                        .display("flex")
                        .align_items("center")
                        .gap("xs")
                        .breakpoint(Size::Sm, sx().gap("md"))
                        .min_width("0"),
                    Box {
                        component: "span",
                        sx: sx()
                            .width("40px")
                            .breakpoint(Size::Sm, sx().width("72px"))
                            .min_width("0")
                            .height("44px"),
                    }
                    Box {
                        component: "span",
                        sx: sx().font_family(TITLE_FONT_FAMILY.value()).and(TitleDefaults::size_sx(Size::Lg)),
                        "Libero"
                    }
                }
                {search_button(search, "Ctrl")}
                {site_tools(search)}
            }
        }
    }
}

/// Tighter on a phone: the seven controls just miss 320px at `md`.
fn header_sx() -> Sx {
    sx().gap("sm").breakpoint(Size::Sm, sx().gap("md"))
}

/// Hidden from `Sm` up, so the drawer can't open on desktop; the home page has no sidebar.
fn burger_sx(home: bool) -> Sx {
    if home {
        sx().hover(sx().background("muted.1"))
    } else {
        sx().hover(sx().background("muted.1"))
            .breakpoint(Size::Sm, sx().display("none"))
    }
}

/// One joined control.
fn site_tools(search: SpotlightHandle) -> Element {
    rsx! {
        ButtonGroup {
            "aria-label": "Site tools",
            size: "sm",
            // Logical, so the controls stay at the end under RTL. From `Sm` up the
            // search field takes the free space instead.
            sx: sx()
                .margin_inline_start("auto")
                .breakpoint(
                    Size::Sm,
                    sx().margin_inline_start("0")
                        // The hidden phone search still counts as the first child, so
                        // the next item gets back its outer corners and loses the seam.
                        .selector("& > :nth-child(2) :is(button, a)", first_item_sx())
                        .selector("& > :nth-child(2):is(button, a)", first_item_sx()),
                )
                // Under a finger the small icons are boxes of their own: a group's press
                // box grows across it only, and these sit edge to edge.
                .media(
                    "(pointer: coarse)",
                    sx().selector(
                        "& > :is(button, a), & > * :is(button, a)",
                        sx().min_width("44px").min_height("44px"),
                    ),
                ),
            // Below `Sm`, the field's own breakpoint: one of the two always shows, also
            // with a raised default font size (a `px` query and a `rem` one drift apart).
            ActionIcon {
                aria_label: "Search",
                "aria-keyshortcuts": "Control+K Meta+K",
                onclick: move |_| search.open(),
                variant: "outlined",
                color: "muted",
                sx: sx().breakpoint(Size::Sm, sx().display("none")),
                span {
                    display: "inline-flex",
                    width: "18px",
                    height: "18px",
                    Pictogram { icon: lucide::search::outlined }
                }
            }
            // Wide enough for the count, so it arriving late moves no neighbour (todo 3336).
            Repository { repo: REPO, sx: sx().min_width("3.5rem") }
            // Lets a reviewer check any component right to left.
            DirectionToggle {}
            // In the header, so any page can be checked in every scheme and palette.
            ThemeSwitcher { themes: ThemeSet::CATALOGUE }
        }
    }
}

/// Looks like a field, but opens Spotlight, so it stays a button. On a phone the
/// search is an icon button in the site tools. Both carry the shortcut in
/// `aria-keyshortcuts`, not the name.
fn search_button(search: SpotlightHandle, modifier: &'static str) -> Element {
    rsx! {
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
                "{modifier} K"
            }
        }
    }
}
