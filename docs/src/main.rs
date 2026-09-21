// Matches libero: a `demo/demo.rs` beside its siblings reads better than a
// flattened `mod.rs`.
#![allow(clippy::module_inception)]

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Anchor, Box, Burger, Button, Container, DirectionToggle, Flex, Header, Icon,
        Kbd, Notifications, RepoButton, ScrollArea, SpotlightOptions, ThemeToggle, Title,
        spotlight_filter, use_scroll_area, use_spotlight,
    },
    hooks::use_element,
    localization::Formats,
    platform::ElementApi,
    sx::sx,
    theme::{HEADER_HEIGHT_VAR, ICON_SIZE, PAPER_BACKGROUND, Size, ThemeSet, Z_INDEX_HEADER},
};

mod components;
mod exports;
mod heading_focus;
mod icons;
#[cfg(test)]
mod index_html;
mod nav;
mod pages;
#[cfg(test)]
mod snippets;

use icons::SearchIcon;
use nav::DocsNav;
// A glob, so a new page never edits this file's import list.
use pages::*;

pub(crate) static LOGO: Asset = asset!("/assets/logo.svg");
const REPO: &str = "tdymel/libero";
const GITHUB: &str = "https://github.com/tdymel/libero";
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
    Home {},

    #[route("/about/getting-started")]
    GettingStarted {},
    #[route("/about/philosophy")]
    PhilosophyPage {},
    #[route("/about/styling")]
    StylingPage {},
    #[route("/about/theming")]
    ThemingPage {},
    #[route("/about/localization")]
    LocalizationPage {},
    #[route("/about/platform")]
    PlatformPage {},

    #[route("/accessibility")]
    AccessibilityPage {},
    #[route("/accessibility/focus-trap")]
    FocusTrapPage {},
    #[route("/accessibility/visually-hidden")]
    VisuallyHiddenPage {},

    #[route("/hooks")]
    HooksPage {},
    #[route("/hooks/use-id")]
    UseIdPage {},
    #[route("/hooks/use-element")]
    UseElementPage {},
    #[route("/hooks/use-focus-return")]
    UseFocusReturnPage {},
    #[route("/hooks/use-drag")]
    UseDragPage {},
    #[route("/hooks/use-theme-set")]
    UseThemeSetPage {},
    #[route("/hooks/use-stylesheet")]
    UseStylesheetPage {},
    #[route("/hooks/use-accessibility")]
    UseAccessibilityPage {},

    #[route("/buttons/action-icon")]
    ActionIconPage {},
    #[route("/buttons/button")]
    ButtonPage {},
    #[route("/buttons/copy-button")]
    CopyButtonPage {},
    #[route("/buttons/direction-toggle")]
    DirectionTogglePage {},
    #[route("/buttons/repo-button")]
    RepoButtonPage {},
    #[route("/buttons/theme-toggle")]
    ThemeTogglePage {},

    #[route("/data-display/accordion")]
    AccordionPage {},
    #[route("/data-display/carousel")]
    CarouselPage {},
    #[route("/data-display/image-list")]
    ImageListPage {},
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
    #[route("/form/chrono-field")]
    ChronoFieldPage {},
    #[route("/form/chrono-picker")]
    ChronoPickerPage {},
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
    #[route("/layout/paper")]
    PaperPage {},
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
    #[route("/navigation/anchor")]
    AnchorPage {},
    #[route("/navigation/nav-link")]
    NavLinkPage {},
    #[route("/navigation/pagination")]
    PaginationPage {},
    #[route("/navigation/stepper")]
    StepperPage {},
    #[route("/navigation/tabs")]
    TabsPage {},
    #[route("/navigation/menubar")]
    MenubarPage {},
    #[route("/navigation/tree")]
    TreePage {},

    #[route("/overlay/dialog")]
    DialogPage {},
    #[route("/overlay/drawer")]
    DrawerPage {},
    #[route("/overlay/floating-window")]
    FloatingWindowPage {},
    #[route("/overlay/hover-card")]
    HoverCardPage {},
    #[route("/overlay/lightbox")]
    LightboxPage {},
    #[route("/overlay/menu")]
    MenuPage {},
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
        LiberoProvider { formats: &Formats::GERMAN,
            Router::<Route> {}
            Notifications {}
        }
    }
}

#[component]
fn AppShell() -> Element {
    let mut open = use_signal(|| false);
    let burger = use_element();
    let content = use_element();
    // The home page is full width: the nav is a drawer at every width there.
    let home = use_route::<Route>() == Route::Home {};
    // The docs search: every page, Ctrl/Cmd+K from anywhere.
    let pages = use_hook(nav::page_actions);
    let search = use_spotlight(SpotlightOptions {
        placeholder: Some("Search the docs...".into()),
        aria_label: Some("Search the docs".into()),
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &pages)
        })),
        ..Default::default()
    });
    let area = use_scroll_area();
    heading_focus::use_scroll_reset(use_route::<Route>(), area);
    heading_focus::use_heading_focus(use_route::<Route>(), content);

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
                sx: sx().gap("md"),
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
                // The way home: the nav has no entry for it. Named by the title.
                Anchor {
                    to: Route::Home {},
                    underline: "never",
                    sx: sx().display("flex").align_items("center").gap("md").color("inherit"),
                    Icon {
                        src: LOGO,
                        variant: "standard",
                        color: "primary",
                        // Wide, and the glyph nearly filling it: at icon size the bars blur together.
                        sx: sx()
                            .width("72px")
                            .height("44px")
                            .with("--lsx-icon-glyph", "84%"),
                    }
                    Title { size: "lg", component: "span", "Libero" }
                }
                // Search: an icon button on a phone, the field-shaped button below from `Sm` up.
                // Both open Spotlight; the shortcut rides `aria-keyshortcuts`, not the name.
                ActionIcon {
                    aria_label: "Search",
                    "aria-keyshortcuts": "Control+K Meta+K",
                    onclick: move |_| search.open(),
                    // Same box as the icon buttons beside it.
                    variant: "outlined",
                    color: "muted",
                    size: "lg",
                    // Logical, so the controls stay at the end under RTL.
                    sx: sx()
                        .margin_inline_start("auto")
                        .breakpoint(Size::Sm, sx().display("none")),
                    span {
                        display: "inline-flex",
                        width: "18px",
                        height: "18px",
                        SearchIcon {}
                    }
                }
                // Looks like a field, but opens Spotlight, so it stays a button.
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
                        // The icon buttons' `lg` box, not a `Button`'s own 36px:
                        // one row of controls, one height.
                        .height(ICON_SIZE.value(Size::Lg))
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
                        SearchIcon {}
                    }
                    "Search"
                    Kbd {
                        sx: sx().margin_left("auto"),
                        "Ctrl K"
                    }
                }
                RepoButton { repo: REPO, size: "lg" }
                // Lets a reviewer check any component right to left.
                DirectionToggle { size: "lg" }
                // In the header, so any page can be checked in every scheme and palette.
                ThemeToggle { size: "lg", themes: ThemeSet::CATALOGUE }
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
