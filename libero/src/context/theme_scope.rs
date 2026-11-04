use dioxus::prelude::*;

use crate::theme::Theme;

/// The nearest [`ThemeScope`](crate::ThemeScope): its theme name, which portalled overlays carry
/// out of the tree, and its theme; an unknown inner scope takes both from the scope around it.
#[derive(Clone, Copy)]
pub(crate) struct ThemeScopeContext {
    pub(crate) name: Memo<Option<String>>,
    pub(crate) theme: Memo<Option<&'static Theme>>,
}
