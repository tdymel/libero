mod defaults;
mod stylesheet;
mod theme;
mod theme_set;

// `defaults` curates its own per-component export list.
pub use defaults::*;
pub use theme::Theme;
pub use theme_set::ThemeSet;

// `DARK_SCHEME_QUERY` is read by the web backend only; off wasm nothing asks
// the platform what it is set to.
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_imports))]
pub(crate) use stylesheet::{DARK_SCHEME_QUERY, THEME_ATTRIBUTE};

// Re-exported from `crate::tokens` (a layer below `sx`), so
// `libero::theme::Size` stays the public path.
pub use crate::tokens::{
    COLOR_SCHEME_RESTORE_SCRIPT, COLOR_SCHEME_STORAGE_KEY, Color, ColorCss, ColorScheme,
    ColorSchemeSetting, ColorShade, ColorValue, CssVar, HexColor, NamedColorCss, Size, SizeCss,
    Sizes,
};
