use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const CODE_FONT_FAMILY: CssVar = CssVar::new("--lsx-code-font-family");
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeDefaults {
    pub font_family: &'static str,
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

impl ToCssDeclarations for CodeDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            CODE_FONT_FAMILY.declare(self.font_family),
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
