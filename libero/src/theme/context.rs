use dioxus::prelude::*;

use crate::LiberoContext;

use super::Theme;

pub fn use_theme() -> Theme {
    use_context::<LiberoContext>().theme
}
