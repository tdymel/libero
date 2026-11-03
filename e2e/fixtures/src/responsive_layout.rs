//! Responsive layout and typography props (todo 5651): each responsive element sits beside
//! two plain twins, `-narrow` (its base value) and `-wide` (its `md` value), to compare against.

use dioxus::prelude::*;
use libero::components::{
    Anchor, Blockquote, Container, Divider, Flex, FlexWrap, Grid, GridArea, GridTemplate,
    GridZone, Header, Kbd, Sidebar, Text, Title,
};
use libero::theme::{Size, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive-layout", || rsx! { ResponsiveLayoutPage {} })];

#[derive(Clone, Copy, PartialEq)]
enum Area {
    Nav,
    Main,
}

impl GridArea for Area {
    fn name(&self) -> &'static str {
        match self {
            Self::Nav => "nav",
            Self::Main => "main",
        }
    }
}

/// `responsive(narrow).md(wide)` and its two plain twins, by id.
macro_rules! trio {
    ($id:literal, $component:ident { $($prop:ident: $narrow:expr => $wide:expr),* } $($child:tt)*) => {
        rsx! {
            $component { id: $id, $($prop: responsive($narrow).md($wide),)* $($child)* }
            $component { id: concat!($id, "-narrow"), $($prop: $narrow,)* $($child)* }
            $component { id: concat!($id, "-wide"), $($prop: $wide,)* $($child)* }
        }
    };
}

#[component]
fn ResponsiveLayoutPage() -> Element {
    let stacked = GridTemplate::new()
        .row(|row| row.cell(Area::Nav))
        .row(|row| row.cell(Area::Main))
        .build()
        .expect("a rectangular template");
    let side = GridTemplate::new()
        .row(|row| row.cell(Area::Nav).cells(Area::Main, 3))
        .build()
        .expect("a rectangular template");
    rsx! {
        {trio!("rl-header", Header { size: Size::Xs => Size::Xl } position: "static", "Banner")}
        {trio!("rl-text", Text { size: Size::Xs => Size::Xl } "Text")}
        {trio!("rl-title", Title { size: Size::Sm => Size::Xl } component: "h2", "Title")}
        {trio!("rl-kbd", Kbd { size: Size::Xs => Size::Xl } "K")}
        {trio!("rl-anchor", Anchor { size: Size::Xs => Size::Xl } to: "/x", "Link")}
        {trio!("rl-quote", Blockquote { size: Size::Xs => Size::Xl } "Quote")}
        {trio!("rl-divider", Divider { size: Size::Xs => Size::Xl, spacing: Size::Xs => Size::Xl })}
        {trio!("rl-container", Container { size: Size::Xs => Size::Sm, gutters: Size::Xs => Size::Xl } "Content")}
        {trio!("rl-sidebar", Sidebar { size: Size::Xs => Size::Lg } aria_label: "Panel", "Panel")}
        {trio!("rl-zone", GridZone { gap: Size::Xs => Size::Xl } "Zone")}
        Text { id: "rl-clamp", line_clamp: responsive(0).md(2), "Clamped from md" }
        Flex { id: "rl-flex", direction: "row", wrap: responsive(FlexWrap::Wrap).md(FlexWrap::NoWrap),
            "A"
        }
        Grid { id: "rl-grid", template: responsive(stacked).md(side),
            GridZone { area: Area::Nav, "Nav" }
            GridZone { area: Area::Main, "Main" }
        }
    }
}
