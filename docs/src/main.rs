// Matches libero: a `demo/demo.rs` beside its siblings reads better than a
// flattened `mod.rs`.
#![allow(clippy::module_inception)]

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Anchor, Box, Burger, Button, ColorSchemeButton, Container, DirectionToggle,
        Flex, Header, Image, Kbd, Notifications, RepoButton, ScrollArea, SpotlightOptions, Title,
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
    #[route("/hooks/use-clipboard")]
    UseClipboardPage {},
    #[route("/hooks/use-theme")]
    UseThemePage {},
    #[route("/hooks/use-theme-set")]
    UseThemeSetPage {},
    #[route("/hooks/use-color-scheme")]
    UseColorSchemePage {},
    #[route("/hooks/use-localization")]
    UseLocalizationPage {},
    #[route("/hooks/use-localization-handle")]
    UseLocalizationHandlePage {},
    #[route("/hooks/use-formats")]
    UseFormatsPage {},
    #[route("/hooks/use-formats-handle")]
    UseFormatsHandlePage {},
    #[route("/hooks/use-stylesheet")]
    UseStylesheetPage {},
    #[route("/hooks/use-scroll-area")]
    UseScrollAreaPage {},
    #[route("/hooks/use-scroller")]
    UseScrollerPage {},
    #[route("/hooks/use-form")]
    UseFormPage {},
    #[route("/hooks/use-form-context")]
    UseFormContextPage {},
    #[route("/hooks/use-combobox")]
    UseComboboxPage {},
    #[route("/hooks/use-modal")]
    UseModalPage {},
    #[route("/hooks/use-modal-close")]
    UseModalClosePage {},
    #[route("/hooks/use-drawer")]
    UseDrawerPage {},
    #[route("/hooks/use-popover")]
    UsePopoverPage {},
    #[route("/hooks/use-menu")]
    UseMenuPage {},
    #[route("/hooks/use-spotlight")]
    UseSpotlightPage {},
    #[route("/hooks/use-lightbox")]
    UseLightboxPage {},
    #[route("/hooks/use-floating-window")]
    UseFloatingWindowPage {},
    #[route("/hooks/use-notifications")]
    UseNotificationsPage {},
    #[route("/hooks/use-notifications-with")]
    UseNotificationsWithPage {},

    #[route("/buttons/action-icon")]
    ActionIconPage {},
    #[route("/buttons/button")]
    ButtonPage {},
    #[route("/buttons/color-scheme-button")]
    ColorSchemeButtonPage {},
    #[route("/buttons/direction-toggle")]
    DirectionTogglePage {},
    #[route("/buttons/repo-button")]
    RepoButtonPage {},

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
            // WCAG 2.4.1: the first tab stop skips the header and the nav.
            // Off-screen until focused; the click moves focus itself, so the
            // router never sees the fragment.
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
                // No `color`: the header is the page's own surface, so it
                // reads as chrome rather than as a
                // banner and follows the colour scheme without a second
                // palette. Translucent plus a blur, so content scrolling
                // under it is suggested rather than hidden - the bar is
                // sticky, and an opaque one reads as a lid.
                publish_height: true,
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
                        // Only relevant below `Sm` - the burger is the only
                        // way to set `open`, so hiding it here means the
                        // mobile drawer can never actually be open at
                        // desktop widths. The home page has no sidebar, so
                        // its burger stays.
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
                    Image { src: LOGO, decorative: true, sx: sx().width("auto").height("28px") }
                    Title { size: "lg", component: "span", "Libero" }
                }
                // Looks like a search field, but it opens the Spotlight
                // dialog, so it stays a button. The name is fixed, so the
                // hidden hint on a phone does not change it; the shortcut
                // rides `aria-keyshortcuts` instead of the name.
                // On a phone the search is an icon button and nothing else -
                // the room belongs to the burger and the title. From `Sm` up it
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
                        // A flex item's default `min-width: auto` means "at
                        // least my content's min-content width" - here
                        // that's whichever single code line on the page is
                        // longest, which can easily exceed the row's actual
                        // available space. Without this, this item refuses
                        // to shrink past that width, so it would push the
                        // *row* wider instead of just that one code block
                        // scrolling internally like it's meant to.
                        .min_width("0"),
                    // The skip link's handle on `main`, as `burger` is on the burger.
                    div { display: "contents", onmounted: content.mount(),
                        Container {
                            component: "main",
                            id: "docs-main",
                            // Focusable by the skip link only, with no ring round the page.
                            tabindex: "-1",
                            size: if home { "xl" } else { "lg" },
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
