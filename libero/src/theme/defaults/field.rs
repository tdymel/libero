use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const FIELD_LABEL_FONT_SIZE: SizeCss = SizeCss::new("--lsx-field-label-font-size-");
pub const FIELD_CAPTION_FONT_SIZE: SizeCss = SizeCss::new("--lsx-field-caption-font-size-");
pub const FIELD_FONT_SIZE: SizeCss = SizeCss::new("--lsx-field-font-size-");
pub const FIELD_HEIGHT: SizeCss = SizeCss::new("--lsx-field-height-");
pub const FIELD_PADDING_Y: SizeCss = SizeCss::new("--lsx-field-padding-y-");
pub const FIELD_PADDING_X: SizeCss = SizeCss::new("--lsx-field-padding-x-");
pub const FIELD_GAP: CssVar = CssVar::new("--lsx-field-gap");
pub const FIELD_FRAME_GAP: CssVar = CssVar::new("--lsx-field-frame-gap");

/// One size step of a field: the slots stacked around the control, and the
/// frame the control sits in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldSizeLevel {
    /// The caption above the control.
    pub label_font_size: &'static str,
    /// Description, helper and status - all three share one scale.
    pub caption_font_size: &'static str,
    /// The control's own text.
    pub font_size: &'static str,
    /// Floor for a single-line field, so every field in a form lines up.
    pub height: &'static str,
    /// Drives the frame's real height once the control wraps - a `Textarea`,
    /// or a field whose leading slot grows.
    pub padding_y: &'static str,
    pub padding_x: &'static str,
}

/// Shared by every field. A component keeps a `*Defaults` of its own only for
/// what genuinely differs - which, for `TextField`, is the default `size` and
/// `radius` and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldDefaults {
    /// Vertical gap between the slots.
    pub gap: &'static str,
    /// Horizontal gap between the frame's leading, control and trailing.
    pub frame_gap: &'static str,
    pub sizes: Sizes<FieldSizeLevel>,
}

impl FieldDefaults {
    /// Per-slot typography for one size step. The slots are addressed from the
    /// wrapper rather than each carrying its own class - four captions with
    /// four `use_box` chains would cost four stylesheet registrations per
    /// field, and none of them takes styling props of its own.
    fn size_sx(size: Size) -> Sx {
        sx().selector(
            "& > label",
            sx().font_size(FIELD_LABEL_FONT_SIZE.value(size)),
        )
        .selector(
            "& > [data-slot]",
            sx().font_size(FIELD_CAPTION_FONT_SIZE.value(size)),
        )
    }

    pub fn theme_vars() -> Sx {
        sx().gap(FIELD_GAP.value()).per_size(Self::size_sx)
    }

    fn frame_size_sx(size: Size) -> Sx {
        sx().font_size(FIELD_FONT_SIZE.value(size))
            .min_height(FIELD_HEIGHT.value(size))
            .padding_top(FIELD_PADDING_Y.value(size))
            .padding_bottom(FIELD_PADDING_Y.value(size))
            .padding_left(FIELD_PADDING_X.value(size))
            .padding_right(FIELD_PADDING_X.value(size))
    }

    fn frame_radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.value(radius))
    }

    pub fn frame_theme_vars() -> Sx {
        sx().gap(FIELD_FRAME_GAP.value())
            .per_size(Self::frame_size_sx)
            .per_radius(Self::frame_radius_sx)
    }
}

impl ToCssDeclarations for FieldDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![
            FIELD_GAP.declare(self.gap),
            FIELD_FRAME_GAP.declare(self.frame_gap),
        ];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(FIELD_LABEL_FONT_SIZE.declare(size, level.label_font_size));
            declarations.push(FIELD_CAPTION_FONT_SIZE.declare(size, level.caption_font_size));
            declarations.push(FIELD_FONT_SIZE.declare(size, level.font_size));
            declarations.push(FIELD_HEIGHT.declare(size, level.height));
            declarations.push(FIELD_PADDING_Y.declare(size, level.padding_y));
            declarations.push(FIELD_PADDING_X.declare(size, level.padding_x));
        }
        declarations
    }
}
