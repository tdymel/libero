use dioxus::prelude::*;

use crate::{context::LiberoContext, theme::Theme};

pub fn use_theme() -> &'static Theme {
    use_context::<LiberoContext>().theme
}
