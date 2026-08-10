use crate::common::ConstStr;
use crate::theme::Size;

use crate::sx::sx_block::SxBlock;
use crate::sx::sx_modifier::SxModifier;
use crate::sx::{Declaration, ROOT_BLOCK_PARENT, Sx};

pub(crate) const DEFAULT_SX_CSS_CAPACITY: usize = 4096;

impl Sx {
    pub const fn to_css(&self, class_name: &'static str) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
        let blocks = self.blocks();
        let declarations = self.declarations();
        let mut css = ConstStr::new();

        css = emit_node(
            css,
            declarations,
            blocks,
            ROOT_BLOCK_PARENT,
            class_name,
            None,
            0,
            declarations.len(),
        );

        css
    }
}

const fn emit_node(
    mut css: ConstStr<DEFAULT_SX_CSS_CAPACITY>,
    declarations: &[Declaration],
    blocks: &[SxBlock],
    parent_block_index: usize,
    selector: &str,
    breakpoint: Option<Size>,
    start: usize,
    end: usize,
) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
    css = emit_rule(
        css,
        declarations,
        blocks,
        parent_block_index,
        selector,
        breakpoint,
        start,
        end,
    );

    let mut block_index = 0;
    while block_index < blocks.len() {
        let block = blocks[block_index];
        if block.parent == parent_block_index {
            let mut next_selector: ConstStr<DEFAULT_SX_CSS_CAPACITY> = ConstStr::new();
            next_selector = next_selector.push_str(selector);
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

            css = emit_node(
                css,
                declarations,
                blocks,
                block_index,
                next_selector.as_str(),
                next_breakpoint,
                block.start,
                block.end,
            );
        }
        block_index += 1;
    }

    css
}

const fn emit_rule(
    mut css: ConstStr<DEFAULT_SX_CSS_CAPACITY>,
    declarations: &[Declaration],
    blocks: &[SxBlock],
    parent_block_index: usize,
    selector: &str,
    breakpoint: Option<Size>,
    start: usize,
    end: usize,
) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
    if !has_owned_declarations(declarations, blocks, parent_block_index, start, end) {
        return css;
    }

    if let Some(breakpoint) = breakpoint {
        css = css.push_str("@media (min-width: ");
        css = css.push_str(breakpoint.breakpoint_value());
        css = css.push_char(')');
        css = css.push_char('{');
    }

    css = css.push_str(selector);
    css = css.push_char('{');

    let mut declaration_index = start;
    while declaration_index < end {
        if declaration_belongs_to_direct_child(declaration_index, blocks, parent_block_index) {
            declaration_index += 1;
            continue;
        }

        let declaration = declarations[declaration_index];
        css = declaration.push(css);
        declaration_index += 1;
    }

    css = css.push_char('}');

    if breakpoint.is_some() {
        css = css.push_char('}');
    }

    css
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
