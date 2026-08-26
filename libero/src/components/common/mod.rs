mod base_props;
mod class_list;
mod color_variant;
mod dom_api;
mod element_api;
mod input;
mod options;
mod orientation;
mod platform_error;
mod polymorphic;
mod regex_api;
mod states;
mod style_attributes;
mod util;
mod variables;

pub(crate) use base_props::base_props;
pub use class_list::{ClassList, class_list};
pub(crate) use color_variant::{
    base_color, contrast_color, contrast_shade_color, hover_color, selected_color, shade_color,
};
pub use dom_api::{DomApi, dom_api};
pub use element_api::{Dimensions, ElementApi};
pub use input::Input;
pub(crate) use input::input_from_str;
pub use options::{OptionLabel, Options};
// The derive and the trait share a name and one import, the way serde's do.
pub use libero_macros::Options;
pub use orientation::Orientation;
pub use platform_error::PlatformError;
pub use polymorphic::HtmlTag;
pub(crate) use polymorphic::{IntoChildren, render_polymorphic, styling_attributes};
pub(crate) use regex_api::{PreparedText, RegexMatch, regex_api};
pub use states::{States, states};
pub(crate) use style_attributes::{StyleAttributes, use_style_attributes};
pub(crate) use util::{attr, css_string, focus_ring_sx};
pub use variables::{Variables, variables};
