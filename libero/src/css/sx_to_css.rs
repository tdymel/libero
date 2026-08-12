use crate::{
    sx::{Property, Sx, SxEntry, SxModifier, SxPropertyKey},
    theme::{ColorValue, Size},
};

use super::{
    CssBlock, CssDeclaration, CssMediaQuery, CssScope, Stylesheet, StylesheetBuilder,
    css_var::SizeCssVar,
};

#[derive(Clone, Debug, PartialEq)]
struct CssContext {
    selector: String,
    breakpoint: Option<Size>,
}

impl From<&Sx> for Stylesheet {
    fn from(sx: &Sx) -> Self {
        let mut builder = StylesheetBuilder::new();
        let context = CssContext {
            selector: format!(".{}", sx.class_name()),
            breakpoint: None,
        };

        push_sx(&mut builder, sx, &context);
        builder.into()
    }
}

fn push_sx(builder: &mut StylesheetBuilder, sx: &Sx, context: &CssContext) {
    let declarations = sx
        .entries()
        .iter()
        .filter_map(|entry| match entry {
            SxEntry::Declaration { property, value } => Some(to_css_declaration(property, value)),
            SxEntry::Nested { .. } => None,
        })
        .collect::<Vec<_>>();

    if !declarations.is_empty() {
        let scope = declarations.into_iter().fold(
            CssScope::from_string(context.selector.clone()),
            |scope, declaration| scope.with(declaration),
        );

        let block = match context.breakpoint {
            Some(breakpoint) => CssBlock::MediaQuery(
                CssMediaQuery::from_string(format!(
                    "(min-width: {})",
                    breakpoint.breakpoint_value()
                ))
                .with(scope),
            ),
            None => CssBlock::Scope(scope),
        };

        let next_builder = std::mem::take(builder).with_block(block);
        *builder = next_builder;
    }

    for entry in sx.entries() {
        if let SxEntry::Nested { modifier, sx } = entry {
            let next = apply_modifier(context, modifier);
            push_sx(builder, sx, &next);
        }
    }
}

fn apply_modifier(context: &CssContext, modifier: &SxModifier) -> CssContext {
    match modifier {
        SxModifier::Selector(suffix) => CssContext {
            selector: format!("{}{}", context.selector, suffix),
            breakpoint: context.breakpoint,
        },
        SxModifier::Condition(condition) => CssContext {
            selector: format!("{}[data-state~=\"{}\"]", context.selector, condition),
            breakpoint: context.breakpoint,
        },
        SxModifier::Breakpoint(size) => CssContext {
            selector: context.selector.clone(),
            breakpoint: Some(merge_breakpoint(context.breakpoint, *size)),
        },
    }
}

fn to_css_declaration(property: &SxPropertyKey, value: &str) -> CssDeclaration {
    let property_name = property.as_str().to_string();
    let value = to_css_value(property, value);
    CssDeclaration::new(property_name, value)
}

fn to_css_value(property: &SxPropertyKey, value: &str) -> String {
    if let Some(color_value) = ColorValue::parse(value) {
        return color_value.css_value();
    }

    if let Some(size) = Size::parse_dynamic(value) {
        return match property {
            SxPropertyKey::Known(Property::PaddingTop) | SxPropertyKey::Known(Property::Gap) => {
                SizeCssVar::SPACING.value(size)
            }
            _ => size.as_str().to_string(),
        };
    }

    value.to_string()
}

fn merge_breakpoint(current: Option<Size>, next: Size) -> Size {
    match current {
        Some(current) if (next as u8) <= (current as u8) => current,
        _ => next,
    }
}
