use crate::theme::Size;

use super::{CssBlock, CssMediaQuery, CssScope, Stylesheet};

use crate::sx::sx_block::SxBlock;
use crate::sx::sx_modifier::SxModifier;
use crate::sx::{Declaration, ROOT_BLOCK_PARENT, Sx};

#[derive(Clone, Debug, PartialEq)]
struct FlattenedScope {
    parent_block_index: usize,
    selector: String,
    breakpoint: Option<Size>,
    start: usize,
    end: usize,
}

impl Sx {
    pub fn to_css(&self) -> Stylesheet {
        let class_name = self.class_name();
        let css_blocks = self.to_css_blocks(class_name_to_selector(&class_name));
        let mut stylesheet = Stylesheet::new();

        let mut index = 0;
        while index < css_blocks.len() {
            stylesheet = stylesheet.append_block(css_blocks[index].clone());
            index += 1;
        }

        stylesheet
    }

    fn to_css_blocks(&self, class_name: String) -> Vec<CssBlock> {
        let flattened_scopes = self.to_flattened_scopes(class_name);
        let mut css_blocks = Vec::new();

        let mut index = 0;
        while index < flattened_scopes.len() {
            css_blocks.push(to_css_block(
                flattened_scopes[index].clone(),
                self.declarations(),
                self.blocks(),
            ));
            index += 1;
        }

        css_blocks
    }

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
                SxModifier::Selector(suffix) => {
                    next_selector.push_str(suffix);
                }
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
    flattened_scope: FlattenedScope,
    declarations: &[Declaration],
    blocks: &[SxBlock],
) -> CssBlock {
    let mut scope = CssScope::from_string(flattened_scope.selector);

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
            CssMediaQuery::from_string(breakpoint_to_media_condition(breakpoint)).with(scope),
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

fn breakpoint_to_media_condition(breakpoint: Size) -> String {
    format!("(min-width: {})", breakpoint.breakpoint_value())
}

fn class_name_to_selector(class_name: &str) -> String {
    format!(".{class_name}")
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
