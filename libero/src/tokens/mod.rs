//! The vocabulary CSS values are written in: sizes, colors, and the custom
//! properties naming them. Below [`crate::sx`], which types its values with
//! these, and knows nothing of `Theme`, which sits above `sx`.

mod accessibility;
mod color;
mod color_scheme;
mod color_shade;
mod color_value;
mod css_var;
mod direction;
mod focus_ring;
mod hex_color;
mod responsive;
mod size;
mod sizes;

pub use accessibility::{AccessibilityPreferences, Contrast};
pub use color::Color;
pub use color_scheme::{
    COLOR_SCHEME_RESTORE_SCRIPT, COLOR_SCHEME_STORAGE_KEY, ColorScheme, ColorSchemeSetting,
};
pub use color_shade::ColorShade;
pub(crate) use color_shade::ShadeRamp;
pub use color_value::ColorValue;
pub(crate) use color_value::{HOVER_TINT_SHADE, SELECTED_TINT_SHADE};
pub use css_var::{ColorCss, CssVar, NamedColorCss, SizeCss};
pub use direction::Direction;
pub use focus_ring::{
    FOCUS_RING_COLOR, FOCUS_RING_HALO, FOCUS_RING_HALO_SPREAD, FOCUS_RING_HALO_WIDTH,
    FOCUS_RING_OFFSET, FOCUS_RING_WIDTH,
};
pub use hex_color::HexColor;
#[cfg_attr(not(test), allow(unused_imports))]
pub(crate) use hex_color::{Ends, TEXT_CONTRAST};
pub use responsive::{Responsive, responsive};
pub use size::{NegativeSize, Size};
pub use sizes::Sizes;
