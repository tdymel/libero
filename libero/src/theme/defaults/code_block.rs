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
    /// Draws the bar naming the language above the code.
    pub header: bool,
    /// Offers the copy button.
    pub copyable: bool,
    /// Draws the line-number gutter.
    pub line_numbers: bool,
}

impl CodeBlockDefaults {
    pub const DEFAULT: Self = Self {
        background: "#f6f8fa",
        border: "#d0d7de",
        muted_text: "#57606a",
        // 4.27:1 on the background above; the lighter grey it replaced was
        // 2.85:1, and a line number is the only way to cite a line (todo 241).
        line_number: "#6e7781",
        copy_hover_background: "rgba(31, 35, 40, 0.08)",
        copy_hover_text: "#1f2328",
        header: true,
        copyable: true,
        line_numbers: true,
    };

    /// The same block on an inked page. GitHub's own dark theme, because the
    /// token colours in [`CodeDefaults::DARK`] are measured against it.
    pub const DARK: Self = Self {
        background: "#161b22",
        border: "#30363d",
        muted_text: "#8b949e",
        line_number: "#8b949e",
        copy_hover_background: "rgba(240, 246, 252, 0.10)",
        copy_hover_text: "#c9d1d9",
        ..Self::DEFAULT
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::CodeDefaults;
    use crate::tokens::HexColor;

    fn ratio(color: &str, background: &str) -> f32 {
        HexColor::parse(color)
            .expect("a hex")
            .contrast_ratio(HexColor::parse(background).expect("a hex"))
    }

    /// Todo 241. Every printed example on the docs site is drawn in these,
    /// and the block's background is tinted, so GitHub's own numbers - which
    /// are measured on white - do not carry over.
    #[test]
    fn the_code_theme_reads_on_its_own_background() {
        the_code_theme_reads_on(CodeBlockDefaults::DEFAULT, CodeDefaults::DEFAULT);
    }

    /// The same measurement for the dark theme's block. GitHub's dark tokens
    /// are published against `#0d1117`, and ours sits on `#161b22`, so the
    /// numbers have to be taken again rather than inherited.
    #[test]
    fn the_dark_code_theme_reads_on_its_own_background() {
        the_code_theme_reads_on(CodeBlockDefaults::DARK, CodeDefaults::DARK);
    }

    fn the_code_theme_reads_on(block: CodeBlockDefaults, code: CodeDefaults) {
        let background = block.background;
        for (name, token) in [
            ("keyword", code.tok_keyword),
            ("string", code.tok_string),
            ("number", code.tok_number),
            ("constant", code.tok_constant),
            ("function", code.tok_function),
            ("type", code.tok_type),
            ("tag", code.tok_tag),
            ("attribute", code.tok_attribute),
            ("heading", code.tok_heading),
        ] {
            let ratio = ratio(token, background);
            assert!(
                ratio >= 4.5,
                "{name} {token} on {background} at {ratio:.2}:1"
            );
        }

        // A line number is held to 1.4.11 rather than 1.4.3 (todo 241): it is
        // a way to cite a line, not text the reader is meant to read through.
        let ratio = ratio(block.line_number, background);
        assert!(ratio >= 3.0, "line number at {ratio:.2}:1");
    }

    #[test]
    fn the_light_code_theme_reads_on_its_own_background_the_way_todo_241_measured_it() {
        let background = CodeBlockDefaults::DEFAULT.background;

        assert!(
            ratio(CodeDefaults::DEFAULT.tok_comment, background) >= 4.5,
            "comments measured {:.2}:1",
            ratio(CodeDefaults::DEFAULT.tok_comment, background)
        );
        // A line number is incidental - it labels, it does not carry the
        // content - so it is held to 1.4.11's 3:1 rather than 1.4.3's 4.5:1.
        // The ramp the theme is built from has nothing between 4.27 and the
        // muted text the comments now use.
        assert!(
            ratio(CodeBlockDefaults::DEFAULT.line_number, background) >= 3.0,
            "line numbers measured {:.2}:1",
            ratio(CodeBlockDefaults::DEFAULT.line_number, background)
        );
    }
}
