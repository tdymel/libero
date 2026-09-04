use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size};

pub const PIN_FIELD_GAP: CssVar = CssVar::new("--lsx-pin-field-gap");

str_enum! {
    /// Which characters a `PinField` cell accepts. Anything else never
    /// reaches the value - a rejected key is dropped, not shown and removed.
    #[state_prefix = "kind"]
    pub enum PinKind {
        /// Digits only. Also picks the numeric keypad on a phone.
        #[default]
        Numeric = "numeric",
        /// ASCII letters and digits.
        Alphanumeric = "alphanumeric",
    }
}

impl PinKind {
    /// Whether a typed or pasted character belongs in the value.
    pub(crate) fn accepts(self, character: char) -> bool {
        match self {
            Self::Numeric => character.is_ascii_digit(),
            Self::Alphanumeric => character.is_ascii_alphanumeric(),
        }
    }
}

/// What `PinField` does not share with every other field. The cell's own
/// numbers are the frame's - a cell is a square of `FIELD_HEIGHT`, so a
/// `PinField` is exactly as tall as a `TextField` at the same size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinFieldDefaults {
    /// How many cells a `PinField` has when the caller states no `length`.
    pub length: usize,
    pub size: Size,
    pub radius: Size,
    /// Which characters a cell accepts.
    pub kind: PinKind,
    /// Horizontal gap between the cells.
    pub gap: &'static str,
}

impl PinFieldDefaults {
    pub const DEFAULT: Self = Self {
        length: 4,
        size: Size::Md,
        radius: Size::Sm,
        kind: PinKind::Numeric,
        gap: "8px",
    };

    pub fn theme_vars() -> Sx {
        sx().gap(PIN_FIELD_GAP.value())
    }
}

impl ToCssDeclarations for PinFieldDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![PIN_FIELD_GAP.declare(self.gap)]
    }
}
