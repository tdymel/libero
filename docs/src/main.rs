use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Container, Flex, Header, Image, Title},
    sx::sx,
};

mod pages;
mod sidebar;

use pages::{GettingStarted, IconPage};
use sidebar::Sidebar;

static LOGO: Asset = asset!("/assets/logo.svg");

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[layout(AppShell)]
    #[route("/")]
    GettingStarted {},
    #[route("/icon")]
    IconPage {},
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
                sx: sx().flex("1").overflow("hidden"),
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
