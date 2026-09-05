use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, Sizes};

pub const PAGINATION_CONTROL_SIZE: SizeCss = SizeCss::new("--lsx-pagination-control-size-");
pub const PAGINATION_FONT_SIZE: SizeCss = SizeCss::new("--lsx-pagination-font-size-");

pub const PAGINATION_GAP: CssVar = CssVar::new("--lsx-pagination-gap");
pub const PAGINATION_BORDER: CssVar = CssVar::new("--lsx-pagination-border");
pub const PAGINATION_ACTIVE_BACKGROUND: CssVar = CssVar::new("--lsx-pagination-active-background");
pub const PAGINATION_ACTIVE_COLOR: CssVar = CssVar::new("--lsx-pagination-active-color");

/// The shade a bare `Color` fills the current page with, matching
/// `base_color`'s own default so a themed color and a prop-supplied one land on
/// the same step of the ramp.
const ACTIVE_SHADE: ColorShade = ColorShade::S6;

/// Geometry and colour. The strings a reader sees live in
/// [`PaginationLabels`], which is swapped whole for a locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationDefaults {
    pub size: Size,
    pub radius: Size,
    /// Fill of the current page. A bare [`Color`] rather than a `ColorValue`,
    /// the `NavLinkDefaults` shape, so the contrast twin is derivable.
    pub color: Color,
    pub siblings: u8,
    pub boundaries: u8,
    pub gap: Size,
    /// Control box, in px. Mantine's scale plus an `xxl`.
    pub control_size: Sizes<u16>,
    pub font_size: Sizes<u16>,
    /// `ColorValue`, not a `&'static str`: a raw `"grey.4"` is declared
    /// verbatim, and a bare palette token is not a CSS colour.
    pub border: ColorValue,
}

impl PaginationDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        color: Color::Primary,
        siblings: 1,
        boundaries: 1,
        gap: Size::Xs,
        // Mantine's control scale, plus an `xxl` continuing its steps.
        control_size: Sizes::new(22, 26, 32, 38, 44, 52),
        font_size: Sizes::new(11, 12, 14, 16, 18, 20),
        border: ColorValue::Shade(Color::Grey, ColorShade::S4),
    };

    fn size_sx(size: Size) -> Sx {
        sx().min_width(PAGINATION_CONTROL_SIZE.value(size))
            .height(PAGINATION_CONTROL_SIZE.value(size))
            .font_size(PAGINATION_FONT_SIZE.value(size))
    }

    /// The `Kbd` shape: the non-size declarations once, then a per-size block,
    /// so every `Pagination` on a page shares one recycled class.
    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for PaginationDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let active = ColorValue::Fill(self.color, ACTIVE_SHADE);
        let on_active = ColorValue::Contrast(self.color, ACTIVE_SHADE);

        let mut declarations = self
            .control_size
            .to_css_declarations(PAGINATION_CONTROL_SIZE, "px");
        declarations.extend(
            self.font_size
                .to_css_declarations(PAGINATION_FONT_SIZE, "px"),
        );
        declarations.push(PAGINATION_GAP.declare(SizeCss::SPACING.value(self.gap)));
        declarations.push(PAGINATION_BORDER.declare(self.border.value()));
        declarations.push(PAGINATION_ACTIVE_BACKGROUND.declare(active.value()));
        declarations.push(PAGINATION_ACTIVE_COLOR.declare(on_active.value()));
        declarations
    }
}

/// Every string `Pagination` puts in front of a reader, in one struct that a
/// locale swaps whole.
///
/// Separate from [`PaginationDefaults`] because geometry and language are
/// changed by different people for different reasons - the `DateDefaults`
/// arrangement, which is locale-only and sits beside `DatePickerDefaults`
/// rather than inside it.
///
/// The `<nav>`'s own name is **not** here. `aria_label` is a required prop, so
/// the caller supplies it and localises it themselves; a default sitting here
/// would be set by a translator, read by nothing, and indistinguishable from
/// their own wiring being wrong (Bob3, reviewing C4).
///
/// **`format!("{page_label} {n}")` assumes the number goes last**, which is
/// wrong in plenty of languages. `Pagination`'s `label` prop is the escape
/// hatch for a language this shape cannot reach; see [`todos`] item 28 for the
/// i18n work this is waiting on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationLabels {
    /// Prefix for a page the reader is not on: `Go to page 4`.
    pub page_label: &'static str,
    /// Prefix for the current page: `Page 4`. `aria-current` already says
    /// "current", so repeating "go to" on the page you are on is a lie.
    pub current_page_label: &'static str,
    pub previous_label: &'static str,
    pub next_label: &'static str,
    pub first_label: &'static str,
    pub last_label: &'static str,
}

impl PaginationLabels {
    pub const ENGLISH: Self = Self {
        page_label: "Go to page",
        current_page_label: "Page",
        previous_label: "Go to previous page",
        next_label: "Go to next page",
        first_label: "Go to first page",
        last_label: "Go to last page",
    };
}
