use crate::{
    sx::{
        Declaration, DeclarationProperty, Property, ROOT_BLOCK_PARENT, Sx, ThemeAwareValue,
        sx_block::SxBlock, sx_modifier::SxModifier,
    },
    theme::Size,
};

use super::{
    CssBlock, CssDeclaration, CssMediaQuery, CssScope, Stylesheet, StylesheetBuilder,
    css_var::SizeCssVar,
};

#[derive(Clone, Debug, PartialEq)]
struct FlattenedScope {
    parent_block_index: usize,
    selector: String,
    breakpoint: Option<Size>,
    start: usize,
    end: usize,
}

impl From<&Sx> for Stylesheet {
    fn from(sx: &Sx) -> Self {
        let class_name = sx.class_name();
        sx.to_flattened_scopes(format!(".{class_name}"))
            .iter()
            .map(|flattened_scope| to_css_block(flattened_scope, sx.declarations(), sx.blocks()))
            .fold(StylesheetBuilder::new(), |stylesheet, block| {
                stylesheet.with_block(block)
            })
            .into()
    }
}

impl Sx {
    fn to_flattened_scopes(&self, class_name: String) -> Vec<FlattenedScope> {
        let mut flattened_scopes = Vec::new();
        flatten_node(
            &mut flattened_scopes,
            self.declarations(),
            self.blocks(),
            ROOT_BLOCK_PARENT,
            class_name,
            None,
            0,
            self.declarations().len(),
        );
        flattened_scopes
    }
}

fn flatten_node(
    flattened_scopes: &mut Vec<FlattenedScope>,
    declarations: &[Declaration],
    blocks: &[SxBlock],
    parent_block_index: usize,
    selector: String,
    breakpoint: Option<Size>,
    start: usize,
    end: usize,
) {
    if has_owned_declarations(declarations, blocks, parent_block_index, start, end) {
        flattened_scopes.push(FlattenedScope {
            parent_block_index,
            selector: selector.clone(),
            breakpoint,
            start,
            end,
        });
    }

    let mut block_index = 0;
    while block_index < blocks.len() {
        let block = blocks[block_index];
        if block.parent == parent_block_index {
            let mut next_selector = selector.clone();
            let mut next_breakpoint = breakpoint;

            match block.modifier {
                SxModifier::Selector(suffix) => next_selector.push_str(suffix),
                SxModifier::Condition(condition) => {
                    next_selector.push_str("[data-state~=\"");
                    next_selector.push_str(condition);
                    next_selector.push_str("\"]");
                }
                SxModifier::Breakpoint(value) => {
                    next_breakpoint = Some(merge_breakpoint(breakpoint, value));
                }
            }

            flatten_node(
                flattened_scopes,
                declarations,
                blocks,
                block_index,
                next_selector,
                next_breakpoint,
                block.start,
                block.end,
            );
        }
        block_index += 1;
    }
}

fn to_css_block(
    flattened_scope: &FlattenedScope,
    declarations: &[Declaration],
    blocks: &[SxBlock],
) -> CssBlock {
    let scope = (flattened_scope.start..flattened_scope.end)
        .filter(|&declaration_index| {
            !declaration_belongs_to_direct_child(
                declaration_index,
                blocks,
                flattened_scope.parent_block_index,
            )
        })
        .fold(
            CssScope::from_string(flattened_scope.selector.clone()),
            |scope, declaration_index| {
                scope.with(to_css_declaration(declarations[declaration_index]))
            },
        );

    match flattened_scope.breakpoint {
        Some(breakpoint) => CssBlock::MediaQuery(
            CssMediaQuery::from_string(format!("(min-width: {})", breakpoint.breakpoint_value()))
                .with(scope),
        ),
        None => CssBlock::Scope(scope),
    }
}

fn to_css_declaration(declaration: Declaration) -> CssDeclaration {
    let property = match declaration.property {
        DeclarationProperty::Known(property) => property.as_str().to_string(),
        DeclarationProperty::Raw(property) => property.to_string(),
    };

    let value = match declaration.value {
        ThemeAwareValue::Raw(value) => value.to_string(),
        ThemeAwareValue::Color(color_value) => color_value.css_value(),
        ThemeAwareValue::Size(size) => match declaration.property {
            DeclarationProperty::Known(Property::PaddingTop) => SizeCssVar::SPACING.value(size),
            _ => size.as_str().to_string(),
        },
    };

    CssDeclaration::new(property, value)
}

const fn has_owned_declarations(
    declarations: &[Declaration],
    blocks: &[SxBlock],
    parent_block_index: usize,
    start: usize,
    end: usize,
) -> bool {
    let mut declaration_index = start;
    while declaration_index < end {
        if !declaration_belongs_to_direct_child(declaration_index, blocks, parent_block_index) {
            let _ = declarations;
            return true;
        }
        declaration_index += 1;
    }
    false
}

const fn declaration_belongs_to_direct_child(
    declaration_index: usize,
    blocks: &[SxBlock],
    parent_block_index: usize,
) -> bool {
    let mut block_index = 0;
    while block_index < blocks.len() {
        let block = blocks[block_index];
        if block.parent == parent_block_index
            && declaration_index >= block.start
            && declaration_index < block.end
        {
            return true;
        }
        block_index += 1;
    }
    false
}

const fn merge_breakpoint(current: Option<Size>, next: Size) -> Size {
    match current {
        Some(current) => {
            if (next as u8) > (current as u8) {
                next
            } else {
                current
            }
        }
        None => next,
    }
}
