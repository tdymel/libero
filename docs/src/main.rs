// Matches libero: a `demo/demo.rs` beside its siblings reads better than a
// flattened `mod.rs`.
#![allow(clippy::module_inception)]
// `--wasm-split` makes the router derive one lazy loader fn per route, named
// after the variant (`routeTreePage<hash>`) - 96 `non_snake_case` warnings in
// the release build, on names we never write. An `allow` on the enum does not
// reach them (tried), so it goes here, and only in the split build.
#![cfg_attr(feature = "wasm-split", allow(non_snake_case))]

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Burger, Button, ColorSchemeButton, Container, Flex, Header, Image, Kbd,
        Notifications, ScrollArea, SpotlightOptions, Title, spotlight_filter, use_spotlight,
    },
    hooks::use_element,
    platform::ElementApi,
    sx::sx,
    theme::{HEADER_HEIGHT, ICON_SIZE, PAPER_BACKGROUND, Size, ThemeSet},
};

mod components;
mod icons;
mod nav;
mod pages;
#[cfg(test)]
mod snippets;

use icons::SearchIcon;
use nav::DocsNav;
// A glob, so a new page never edits this file's import list.
use pages::*;

pub(crate) static LOGO: Asset = asset!("/assets/logo.svg");
/// A 16:9 landscape, for docs examples where the logo's square shape hides
/// what the example is about.
pub(crate) static SAMPLE_IMAGE: Asset = asset!("/assets/sample.svg");
/// Stands in for a source that failed to load.
pub(crate) static FALLBACK_IMAGE: Asset = asset!("/assets/fallback.svg");

fn main() {
    #[cfg(feature = "native-cpu")]
    dioxus_native::launch(App);
    #[cfg(not(feature = "native-cpu"))]
    dioxus::launch(App);
}

#[derive(Clone, Routable, PartialEq, Debug)]
pub(crate) enum Route {
    #[layout(AppShell)]
    #[route("/")]
    GettingStarted {},

    #[route("/about/styling")]
    StylingPage {},
    #[route("/about/theming")]
    ThemingPage {},
    #[route("/about/performance")]
    PerformancePage {},
    #[route("/about/hooks")]
    HooksPage {},
    #[route("/about/platform")]
    PlatformPage {},
    #[route("/about/accessibility")]
    AccessibilityPage {},

    #[route("/a11y/focus-trap")]
    FocusTrapPage {},
    #[route("/a11y/visually-hidden")]
    VisuallyHiddenPage {},

    #[route("/data-display/avatar")]
    AvatarPage {},
    #[route("/data-display/badge")]
    BadgePage {},
    #[route("/data-display/data-list")]
    DataListPage {},
    #[route("/data-display/icon")]
    IconPage {},
    #[route("/data-display/image")]
    ImagePage {},
    #[route("/data-display/indicator")]
    IndicatorPage {},
    #[route("/data-display/table")]
    TablePage {},
    #[route("/data-display/timeline")]
    TimelinePage {},
    #[route("/data-display/list")]
    ListPage {},
    #[route("/data-display/marquee")]
    MarqueePage {},
    #[route("/data-display/qr-code")]
    QrCodePage {},

    #[route("/feedback/alert")]
    AlertPage {},
    #[route("/feedback/loader")]
    LoaderPage {},
    #[route("/feedback/notifications")]
    NotificationsPage {},
    #[route("/feedback/progress-bar")]
    ProgressBarPage {},
    #[route("/feedback/skeleton")]
    SkeletonPage {},

