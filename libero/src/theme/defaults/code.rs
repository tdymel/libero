use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const CODE_FONT_FAMILY: CssVar = CssVar::new("--lsx-code-font-family");
pub const CODE_BACKGROUND: CssVar = CssVar::new("--lsx-code-background");
pub const CODE_BORDER: CssVar = CssVar::new("--lsx-code-border");
pub const CODE_MUTED_TEXT: CssVar = CssVar::new("--lsx-code-muted-text");
pub const CODE_LINE_NUMBER: CssVar = CssVar::new("--lsx-code-line-number");
pub const CODE_TOK_KEYWORD: CssVar = CssVar::new("--lsx-code-tok-keyword");
pub const CODE_TOK_STRING: CssVar = CssVar::new("--lsx-code-tok-string");
pub const CODE_TOK_COMMENT: CssVar = CssVar::new("--lsx-code-tok-comment");
pub const CODE_TOK_NUMBER: CssVar = CssVar::new("--lsx-code-tok-number");
pub const CODE_TOK_CONSTANT: CssVar = CssVar::new("--lsx-code-tok-constant");
pub const CODE_TOK_FUNCTION: CssVar = CssVar::new("--lsx-code-tok-function");
pub const CODE_TOK_TYPE: CssVar = CssVar::new("--lsx-code-tok-type");
pub const CODE_TOK_TAG: CssVar = CssVar::new("--lsx-code-tok-tag");
pub const CODE_TOK_ATTRIBUTE: CssVar = CssVar::new("--lsx-code-tok-attribute");
pub const CODE_TOK_HEADING: CssVar = CssVar::new("--lsx-code-tok-heading");

// Highlighted-line and diff row colors intentionally aren't here - they're
// derived directly from the theme's primary/success/error palette (see
// `code.rs`'s `CODE_LINE_ROW_HIGHLIGHTED_SX`/`_DIFF_ADD_SX`/`_DIFF_REMOVE_SX`)
// rather than being independent fields that could drift out of sync with it.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeDefaults {
    pub font_family: &'static str,
    pub background: &'static str,
    pub border: &'static str,
    pub muted_text: &'static str,
    pub line_number: &'static str,
    pub tok_keyword: &'static str,
    pub tok_string: &'static str,
    pub tok_comment: &'static str,
    pub tok_number: &'static str,
    pub tok_constant: &'static str,
    pub tok_function: &'static str,
    pub tok_type: &'static str,
    pub tok_tag: &'static str,
    pub tok_attribute: &'static str,
    pub tok_heading: &'static str,
}

impl CodeDefaults {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        font_family: &'static str,
        background: &'static str,
        border: &'static str,
        muted_text: &'static str,
        line_number: &'static str,
        tok_keyword: &'static str,
        tok_string: &'static str,
        tok_comment: &'static str,
        tok_number: &'static str,
        tok_constant: &'static str,
        tok_function: &'static str,
        tok_type: &'static str,
        tok_tag: &'static str,
        tok_attribute: &'static str,
        tok_heading: &'static str,
    ) -> Self {
        Self {
            font_family,
            background,
            border,
            muted_text,
            line_number,
            tok_keyword,
            tok_string,
            tok_comment,
            tok_number,
            tok_constant,
            tok_function,
            tok_type,
            tok_tag,
            tok_attribute,
            tok_heading,
        }
    }
}

impl ToCssDeclarations for CodeDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            CODE_FONT_FAMILY.declare(self.font_family),
            CODE_BACKGROUND.declare(self.background),
            CODE_BORDER.declare(self.border),
            CODE_MUTED_TEXT.declare(self.muted_text),
            CODE_LINE_NUMBER.declare(self.line_number),
            CODE_TOK_KEYWORD.declare(self.tok_keyword),
            CODE_TOK_STRING.declare(self.tok_string),
            CODE_TOK_COMMENT.declare(self.tok_comment),
            CODE_TOK_NUMBER.declare(self.tok_number),
            CODE_TOK_CONSTANT.declare(self.tok_constant),
            CODE_TOK_FUNCTION.declare(self.tok_function),
            CODE_TOK_TYPE.declare(self.tok_type),
            CODE_TOK_TAG.declare(self.tok_tag),
            CODE_TOK_ATTRIBUTE.declare(self.tok_attribute),
            CODE_TOK_HEADING.declare(self.tok_heading),
        ]
    }
}
