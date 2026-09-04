use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const TOOLTIP_FONT_SIZE: SizeCss = SizeCss::new("--lsx-tooltip-font-size-");

pub const TOOLTIP_BACKGROUND: CssVar = CssVar::new("--lsx-tooltip-background");
pub const TOOLTIP_COLOR: CssVar = CssVar::new("--lsx-tooltip-color");
pub const TOOLTIP_DURATION: CssVar = CssVar::new("--lsx-tooltip-duration");

str_enum! {
    /// Which side of the trigger the bubble sits on, centred on that side.
    #[state_prefix = "placement"]
    pub enum TooltipPlacement {
        #[default]
        Top = "top",
        Right = "right",
        Bottom = "bottom",
        Left = "left",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooltipDefaults {
    pub placement: TooltipPlacement,
    /// Distance between trigger and bubble, bridged so the pointer can cross.
    pub gap: Size,
    pub size: Size,
    /// Milliseconds before the bubble appears / disappears.
    pub open_delay: u32,
    pub close_delay: u32,
    /// Fade duration, in milliseconds.
    pub duration: u32,
    pub font_size: Sizes<u16>,
    pub background: &'static str,
    pub color: &'static str,
}

impl TooltipDefaults {
    pub const DEFAULT: Self = Self {
        placement: TooltipPlacement::Top,
        gap: Size::Xs,
        size: Size::Sm,
        open_delay: 0,
        close_delay: 0,
        duration: 150,
        font_size: Sizes::new(10, 12, 13, 14, 16, 18),
        background: "#1f2328",
        color: "#ffffff",
    };

    fn size_sx(size: Size) -> Sx {
        sx().font_size(TOOLTIP_FONT_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().background(TOOLTIP_BACKGROUND.value())
            .color(TOOLTIP_COLOR.value())
            .border_radius(SizeCss::RADIUS.value(Size::Sm))
            .padding(format!(
                "{} {}",
                SizeCss::SPACING.value(Size::Xs),
                SizeCss::SPACING.value(Size::Sm)
            ))
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for TooltipDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.font_size.to_css_declarations(TOOLTIP_FONT_SIZE, "px");
        declarations.push(TOOLTIP_BACKGROUND.declare(self.background));
        declarations.push(TOOLTIP_COLOR.declare(self.color));
        declarations.push(TOOLTIP_DURATION.declare(format!("{}ms", self.duration)));
        declarations
    }
}