    #[route("/form/getting-started")]
    FormGettingStartedPage {},
    #[route("/form/autocomplete")]
    AutocompletePage {},
    #[route("/form/checkbox")]
    CheckboxPage {},
    #[route("/form/chip")]
    ChipPage {},
    #[route("/form/color-field")]
    ColorFieldPage {},
    #[route("/form/color-picker")]
    ColorPickerPage {},
    #[route("/form/combobox")]
    ComboboxPage {},
    #[route("/form/date-field")]
    DateFieldPage {},
    #[route("/form/date-picker")]
    DatePickerPage {},
    #[route("/form/fieldset")]
    FieldsetPage {},
    #[route("/form/file-field")]
    FileFieldPage {},
    #[route("/form/form")]
    FormPage {},
    #[route("/form/multi-select")]
    MultiSelectPage {},
    #[route("/form/cascader")]
    CascaderPage {},
    #[route("/form/native-select")]
    NativeSelectPage {},
    #[route("/form/number-field")]
    NumberFieldPage {},
    #[route("/form/password-field")]
    PasswordFieldPage {},
    #[route("/form/phone-field")]
    PhoneFieldPage {},
    #[route("/form/pin-field")]
    PinFieldPage {},
    #[route("/form/radio-group")]
    RadioGroupPage {},
    #[route("/form/range-slider")]
    RangeSliderPage {},
    #[route("/form/segmented-control")]
    SegmentedControlPage {},
    #[route("/form/select")]
    SelectPage {},
    #[route("/form/slider")]
    SliderPage {},
    #[route("/form/switch")]
    SwitchPage {},
    #[route("/form/tags-field")]
    TagsFieldPage {},
    #[route("/form/text-field")]
    TextFieldPage {},
    #[route("/form/textarea")]
    TextareaPage {},

    #[route("/inputs/action-icon")]
    ActionIconPage {},
    #[route("/inputs/button")]
    ButtonPage {},
    #[route("/inputs/color-scheme-button")]
    ColorSchemeButtonPage {},
    #[route("/layout/aspect-ratio")]
    AspectRatioPage {},
    #[route("/layout/box")]
    BoxPage {},
    #[route("/layout/center")]
    CenterPage {},
    #[route("/layout/collapse")]
    CollapsePage {},
    #[route("/layout/container")]
    ContainerPage {},
    #[route("/layout/divider")]
    DividerPage {},
    #[route("/layout/flex")]
    FlexPage {},
    #[route("/layout/float")]
    FloatPage {},
    #[route("/layout/grid")]
    GridPage {},
    #[route("/layout/header")]
    HeaderPage {},
    #[route("/layout/image-list")]
    ImageListPage {},
    #[route("/layout/scroll-area")]
    ScrollAreaPage {},
    #[route("/layout/scroller")]
    ScrollerPage {},
    #[route("/layout/sidebar")]
    SidebarPage {},
    #[route("/layout/splitter")]
    SplitterPage {},

    #[route("/navigation/burger")]
    BurgerPage {},
    #[route("/navigation/carousel")]
    CarouselPage {},
    #[route("/navigation/anchor")]
    AnchorPage {},
    #[route("/navigation/nav-link")]
    NavLinkPage {},
    #[route("/navigation/pagination")]
    PaginationPage {},
    #[route("/navigation/accordion")]
    AccordionPage {},
    #[route("/navigation/stepper")]
    StepperPage {},
    #[route("/navigation/tabs")]
    TabsPage {},
    #[route("/navigation/menu")]
    MenuPage {},
    #[route("/navigation/menubar")]
    MenubarPage {},
    #[route("/navigation/tree")]
    TreePage {},

    #[route("/overlay/drawer")]
    DrawerPage {},
    #[route("/overlay/floating-window")]
    FloatingWindowPage {},
    #[route("/overlay/hover-card")]
    HoverCardPage {},
    #[route("/overlay/lightbox")]
    LightboxPage {},
    #[route("/overlay/modal")]
    ModalPage {},
    #[route("/overlay/overlay")]
    OverlayPage {},
    #[route("/overlay/popover")]
    PopoverPage {},
    #[route("/overlay/spotlight")]
    SpotlightPage {},
    #[route("/overlay/tooltip")]
    TooltipPage {},

    #[route("/surface/paper")]
    PaperPage {},
    #[route("/surface/dialog")]
    DialogPage {},

