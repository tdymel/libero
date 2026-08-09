use crate::common::ConstStr;

use super::{
    breakpoint::Breakpoint, declaration::Declaration, sx::Sx, sx_block::SxBlock,
    sx_modifier::SxModifier,
};

pub(super) const DEFAULT_SX_CSS_CAPACITY: usize = 4096;
pub(super) const ROOT_BLOCK_PARENT: usize = usize::MAX;

pub(super) const fn to_css(sx: &Sx, class_name: &'static str) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
    let blocks = sx.blocks();
    let declarations = sx.declarations();
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

const fn emit_node(
    mut css: ConstStr<DEFAULT_SX_CSS_CAPACITY>,
    declarations: &[Declaration],
    blocks: &[SxBlock],
    parent_block_index: usize,
    selector: &str,
    breakpoint: Option<Breakpoint>,
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
    breakpoint: Option<Breakpoint>,
    start: usize,
    end: usize,
) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
    if !has_owned_declarations(declarations, blocks, parent_block_index, start, end) {
        return css;
    }

    if let Some(breakpoint) = breakpoint {
        css = css.push_str("@media (min-width: ");
        css = css.push_str(breakpoint.value());
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
        css = css.push_str(declaration.property);
        css = css.push_char(':');
        css = push_theme_aware_value(css, declaration.value);
        css = css.push_char(';');
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

const fn push_theme_aware_value(
    mut css: ConstStr<DEFAULT_SX_CSS_CAPACITY>,
    value: &'static str,
) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
    if let Some((palette, shade)) = parse_palette_token(value) {
        css = css.push_str("var(--lsx-");
        css = css.push_str(palette);
        css = css.push_char('-');
        css = css.push_char(shade_hundreds_digit(shade));
        css = css.push_char(shade_tens_digit(shade));
        css = css.push_char('0');
        css = css.push_char(')');
        return css;
    }

    css.push_str(value)
}

const fn parse_palette_token(value: &'static str) -> Option<(&'static str, u8)> {
    if starts_with(value, "primary") {
        return Some(("primary", parse_shade(value, 7)));
    }

    if starts_with(value, "secondary") {
        return Some(("secondary", parse_shade(value, 9)));
    }

    None
}

const fn starts_with(value: &str, prefix: &str) -> bool {
    let value = value.as_bytes();
    let prefix = prefix.as_bytes();

    if prefix.len() > value.len() {
        return false;
    }

    let mut i = 0;
    while i < prefix.len() {
        if value[i] != prefix[i] {
            return false;
        }
        i += 1;
    }

    true
}

const fn parse_shade(value: &'static str, palette_len: usize) -> u8 {
    let bytes = value.as_bytes();
    if bytes.len() == palette_len {
        return 5;
    }

    if bytes.len() == palette_len + 2 && bytes[palette_len] == b'.' {
        let shade = bytes[palette_len + 1];
        if shade >= b'0' && shade <= b'9' {
            return shade - b'0';
        }
    }

    panic!("invalid palette token")
}

const fn shade_hundreds_digit(shade: u8) -> char {
    if shade == 0 {
        '0'
    } else {
        ((b'0' + shade) as char)
    }
}

const fn shade_tens_digit(shade: u8) -> char {
    if shade == 0 { '0' } else { '0' }
}

const fn merge_breakpoint(current: Option<Breakpoint>, next: Breakpoint) -> Breakpoint {
    match current {
        Some(current) => {
            if breakpoint_order(next) > breakpoint_order(current) {
                next
            } else {
                current
            }
        }
        None => next,
    }
}

const fn breakpoint_order(breakpoint: Breakpoint) -> usize {
    match breakpoint {
        Breakpoint::XS => 0,
        Breakpoint::S => 1,
        Breakpoint::M => 2,
        Breakpoint::L => 3,
        Breakpoint::XL => 4,
    }
}
