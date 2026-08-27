use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Container, Flex, Header, Image, ScrollArea, Title},
    sx::sx,
    theme::{HEADER_HEIGHT, Size},
};

mod components;
mod icons;
mod nav;
mod pages;

use icons::BurgerIcon;

use nav::DocsNav;
use pages::{
    ActionIconPage, AnchorPage, AspectRatioPage, BoxPage, ButtonPage, CenterPage, ChipPage,
    CodeBlockPage, CodePage, ComboboxPage, ContainerPage, DataListPage, DialogPage, DividerPage,
    DrawerPage, FlexPage, FloatPage, FocusTrapPage, GettingStarted, GridPage, HeaderPage, IconPage,
    ImagePage, KbdPage, ListPage, MarkPage, ModalPage, NavLinkPage, OverlayPage, PasswordFieldPage,
    PerformancePage, QrCodePage, ScrollAreaPage, SegmentedControlPage, SelectPage, SidebarPage,
    SliderPage, SplitterPage, StylingPage, SwitchPage, TablePage, TabsPage, TextFieldPage,
    TextPage, ThemingPage, TitlePage, TooltipPage, TreePage, VisuallyHiddenPage,
};

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

    #[route("/a11y/focus-trap")]
    FocusTrapPage {},
    #[route("/a11y/visually-hidden")]
    VisuallyHiddenPage {},

    #[route("/data-display/data-list")]
    DataListPage {},
    #[route("/data-display/icon")]
    IconPage {},
    #[route("/data-display/image")]
    ImagePage {},
    #[route("/data-display/table")]
    TablePage {},
    #[route("/data-display/list")]
    ListPage {},
    #[route("/data-display/qr-code")]
    QrCodePage {},

    #[route("/form/password-field")]
    PasswordFieldPage {},
    #[route("/form/select")]
    SelectPage {},
    #[route("/form/text-field")]
    TextFieldPage {},

    #[route("/inputs/action-icon")]
    ActionIconPage {},
    #[route("/inputs/button")]
    ButtonPage {},
    #[route("/inputs/chip")]
    ChipPage {},
    #[route("/inputs/combobox")]
    ComboboxPage {},
    #[route("/inputs/switch")]
    SwitchPage {},
    #[route("/inputs/segmented-control")]
    SegmentedControlPage {},
    #[route("/inputs/slider")]
    SliderPage {},
    #[route("/layout/aspect-ratio")]
    AspectRatioPage {},
    #[route("/layout/box")]
    BoxPage {},
    #[route("/layout/center")]
    CenterPage {},
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
    #[route("/layout/scroll-area")]
    ScrollAreaPage {},
    #[route("/layout/sidebar")]
    SidebarPage {},
    #[route("/layout/splitter")]
    SplitterPage {},

    #[route("/navigation/anchor")]
    AnchorPage {},
    #[route("/navigation/nav-link")]
    NavLinkPage {},
    #[route("/navigation/tabs")]
    TabsPage {},
    #[route("/navigation/tree")]
    TreePage {},

    #[route("/overlay/drawer")]
    DrawerPage {},
    #[route("/overlay/modal")]
    ModalPage {},
    #[route("/overlay/overlay")]
    OverlayPage {},
    #[route("/overlay/tooltip")]
    TooltipPage {},

    #[route("/surface/dialog")]
    DialogPage {},

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
        }
    }
}

#[component]
fn AppShell() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex {
            direction: "column",
            // Header sits flush against the content below - no rounding to
            // the nearest Size step here, this needs to stay exactly 0.
            sx: sx().gap("0"),
            Header {
                color: "primary",
                sx: sx().gap("md"),
                ActionIcon {
                    aria_label: if open() { "Close navigation" } else { "Open navigation" },
                    onclick: move |_| open.set(!open()),
                    // No `variant`/`color` prop - ActionIcon then contributes
                    // no background/color/hover of its own (same reasoning
                    // as `Code`'s copy button), so this `sx` is the only
                    // thing controlling its look. Needed here specifically:
                    // the shade-based hover `variant`/`color` would compute
                    // (tinting `color` a shade lighter) is a no-op on white -
                    // there's no lighter shade of white, so it rendered as a
                    // solid white box instead of a subtle hover. A
                    // translucent white overlay is the actual right look for
                    // a light control against Header's solid primary banner.
                    sx: sx()
                        .color("white")
                        .hover(sx().background("rgba(255, 255, 255, 0.15)"))
                        // Only relevant below `Sm` - the burger is the only
                        // way to set `open`, so hiding it here means the
                        // mobile drawer can never actually be open at
                        // desktop widths.
                        .breakpoint(Size::Sm, sx().display("none")),
                    BurgerIcon {}
                }
                Flex {
                    direction: "row",
                    align: "center",
                    sx: sx().gap("md"),
                    Image { src: LOGO, sx: sx().width("auto").height("28px") }
                    Title { size: "lg", component: "span", "Libero" }
                }
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
                DocsNav { open }
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
