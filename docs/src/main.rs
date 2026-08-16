use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Container, Flex, Header, Image, Title},
    sx::sx,
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
            sx: sx().height("100vh"),
            gap: "0",
            Header {
                color: "primary",
                sx: sx().gap("md"),
                Image { src: LOGO, sx: sx().width("auto").height("28px") }
                Title { variant: "h3", component: "span", "Libero" }
            }
            Flex {
                direction: "row",
                align: "stretch",
                sx: sx().flex("1").min_height("0").overflow("hidden"),
                Sidebar {}
                Container {
                    component: "main",
                    size: "sm",
                    sx: sx().flex("1").padding("48px 64px").overflow("auto"),
                    Outlet::<Route> {}
                }
            }
        }
    }
}
