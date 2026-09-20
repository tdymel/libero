use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size};

pub const PIN_FIELD_GAP: CssVar = CssVar::new("--lsx-pin-field-gap");

str_enum! {
    /// Which characters a `PinField` cell accepts; a rejected key is dropped.
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

/// Theme defaults for `PinField`, set on [`Theme`](crate::theme::Theme).
/// A cell is a square of `FIELD_HEIGHT`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinFieldDefaults {
    /// Cells when the caller states no `length`.
    pub length: usize,
    pub size: Size,
    pub radius: Size,
    pub kind: PinKind,
    /// Between the cells.
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
