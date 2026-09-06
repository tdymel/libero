use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{ChoiceVariant, CssVar, Size, SizeCss, Sizes};

pub const SWITCH_TRACK_WIDTH: SizeCss = SizeCss::new("--lsx-switch-track-width-");
pub const SWITCH_TRACK_HEIGHT: SizeCss = SizeCss::new("--lsx-switch-track-height-");
pub const SWITCH_THUMB_SIZE: SizeCss = SizeCss::new("--lsx-switch-thumb-size-");

// The picked level, resolved on the root so the track/thumb children - which
// carry no `data-state` of their own - can inherit it.
pub const SWITCH_TRACK_W: CssVar = CssVar::new("--lsx-switch-track-w");
pub const SWITCH_TRACK_H: CssVar = CssVar::new("--lsx-switch-track-h");
pub const SWITCH_THUMB: CssVar = CssVar::new("--lsx-switch-thumb");
pub const SWITCH_RADIUS: CssVar = CssVar::new("--lsx-switch-radius");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchSizeLevel {
    pub track_width: &'static str,
    pub track_height: &'static str,
    pub thumb_size: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchDefaults {
    /// The wrapper a switch takes when a call site names none.
    pub variant: ChoiceVariant,
    pub size: Size,
    /// Track corner radius; the thumb is always a circle.
    pub radius: Size,
    pub sizes: Sizes<SwitchSizeLevel>,
}

impl SwitchDefaults {
    pub const DEFAULT: Self = Self {
        variant: ChoiceVariant::Plain,
        size: Size::Md,
        radius: Size::Xl,
        sizes: Sizes::new(
            SwitchSizeLevel {
                track_width: "30px",
                track_height: "16px",
                thumb_size: "12px",
            },
            SwitchSizeLevel {
                track_width: "34px",
                track_height: "18px",
                thumb_size: "14px",
            },
            SwitchSizeLevel {
                track_width: "42px",
                track_height: "22px",
                thumb_size: "18px",
            },
            SwitchSizeLevel {
                track_width: "50px",
                track_height: "26px",
                thumb_size: "22px",
            },
            SwitchSizeLevel {
                track_width: "58px",
                track_height: "30px",
                thumb_size: "26px",
            },
            SwitchSizeLevel {
                track_width: "66px",
                track_height: "34px",
                thumb_size: "30px",
            },
        ),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(SWITCH_TRACK_W, SWITCH_TRACK_WIDTH.value(size))
            .var(SWITCH_TRACK_H, SWITCH_TRACK_HEIGHT.value(size))
            .var(SWITCH_THUMB, SWITCH_THUMB_SIZE.value(size))
    }

    pub fn radius_sx(radius: Size) -> Sx {
        sx().var(SWITCH_RADIUS, SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for SwitchDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(SWITCH_TRACK_WIDTH.declare(size, level.track_width));
            declarations.push(SWITCH_TRACK_HEIGHT.declare(size, level.track_height));
            declarations.push(SWITCH_THUMB_SIZE.declare(size, level.thumb_size));
        }
        declarations
    }
}
