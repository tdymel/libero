use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const QR_CODE_BACKGROUND: CssVar = CssVar::new("--lsx-qrcode-background");
pub const QR_CODE_FOREGROUND: CssVar = CssVar::new("--lsx-qrcode-foreground");

/// QR error-correction level - higher levels tolerate more damage/occlusion
/// at the cost of a denser code for the same data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum QrRobustness {
    Low,
    #[default]
    Medium,
    Quartile,
    High,
}

impl From<&str> for QrRobustness {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "low" => Self::Low,
            "quartile" => Self::Quartile,
            "high" => Self::High,
            _ => Self::Medium,
        }
    }
}

impl From<String> for QrRobustness {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QrCodeDefaults {
    pub background: &'static str,
    pub foreground: &'static str,
    pub robustness: QrRobustness,
}

impl ToCssDeclarations for QrCodeDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            QR_CODE_BACKGROUND.declare(self.background),
            QR_CODE_FOREGROUND.declare(self.foreground),
        ]
    }
}
