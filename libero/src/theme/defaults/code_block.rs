use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const CODE_BLOCK_BACKGROUND: CssVar = CssVar::new("--lsx-code-block-background");
pub const CODE_BLOCK_BORDER: CssVar = CssVar::new("--lsx-code-block-border");
pub const CODE_BLOCK_MUTED_TEXT: CssVar = CssVar::new("--lsx-code-block-muted-text");
pub const CODE_BLOCK_LINE_NUMBER: CssVar = CssVar::new("--lsx-code-block-line-number");
pub const CODE_BLOCK_COPY_HOVER_BACKGROUND: CssVar =
    CssVar::new("--lsx-code-block-copy-hover-background");
pub const CODE_BLOCK_COPY_HOVER_TEXT: CssVar = CssVar::new("--lsx-code-block-copy-hover-text");

// Highlight and diff row colors aren't here on purpose: they derive from the
// theme's primary/success/error rather than drifting as separate fields.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeBlockDefaults {
    pub background: &'static str,
    pub border: &'static str,
    pub muted_text: &'static str,
    pub line_number: &'static str,
    pub copy_hover_background: &'static str,
    pub copy_hover_text: &'static str,
}

impl CodeBlockDefaults {
    pub const DEFAULT: Self = Self {
        background: "#f6f8fa",
        border: "#d0d7de",
        muted_text: "#57606a",
        line_number: "#8c959f",
        copy_hover_background: "rgba(31, 35, 40, 0.08)",
        copy_hover_text: "#1f2328",
    };
}

impl ToCssDeclarations for CodeBlockDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            CODE_BLOCK_BACKGROUND.declare(self.background),
            CODE_BLOCK_BORDER.declare(self.border),
            CODE_BLOCK_MUTED_TEXT.declare(self.muted_text),
            CODE_BLOCK_LINE_NUMBER.declare(self.line_number),
            CODE_BLOCK_COPY_HOVER_BACKGROUND.declare(self.copy_hover_background),
            CODE_BLOCK_COPY_HOVER_TEXT.declare(self.copy_hover_text),
        ]
    }
}
