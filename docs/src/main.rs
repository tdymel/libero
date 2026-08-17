use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Container, Flex, Header, Image, Title},
    sx::sx,
    theme::{Size, SizeCss},
};

mod icons;
mod pages;
mod sidebar;

use icons::BurgerIcon;

use pages::{
    ActionIconPage, AnchorPage, BoxPage, ButtonPage, CodePage, ContainerPage, DataListPage,
    DialogPage, DividerPage, DrawerPage, FlexPage, FocusTrapPage, GettingStarted, HeaderPage,
    IconPage, ImagePage, KbdPage, ListPage, MarkPage, ModalPage, NavLinkPage, OverlayPage,
    QrCodePage, SelectPage, TextPage, TitlePage, TreePage, VisuallyHiddenPage,
};
use sidebar::Sidebar;

pub(crate) static LOGO: Asset = asset!("/assets/logo.svg");

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Routable, PartialEq, Debug)]
pub(crate) enum Route {
    #[layout(AppShell)]
    #[route("/")]
    GettingStarted {},

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
    #[route("/data-display/list")]
    ListPage {},
    #[route("/data-display/qr-code")]
    QrCodePage {},

    #[route("/inputs/action-icon")]
    ActionIconPage {},
    #[route("/inputs/button")]
    ButtonPage {},
    #[route("/inputs/select")]
    SelectPage {},

    #[route("/layout/box")]
    BoxPage {},
    #[route("/layout/container")]
    ContainerPage {},
    #[route("/layout/divider")]
    DividerPage {},
    #[route("/layout/flex")]
    FlexPage {},
    #[route("/layout/header")]
    HeaderPage {},

    #[route("/navigation/anchor")]
    AnchorPage {},
    #[route("/navigation/nav-link")]
    NavLinkPage {},
    #[route("/navigation/tree")]
    TreePage {},

    #[route("/overlay/dialog")]
    DialogPage {},
    #[route("/overlay/drawer")]
    DrawerPage {},
    #[route("/overlay/modal")]
    ModalPage {},
    #[route("/overlay/overlay")]
    OverlayPage {},

    #[route("/typography/code")]
    CodePage {},
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
            gap: "0",
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
                    Title { variant: "h3", component: "span", "Libero" }
                }
            }
            // This row - not the document - is the scrolling element, so its
            // scrollbar only ever covers the area under the header instead
            // of running the full window height. Full-width (unlike
            // `Container`, which is capped/centered), so the scrollbar still
            // lands on the actual window edge and scrolling works anywhere
            // in the row, not just directly over the narrow content column.
            Flex {
                direction: "row",
                align: "stretch",
                // No `flex: 1` here on purpose - its implied `flex-basis: 0%`
                // would compete with the explicit `height` below for sizing
                // this row (and does nothing useful anyway now that the
                // outer column has no definite height of its own to grow
                // into).
                sx: sx()
                    .height(format!("calc(100vh - {})", SizeCss::HEADER_HEIGHT.value(Size::Md)))
                    .overflow("auto"),
                // Same reasoning as `Drawer`'s own `Static` variant: a
                // scrollable region with actual overflow otherwise becomes
                // an implicit tab stop of its own in Chromium, redundant
                // (and confusing) given its content - the sidebar and
                // `Container` - is already separately focusable.
                tabindex: "-1",
                Sidebar { open }
                Container {
                    component: "main",
                    size: "sm",
                    // Unreachable behind the open mobile drawer otherwise -
                    // still in the DOM, just visually covered. No-op at
                    // desktop widths, since `open` never becomes true there.
                    // `inert` is a boolean HTML attribute - presence (any
                    // value, including "false") is what enables it, so it
                    // must be omitted entirely when not open, not set to the
                    // string "false".
                    inert: open().then_some(true),
                    // Two defaults were fighting content-driven sizing here:
                    // the row's `align: stretch` (only kicks in for an `auto`
                    // cross-size, so `height: auto` alone actually invites
                    // it) and Container's own framework `height: 100%` (a
                    // definite size, which applies regardless of stretch).
                    // Both capped Container to the row's short, viewport-
                    // sized height while its real content just overflowed
                    // past that box - burying this padding-bottom inside the
                    // undersized box instead of after the real content end.
                    // `align-self` opts out of the stretch, `height: auto`
                    // overrides the definite 100%; together they let
                    // Container grow to its content, like the row's own
                    // `overflow: auto` already assumes it can.
                    sx: sx()
                        .flex("1")
                        .align_self("flex-start")
                        .height("auto")
                        // A flex item's default `min-width: auto` means "at
                        // least my content's min-content width" - here
                        // that's whichever single code line on the page is
                        // longest, which can easily exceed the row's actual
                        // available space. Without this, Container refuses
                        // to shrink past that width, so the *row* scrolls
                        // horizontally instead of just that one code block
                        // scrolling internally like it's meant to.
                        .min_width("0")
                        // Mobile has no room to spare for the desktop
                        // padding - shrink it there, restore it from `Sm` up.
                        .padding("24px 16px")
                        .breakpoint(Size::Sm, sx().padding("48px 64px")),
                    Outlet::<Route> {}
                }
            }
        }
    }
}
