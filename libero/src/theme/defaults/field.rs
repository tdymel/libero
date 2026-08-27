use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const FIELD_LABEL_FONT_SIZE: SizeCss = SizeCss::new("--lsx-field-label-font-size-");
pub const FIELD_CAPTION_FONT_SIZE: SizeCss = SizeCss::new("--lsx-field-caption-font-size-");
pub const FIELD_GAP: CssVar = CssVar::new("--lsx-field-gap");

/// Typography for the four text slots a field stacks around its control.
///
/// The frame numbers - font size, height, padding - still live in
/// `TextFieldDefaults` and `SelectDefaults` until R5 merges them in here, so no
/// var is defined in two places.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldSizeLevel {
    /// The caption above the control.
    pub label_font_size: &'static str,
    /// Description, helper and status - all three share one scale.
    pub caption_font_size: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldDefaults {
    /// Vertical gap between the slots.
    pub gap: &'static str,
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
}

impl ToCssDeclarations for FieldDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![FIELD_GAP.declare(self.gap)];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(FIELD_LABEL_FONT_SIZE.declare(size, level.label_font_size));
            declarations.push(FIELD_CAPTION_FONT_SIZE.declare(size, level.caption_font_size));
        }
        declarations
    }
}
