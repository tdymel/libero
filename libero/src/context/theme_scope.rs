use dioxus::prelude::*;

/// The theme name of the nearest [`ThemeScope`](crate::ThemeScope), `None` for an unknown one.
/// Portalled overlays read it to carry the scope out of the tree.
#[derive(Clone, Copy)]
pub(crate) struct ThemeScopeContext(pub(crate) Memo<Option<String>>);
