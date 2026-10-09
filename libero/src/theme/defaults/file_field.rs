use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, FIELD_FONT_SIZE, FIELD_PADDING_X, Size, SizeCss, Sizes};

pub const FILE_FIELD_DROPZONE_HEIGHT_SIZE: SizeCss =
    SizeCss::new("--lsx-file-field-dropzone-height-");

// The picked level, resolved on the root so the surface, cards and prompt inherit it.
pub const FILE_FIELD_DROPZONE_HEIGHT: CssVar = CssVar::new("--lsx-file-field-dropzone-height");
pub const FILE_FIELD_PADDING: CssVar = CssVar::new("--lsx-file-field-padding");
pub const FILE_FIELD_RADIUS: CssVar = CssVar::new("--lsx-file-field-radius");

str_enum! {
    /// Which control a `FileField` draws; both use the same hidden input.
    #[state_prefix = "variant"]
    pub enum FileFieldVariant {
        /// One line in the field frame, like every other input.
        #[default]
        Input = "input",
        /// A tall dashed surface to drop onto or click.
        Dropzone = "dropzone",
    }
}

/// Theme defaults for `FileField`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileFieldDefaults {
    pub size: Size,
    pub radius: Size,
    /// Which control is drawn when the caller states no `variant`.
    pub variant: FileFieldVariant,
    /// Whether the clear button shows once a file is picked.
    pub clearable: bool,
    /// Minimum height of the `Dropzone` variant's surface, per size step.
    pub dropzone_heights: Sizes<&'static str>,
}

impl FileFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        variant: FileFieldVariant::Input,
        clearable: true,
        dropzone_heights: Sizes::new("72px", "88px", "104px", "124px", "148px", "176px"),
    };

    /// The surface's own height; padding and font come from the field's scale.
    pub fn size_sx(size: Size) -> Sx {
        sx().var(
            FILE_FIELD_DROPZONE_HEIGHT,
            FILE_FIELD_DROPZONE_HEIGHT_SIZE.value(size),
        )
        .var(FILE_FIELD_PADDING, FIELD_PADDING_X.value(size))
        .font_size(FIELD_FONT_SIZE.value(size))
    }

    pub fn radius_sx(radius: Size) -> Sx {
        sx().var(FILE_FIELD_RADIUS, SizeCss::RADIUS.overridable(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for FileFieldDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        Size::ALL
            .into_iter()
            .map(|size| {
                FILE_FIELD_DROPZONE_HEIGHT_SIZE.declare(size, self.dropzone_heights.get(size))
            })
            .collect()
    }
}
