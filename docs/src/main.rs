use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Container, Flex, Header, Image, Title},
    sx::sx,
    theme::{Size, SizeCss},
};

mod icons;
mod pages;
mod sidebar;

use pages::{
    ActionIconPage, AnchorPage, BoxPage, ButtonPage, CodePage, ContainerPage, DialogPage,
    DividerPage, DrawerPage, FlexPage, FocusTrapPage, GettingStarted, HeaderPage, IconPage,
    ImagePage, KbdPage, ListPage, MarkPage, ModalPage, NavLinkPage, OverlayPage, QrCodePage,
    SelectPage, TextPage, TitlePage, VisuallyHiddenPage,
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
    rsx! {
        Flex {
            direction: "column",
            gap: "0",
            Header {
                color: "primary",
                sx: sx().gap("md"),
                Image { src: LOGO, sx: sx().width("auto").height("28px") }
                Title { variant: "h3", component: "span", "Libero" }
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
                Sidebar {}
                Container {
                    component: "main",
                    size: "sm",
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
                        .padding("48px 64px"),
                    Outlet::<Route> {}
                }
            }
        }
    }
}
