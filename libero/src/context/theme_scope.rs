use dioxus::prelude::*;

use crate::theme::Theme;

/// The nearest [`ThemeScope`](crate::ThemeScope): its theme name, `None` for an unknown one
/// (portalled overlays read it to carry the scope out of the tree), and its theme, which an
/// unknown inner scope takes from the scope around it.
#[derive(Clone, Copy)]
pub(crate) struct ThemeScopeContext {
    pub(crate) name: Memo<Option<String>>,
    pub(crate) theme: Memo<Option<&'static Theme>>,
}
