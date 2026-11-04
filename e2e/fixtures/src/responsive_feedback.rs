//! Responsive feedback, navigation and overlay props (todo 5655): each responsive element sits
//! beside two plain twins, `-narrow` (its base value) and `-wide` (its `md` value).

use dioxus::prelude::*;
use libero::components::{
    Burger, CircularProgress, Dialog, Loader, MenuItem, Menubar, MenubarMenu, Pagination,
    ProgressBar, Skeleton,
};
use libero::sx::ThemeAwareValue;
use libero::theme::{Size, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive-feedback", || rsx! { ResponsiveFeedbackPage {} })];

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

fn css(value: &str) -> ThemeAwareValue {
    ThemeAwareValue::from(value)
}

fn step(size: Size) -> ThemeAwareValue {
    ThemeAwareValue::from(size)
}

fn menus() -> Vec<MenubarMenu> {
    vec![MenubarMenu::new("File", vec![MenuItem::new("New").into()])]
}

#[component]
fn ResponsiveFeedbackPage() -> Element {
    rsx! {
        {trio!("rf-loader", Loader { size: Size::Xs => Size::Xl })}
        {trio!("rf-progress", ProgressBar { size: Size::Xs => Size::Xl, radius: step(Size::Xs) => css("0") } aria_label: "Upload", value: 40.0)}
        {trio!("rf-circular", CircularProgress { size: Size::Xs => Size::Xl } aria_label: "Upload", value: 40.0)}
        {trio!("rf-skeleton", Skeleton { height: css("20px") => css("60px"), width: css("40px") => css("120px"), radius: step(Size::Xs) => step(Size::Xl) })}
        {trio!("rf-pagination", Pagination { size: Size::Xs => Size::Xl, radius: step(Size::Xs) => step(Size::Xl) } total: 5, page: 2, aria_label: "Pages", onchange: |_| {})}
        {trio!("rf-menubar", Menubar { size: Size::Xs => Size::Xl, radius: step(Size::Xs) => css("0") } aria_label: "Main", menus: menus())}
        {trio!("rf-burger", Burger { size: step(Size::Xs) => step(Size::Xl) })}
        {trio!("rf-dialog", Dialog { size: step(Size::Xs) => step(Size::Lg), radius: step(Size::Xs) => css("0") } aria_label: "Inline", "Content")}
    }
}
