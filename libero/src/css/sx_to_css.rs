use crate::{
    common::{ConstStr, ConstVec},
    theme::Size,
};

use super::{CssBlock, CssMediaQuery, CssScope, Stylesheet};

use crate::sx::sx_block::SxBlock;
use crate::sx::sx_modifier::SxModifier;
use crate::sx::{Declaration, ROOT_BLOCK_PARENT, Sx};

const DEFAULT_SX_FLATTENED_SCOPE_CAPACITY: usize = 64;
const DEFAULT_SX_BLOCK_OUTPUT_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
struct FlattenedScope {
    parent_block_index: usize,
    selector: ConstStr,
    breakpoint: Option<Size>,
    start: usize,
    end: usize,
}

impl Sx {
    pub const fn hash(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325u64;

        let declarations = self.declarations();
        let mut declaration_index = 0;
        while declaration_index < declarations.len() {
            hash = hash_declaration(hash, declarations[declaration_index]);
            declaration_index += 1;
        }

        let blocks = self.blocks();
        let mut block_index = 0;
        while block_index < blocks.len() {
            hash = hash_sx_block(hash, blocks[block_index]);
            block_index += 1;
        }

        hash
    }

    pub const fn class_name(&self) -> ConstStr {
        let mut class_name = ConstStr::from_str("lsx-");
        class_name = push_hex_u64(class_name, self.hash());
        class_name
    }

    pub const fn to_css(&self) -> Stylesheet {
        let class_name = self.class_name();
        let css_blocks = self.to_css_blocks(class_name_to_selector(class_name));
        let css_blocks_ref = css_blocks.as_ref();
        let mut stylesheet = Stylesheet::new().with_class_name(class_name);

        let mut index = 0;
        while index < css_blocks_ref.len() {
            stylesheet = stylesheet.append_block(css_blocks_ref[index]);
            index += 1;
        }

        stylesheet
    }

    const fn to_css_blocks(
        &self,
        class_name: ConstStr,
    ) -> ConstVec<CssBlock, DEFAULT_SX_BLOCK_OUTPUT_CAPACITY> {
        let flattened_scopes = self.to_flattened_scopes(class_name);
        let flattened_scopes_ref = flattened_scopes.as_ref();
        let mut css_blocks = ConstVec::new_with_max_size();

        let mut index = 0;
        while index < flattened_scopes_ref.len() {
            css_blocks.push(to_css_block(
                flattened_scopes_ref[index],
                self.declarations(),
                self.blocks(),
            ));
            index += 1;
        }

        css_blocks
    }

