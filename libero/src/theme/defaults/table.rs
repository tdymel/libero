use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, Sizes};

pub const TABLE_FONT_SIZE: SizeCss = SizeCss::new("--lsx-table-font-size-");
pub const TABLE_PADDING_X: SizeCss = SizeCss::new("--lsx-table-padding-x-");
pub const TABLE_PADDING_Y: SizeCss = SizeCss::new("--lsx-table-padding-y-");

// Resolved on the table for its size; the cells and the sort button read them.
pub const TABLE_PAD_X: CssVar = CssVar::new("--lsx-table-pad-x");
pub const TABLE_PAD_Y: CssVar = CssVar::new("--lsx-table-pad-y");

pub const TABLE_BORDER_COLOR: CssVar = CssVar::new("--lsx-table-border-color");
pub const TABLE_HOVER: CssVar = CssVar::new("--lsx-table-hover");
pub const TABLE_STRIPE: CssVar = CssVar::new("--lsx-table-stripe");
pub const TABLE_SELECTED: CssVar = CssVar::new("--lsx-table-selected");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableSizeLevel {
    pub font_size: &'static str,
    /// A cell's inline padding, the sort button's too.
    pub padding_x: &'static str,
    pub padding_y: &'static str,
}

/// Theme defaults for `Table`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableDefaults {
    pub size: Size,
    pub sizes: Sizes<TableSizeLevel>,
    pub border_color: ColorValue,
    pub hover_color: ColorValue,
    /// Every other body row's background, when `striped`.
    pub stripe_color: ColorValue,
    /// A selected row's background, hovered or not.
    pub selected_color: ColorValue,
}

impl TableDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(
            TableSizeLevel {
                font_size: "12px",
                padding_x: "8px",
                padding_y: "4px",
            },
            TableSizeLevel {
                font_size: "13px",
                padding_x: "10px",
                padding_y: "6px",
            },
            TableSizeLevel {
                font_size: "14px",
                padding_x: "12px",
                padding_y: "10px",
            },
            TableSizeLevel {
                font_size: "16px",
                padding_x: "16px",
                padding_y: "12px",
            },
            TableSizeLevel {
                font_size: "18px",
                padding_x: "20px",
                padding_y: "14px",
            },
            TableSizeLevel {
                font_size: "20px",
                padding_x: "24px",
                padding_y: "16px",
            },
        ),
        border_color: ColorValue::Shade(Color::Muted, ColorShade::S3),
        // One step past the stripe, so a hovered striped row still changes.
        hover_color: ColorValue::Shade(Color::Muted, ColorShade::S2),
        stripe_color: ColorValue::Shade(Color::Muted, ColorShade::S1),
        selected_color: ColorValue::Shade(Color::Primary, ColorShade::S1),
    };

    pub(crate) fn border() -> String {
        format!("1px solid {}", TABLE_BORDER_COLOR.value())
    }

    /// Also used by the sort button, which has to fill its header cell.
    pub(crate) fn padding() -> String {
        format!("{} {}", TABLE_PAD_Y.value(), TABLE_PAD_X.value())
    }

    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TABLE_FONT_SIZE.value(size))
            .var(TABLE_PAD_X, TABLE_PADDING_X.value(size))
            .var(TABLE_PAD_Y, TABLE_PADDING_Y.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
            .selector("& th, & td", sx().padding(Self::padding()))
            .selector("& thead th", sx().border_bottom(Self::border()))
            // On the cells: a row's own border draws only when borders collapse.
            .selector("& tbody tr > *", sx().border_bottom(Self::border()))
            .selector("& tbody tr:hover", sx().background(TABLE_HOVER.value()))
            // Not on the hovered row: the stripe's selector would outrank the hover.
            .when(
                "striped",
                sx().selector(
                    "& tbody tr:nth-child(even):not(:hover):not([aria-selected=\"true\"])",
                    sx().background(TABLE_STRIPE.value()),
                ),
            )
            // After the hover, which it outranks by order alone.
            .selector(
                "& tbody tr[aria-selected=\"true\"]",
                sx().background(TABLE_SELECTED.value()),
            )
    }
}

impl ToCssDeclarations for TableDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![
            TABLE_BORDER_COLOR.declare(self.border_color.value()),
            TABLE_HOVER.declare(self.hover_color.value()),
            TABLE_STRIPE.declare(self.stripe_color.value()),
            TABLE_SELECTED.declare(self.selected_color.value()),
        ];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(TABLE_FONT_SIZE.declare(size, level.font_size));
            declarations.push(TABLE_PADDING_X.declare(size, level.padding_x));
            declarations.push(TABLE_PADDING_Y.declare(size, level.padding_y));
        }
        declarations
    }
}
