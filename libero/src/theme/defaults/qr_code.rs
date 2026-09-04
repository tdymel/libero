use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::CssVar;

pub const QR_CODE_BACKGROUND: CssVar = CssVar::new("--lsx-qrcode-background");
pub const QR_CODE_FOREGROUND: CssVar = CssVar::new("--lsx-qrcode-foreground");

str_enum! {
    /// QR error-correction level - higher levels tolerate more damage/occlusion
    /// at the cost of a denser code for the same data.
    pub enum QrRobustness {
        Low = "low",
        #[default]
        Medium = "medium",
        Quartile = "quartile",
        High = "high",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QrCodeDefaults {
    pub background: &'static str,
    pub foreground: &'static str,
    pub robustness: QrRobustness,
}

impl QrCodeDefaults {
    pub const DEFAULT: Self = Self {
        background: "#FFFFFF",
        foreground: "#000000",
        robustness: QrRobustness::Medium,
    };
}

impl ToCssDeclarations for QrCodeDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            QR_CODE_BACKGROUND.declare(self.background),
            QR_CODE_FOREGROUND.declare(self.foreground),
        ]
    }
}