    const fn to_flattened_scopes(
        &self,
        class_name: ConstStr,
    ) -> ConstVec<FlattenedScope, DEFAULT_SX_FLATTENED_SCOPE_CAPACITY> {
        let mut flattened_scopes = ConstVec::new_with_max_size();
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

const fn flatten_node(
    flattened_scopes: &mut ConstVec<FlattenedScope, DEFAULT_SX_FLATTENED_SCOPE_CAPACITY>,
    declarations: &[Declaration],
    blocks: &[SxBlock],
    parent_block_index: usize,
    selector: ConstStr,
    breakpoint: Option<Size>,
    start: usize,
    end: usize,
) {
    if has_owned_declarations(declarations, blocks, parent_block_index, start, end) {
        flattened_scopes.push(FlattenedScope {
            parent_block_index,
            selector,
            breakpoint,
            start,
            end,
        });
    }

    let mut block_index = 0;
    while block_index < blocks.len() {
        let block = blocks[block_index];
        if block.parent == parent_block_index {
            let mut next_selector = selector;
            let mut next_breakpoint = breakpoint;

            match block.modifier {
                SxModifier::Selector(suffix) => {
                    next_selector = next_selector.push_str(suffix);
                }
                SxModifier::Condition(condition) => {
                    next_selector = next_selector.push_str("[data-state~=\"");
                    next_selector = next_selector.push_str(condition);
                    next_selector = next_selector.push_str("\"]");
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

const fn to_css_block(
    flattened_scope: FlattenedScope,
    declarations: &[Declaration],
    blocks: &[SxBlock],
) -> CssBlock {
    let mut scope = CssScope::from_const_str(flattened_scope.selector);

    let mut declaration_index = flattened_scope.start;
    while declaration_index < flattened_scope.end {
        if declaration_belongs_to_direct_child(
            declaration_index,
            blocks,
            flattened_scope.parent_block_index,
        ) {
            declaration_index += 1;
            continue;
        }

        scope = scope.with(declarations[declaration_index]);
        declaration_index += 1;
    }

    match flattened_scope.breakpoint {
        Some(breakpoint) => CssBlock::MediaQuery(
            CssMediaQuery::from_const_str(breakpoint_to_media_condition(breakpoint)).with(scope),
        ),
        None => CssBlock::Scope(scope),
    }
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

const fn breakpoint_to_media_condition(breakpoint: Size) -> crate::common::ConstStr {
    crate::common::ConstStr::from_str("(min-width: ")
        .push_str(breakpoint.breakpoint_value())
        .push_char(')')
}

const fn hash_u8(mut hash: u64, value: u8) -> u64 {
    hash ^= value as u64;
    hash.wrapping_mul(0x00000100000001B3)
}

const fn hash_str(mut hash: u64, value: &str) -> u64 {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        hash = hash_u8(hash, bytes[index]);
        index += 1;
    }
    hash
}

const fn hash_declaration(mut hash: u64, declaration: Declaration) -> u64 {
    hash = match declaration.property {
        crate::sx::DeclarationProperty::Known(property) => {
            hash_str(hash_u8(hash, 0), property.as_str())
        }
        crate::sx::DeclarationProperty::Raw(property) => hash_str(hash_u8(hash, 1), property),
        crate::sx::DeclarationProperty::RawConstStr(property) => {
            hash_str(hash_u8(hash, 2), property.as_str())
        }
    };

    match declaration.value {
        crate::sx::ThemeAwareValue::Raw(value) => hash_str(hash_u8(hash, 3), value),
        crate::sx::ThemeAwareValue::RawConstStr(value) => {
            hash_str(hash_u8(hash, 4), value.as_str())
        }
        crate::sx::ThemeAwareValue::Color(color_value) => match color_value {
            crate::theme::ColorValue::Shade(color, shade) => hash_str(
                hash_str(hash_u8(hash_u8(hash, 5), color as u8), shade.as_str()),
                "",
            ),
            crate::theme::ColorValue::Contrast(color, shade) => hash_str(
                hash_str(hash_u8(hash_u8(hash, 6), color as u8), shade.as_str()),
                "",
            ),
        },
        crate::sx::ThemeAwareValue::Size(size) => hash_str(hash_u8(hash, 7), size.as_str()),
    }
}

const fn hash_sx_block(mut hash: u64, block: SxBlock) -> u64 {
    hash = match block.modifier {
        SxModifier::Selector(selector) => hash_str(hash_u8(hash, 8), selector),
        SxModifier::Condition(condition) => hash_str(hash_u8(hash, 9), condition),
        SxModifier::Breakpoint(size) => hash_str(hash_u8(hash, 10), size.breakpoint_value()),
    };
    hash = hash_u8(hash, (block.start & 0xFF) as u8);
    hash = hash_u8(hash, (block.end & 0xFF) as u8);
    hash_u8(hash, (block.parent & 0xFF) as u8)
}

const fn class_name_to_selector(class_name: ConstStr) -> ConstStr {
    ConstStr::from_str(".").append(class_name)
}

const fn push_hex_u64(mut css: ConstStr, value: u64) -> ConstStr {
    let mut shift = 60;
    loop {
        let digit = ((value >> shift) & 0x0F) as u8;
        css = css.push_char(match digit {
            0..=9 => (b'0' + digit) as char,
            _ => (b'a' + (digit - 10)) as char,
        });

        if shift == 0 {
            break;
        }
        shift -= 4;
    }
    css
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