    #[route("/typography/blockquote")]
    BlockquotePage {},
    #[route("/typography/code")]
    CodePage {},
    #[route("/typography/code-block")]
    CodeBlockPage {},
    #[route("/typography/kbd")]
    KbdPage {},
    #[route("/typography/mark")]
    MarkPage {},
    #[route("/typography/text")]
    TextPage {},
    #[route("/typography/title")]
    TitlePage {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "Libero" }
        document::Link { rel: "icon", href: LOGO }
        LiberoProvider {
            Router::<Route> {}
            Notifications {}
        }
    }
}

#[component]
fn AppShell() -> Element {
    let mut open = use_signal(|| false);
    let burger = use_element();
    // The docs search: every page, Ctrl/Cmd+K from anywhere.
    let pages = use_hook(nav::page_actions);
    let search = use_spotlight(SpotlightOptions {
        placeholder: Some("Search the docs...".into()),
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &pages)
        })),
        ..Default::default()
    });

    rsx! {
        Flex {
            direction: "column",
            // Header sits flush against the content below - no rounding to
            // the nearest Size step here, this needs to stay exactly 0.
            sx: sx().gap("0"),
            // Escape closes the mobile drawer from anywhere it can be pressed
            // while open: the burger (which holds focus right after opening)
            // or the nav. `main` is inert then, so nothing else hears it.
            // Focus goes back to the burger, as after a page link.
            onkeydown: move |event: KeyboardEvent| {
                if open() && event.key() == Key::Escape {
                    open.set(false);
                    let _ = burger.query_selector("button").and_then(|button| button.focus());
                }
            },
            Header {
                // No `color`: the header is the page's own surface, the way
                // Mantine's is, so it reads as chrome rather than as a
                // banner and follows the colour scheme without a second
                // palette. Translucent plus a blur, so content scrolling
                // under it is suggested rather than hidden - the bar is
                // sticky, and an opaque one reads as a lid.
                sx: sx()
                    .gap("md")
                    .background("color-mix(in srgb, var(--lsx-paper-background) 80%, transparent)")
                    .backdrop_filter("blur(12px)"),
                // Where focus goes when a page link closes the drawer: the
                // link hides with it, and focus would fall to `<body>`.
                // `Burger` takes no `onmounted`, so a `display: contents`
                // wrapper holds the handle.
                div { display: "contents", onmounted: burger.mount(),
                    Burger {
                        open: open(),
                        "aria-controls": "docs-nav",
                        onclick: move |_| open.set(!open()),
                        // No `variant`/`color` prop - the underlying ActionIcon
                        // then contributes no background/color/hover of its own
                        // (same reasoning as `Code`'s copy button), so this `sx`
                        // is the only thing controlling its look. The bars fall
                        // back to `currentColor`, so inheriting the header's
                        // text colour is one declaration fewer.
                        sx: sx()
                            .hover(sx().background("muted.1"))
                            // Only relevant below `Sm` - the burger is the only
                            // way to set `open`, so hiding it here means the
                            // mobile drawer can never actually be open at
                            // desktop widths.
                            .breakpoint(Size::Sm, sx().display("none")),
                    }
                }
                Flex {
                    direction: "row",
                    align: "center",
                    sx: sx().gap("md"),
                    Image { src: LOGO, sx: sx().width("auto").height("28px") }
                    Title { size: "lg", component: "span", "Libero" }
                }
                // Looks like a search field, but it opens the Spotlight
                // dialog, so it stays a button. The name is fixed, so the
                // hidden hint on a phone does not change it; the shortcut
                // rides `aria-keyshortcuts` instead of the name.
                // On a phone the search is an icon button and nothing else -
                // the room belongs to the burger and the title, and that is
                // what Mantine's own mobile header does. From `Sm` up it
                // grows into the field-shaped button, label and shortcut
                // included. Two controls rather than one arm of it, because
                // the two want different shapes; only one is ever rendered.
                ActionIcon {
                    aria_label: "Search",
                    "aria-keyshortcuts": "Control+K Meta+K",
                    onclick: move |_| search.open(),
                    // Same box as the two beside it: three icon buttons in a
                    // row, one of them borderless, reads as an oversight.
                    variant: "outlined",
                    color: "muted",
                    size: "lg",
                    sx: sx()
                        .margin_left("auto")
                        .breakpoint(Size::Sm, sx().display("none")),
                    span {
                        display: "inline-flex",
                        width: "18px",
                        height: "18px",
                        SearchIcon {}
                    }
                }
                // Looks like a search field, but it opens the Spotlight
                // dialog, so it stays a button. The name is fixed, so the
                // shortcut rides `aria-keyshortcuts` instead of the name.
                Button {
                    variant: "standard",
                    aria_label: "Search",
                    "aria-keyshortcuts": "Control+K Meta+K",
                    sx: sx()
                        .display("none")
                        .color("muted.7")
                        // A placeholder's weight, not a button label's.
                        .font_weight("400")
                        // A field's border, from the same step `use_field_frame`
                        // draws one in: `standard` paints its own border
                        // transparent, and on a palette whose paper is close to
                        // its page the field had no edge at all.
                        .border_color("muted.5")
                        // The icon buttons' `lg` box, not a `Button`'s own 36px:
                        // one row of controls, one height.
                        .height(ICON_SIZE.value(Size::Lg))
                        .gap("sm")
                        .hover(sx().background("muted.1"))
                        .breakpoint(
                            Size::Sm,
                            sx()
                                .display("inline-flex")
                                .margin_left("auto")
                                .width("240px")
                                .justify_content("flex-start")
                                .background(PAPER_BACKGROUND.value()),
                        ),
                    onclick: move |_| search.open(),
                    span {
                        display: "inline-flex",
                        width: "16px",
                        height: "16px",
                        SearchIcon {}
                    }
                    "Search"
                    Kbd {
                        sx: sx().margin_left("auto"),
                        "Ctrl K"
                    }
                }
                // In the header rather than on a page: its job is to let a
                // reviewer check any component in every scheme and palette,
                // from wherever they are. `lg`, to match the search beside it.
                ColorSchemeButton { size: "lg", themes: ThemeSet::CATALOGUE }
            }
            // This row itself never scrolls - the nav scrolls its own
            // content internally (`Sidebar` does), and only the rest of the
            // row (everything the nav doesn't take up)
            // should scroll. So `ScrollArea` - not `Container` - is the flex
            // item filling that remaining space; `Container` just sizes/
            // centers the actual page content inside it, and is free to grow
            // past the row's height since `ScrollArea` is what scrolls.
            Flex {
                direction: "row",
                align: "stretch",
                sx: sx().height(format!("calc(100vh - {})", HEADER_HEIGHT.value(Size::Md))),
                DocsNav { open, burger }
                ScrollArea {
                    sx: sx()
                        .flex("1")
                        .min_height("0")
                        // A flex item's default `min-width: auto` means "at
                        // least my content's min-content width" - here
                        // that's whichever single code line on the page is
                        // longest, which can easily exceed the row's actual
                        // available space. Without this, this item refuses
                        // to shrink past that width, so it would push the
                        // *row* wider instead of just that one code block
                        // scrolling internally like it's meant to.
                        .min_width("0"),
                    Container {
                        component: "main",
                        size: "lg",
                        // Unreachable behind the open mobile drawer
                        // otherwise - still in the DOM, just visually
                        // covered. No-op at desktop widths, since `open`
                        // never becomes true there. `inert` is a boolean
                        // HTML attribute - presence (any value, including
                        // "false") is what enables it, so it must be omitted
                        // entirely when not open, not set to the string
                        // "false".
                        inert: open().then_some(true),
                        // `Container`'s own framework `height: 100%` would
                        // otherwise cap it to `ScrollArea`'s viewport height
                        // instead of letting it grow to its real content
                        // height - which is exactly what `ScrollArea` needs
                        // to actually have something to scroll.
                        sx: sx()
                            .height("auto")
                            // Mobile has no room to spare for the desktop
                            // padding - shrink it there, restore it from `Sm`
                            // up.
                            .padding("24px 16px")
                            .breakpoint(Size::Sm, sx().padding("48px 64px")),
                        Outlet::<Route> {}
                    }
                }
            }
        }
    }
}
