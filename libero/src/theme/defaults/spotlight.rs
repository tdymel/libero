use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{ColorValue, CssVar, Size};

pub const SPOTLIGHT_WIDTH: CssVar = CssVar::new("--lsx-spotlight-width");
pub const SPOTLIGHT_TOP_OFFSET: CssVar = CssVar::new("--lsx-spotlight-top-offset");
pub const SPOTLIGHT_MAX_LIST_HEIGHT: CssVar = CssVar::new("--lsx-spotlight-max-list-height");
pub const SPOTLIGHT_PADDING: CssVar = CssVar::new("--lsx-spotlight-padding");
pub const SPOTLIGHT_SEARCH_FONT_SIZE: CssVar = CssVar::new("--lsx-spotlight-search-font-size");
pub const SPOTLIGHT_GROUP_COLOR: CssVar = CssVar::new("--lsx-spotlight-group-color");
pub const SPOTLIGHT_DESCRIPTION_COLOR: CssVar = CssVar::new("--lsx-spotlight-description-color");

/// Every string a `Spotlight` says to a reader. Swapped whole for a locale -
/// the [`ComboboxLabels`](crate::theme::ComboboxLabels) shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpotlightLabels {
    /// Names the dialog, unless `SpotlightOptions::aria_label` does.
    pub label: &'static str,
    pub placeholder: &'static str,
    /// Shown, and announced, when a query matches nothing.
    pub nothing_found: &'static str,
    /// Announced while `SpotlightOptions::loading` is set.
    pub loading: &'static str,
}

impl SpotlightLabels {
    pub const ENGLISH: Self = Self {
        label: "Command palette",
        placeholder: "Search...",
        nothing_found: "Nothing found",
        loading: "Searching",
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpotlightDefaults {
    /// The palette's width, capped by the viewport.
    pub width: &'static str,
    /// How far below the top of the viewport it opens. A palette sits high,
    /// where the eye already is, not centred.
    pub top_offset: &'static str,
    pub max_list_height: &'static str,
    pub radius: Size,
    /// Around the search box and the list, and so the inset a row nests at.
    pub padding: &'static str,
    pub search_font_size: &'static str,
    pub group_color: ColorValue,
    pub description_color: ColorValue,
    pub labels: SpotlightLabels,
}

impl ToCssDeclarations for SpotlightDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            SPOTLIGHT_WIDTH.declare(self.width),
            SPOTLIGHT_TOP_OFFSET.declare(self.top_offset),
            SPOTLIGHT_MAX_LIST_HEIGHT.declare(self.max_list_height),
            SPOTLIGHT_PADDING.declare(self.padding),
            SPOTLIGHT_SEARCH_FONT_SIZE.declare(self.search_font_size),
            SPOTLIGHT_GROUP_COLOR.declare(self.group_color.value()),
            SPOTLIGHT_DESCRIPTION_COLOR.declare(self.description_color.value()),
        ]
    }
}
