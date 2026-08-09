use crate::common::ConstStr;

use super::{declaration::Declaration, selector_block::SelectorBlock, sx::Sx};

pub(super) const DEFAULT_SX_CSS_CAPACITY: usize = 4096;
pub(super) const ROOT_SELECTOR_BLOCK_PARENT: usize = usize::MAX;

pub(super) const fn to_css(sx: &Sx, class_name: &'static str) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
    let selector_blocks = sx.selector_blocks();
    let declarations = sx.declarations();
    let mut css = ConstStr::new();

    css = emit_rule(
        css,
        class_name,
        declarations,
        selector_blocks,
        ROOT_SELECTOR_BLOCK_PARENT,
        0,
        declarations.len(),
    );

    css
}

const fn emit_rule(
    mut css: ConstStr<DEFAULT_SX_CSS_CAPACITY>,
    selector: &str,
    declarations: &[Declaration],
    selector_blocks: &[SelectorBlock],
    parent_block_index: usize,
    start: usize,
    end: usize,
) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
    css = css.push_str(selector);
    css = css.push_char('{');

    let mut declaration_index = start;
    while declaration_index < end {
        if declaration_belongs_to_direct_child(
            declaration_index,
            selector_blocks,
            parent_block_index,
        ) {
            declaration_index += 1;
            continue;
        }

        let declaration = declarations[declaration_index];
        css = css.push_str(declaration.property);
        css = css.push_char(':');
        css = css.push_str(declaration.value);
        css = css.push_char(';');
        declaration_index += 1;
    }

    css = css.push_char('}');

    let mut block_index = 0;
    while block_index < selector_blocks.len() {
        let block = selector_blocks[block_index];
        if block.parent == parent_block_index {
            let mut nested_selector: ConstStr<DEFAULT_SX_CSS_CAPACITY> = ConstStr::new();
            nested_selector = nested_selector.push_str(selector);
            nested_selector = nested_selector.push_str(block.selector);
            css = emit_rule(
                css,
                nested_selector.as_str(),
                declarations,
                selector_blocks,
                block_index,
                block.start,
                block.end,
            );
        }
        block_index += 1;
    }

    css
}

const fn declaration_belongs_to_direct_child(
    declaration_index: usize,
    selector_blocks: &[SelectorBlock],
    parent_block_index: usize,
) -> bool {
    let mut block_index = 0;
    while block_index < selector_blocks.len() {
        let block = selector_blocks[block_index];
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
