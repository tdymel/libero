use crate::{
    sx::{Property, Sx, SxEntry, SxModifier, SxPropertyKey, ThemeAwareValue},
    theme::{Size, SizeCss},
};

use super::{CssDeclaration, CssScope, Stylesheet, css_color_value::CssColorValue};

#[derive(Clone, Debug, PartialEq, Eq)]
struct CssContext {
    selector: String,
    media_query: Option<String>,
}

impl From<&Sx> for Stylesheet {
    fn from(sx: &Sx) -> Self {
        let mut scopes = Vec::new();
        let context = CssContext {
            selector: format!(".{}", sx.class_name()),
            media_query: None,
        };

        collect_scopes(&mut scopes, sx, &context);
        Stylesheet::new(scopes)
    }
}

fn collect_scopes(scopes: &mut Vec<CssScope>, sx: &Sx, context: &CssContext) {
    let declarations = sx
        .entries()
        .iter()
        .filter_map(|entry| match entry {
            SxEntry::Declaration { property, value } => Some(to_css_declaration(property, value)),
            SxEntry::Nested { .. } => None,
        })
        .collect::<Vec<_>>();

    if !declarations.is_empty() {
        let mut scope = CssScope::new(context.selector.clone(), declarations);
        if let Some(media_query) = &context.media_query {
            scope = scope.in_media_query(media_query.clone());
        }
        scopes.push(scope);
    }

    for entry in sx.entries() {
        if let SxEntry::Nested { modifier, sx } = entry {
            let next = apply_modifier(context, modifier);
            collect_scopes(scopes, sx, &next);
        }
    }
}

fn apply_modifier(context: &CssContext, modifier: &SxModifier) -> CssContext {
    match modifier {
        SxModifier::Selector(suffix) => CssContext {
            selector: format!("{}{}", context.selector, suffix),
            media_query: context.media_query.clone(),
        },
        SxModifier::Condition(condition) => CssContext {
            selector: format!("{}[data-state~=\"{}\"]", context.selector, condition),
            media_query: context.media_query.clone(),
        },
        SxModifier::Breakpoint(size) => CssContext {
            selector: context.selector.clone(),
            media_query: Some(format!("(min-width: {})", size.breakpoint_value())),
        },
    }
}

fn to_css_declaration(property: &SxPropertyKey, value: &ThemeAwareValue) -> CssDeclaration {
    CssDeclaration::new(property.as_str(), to_css_value(property, value))
}

fn to_css_value(property: &SxPropertyKey, value: &ThemeAwareValue) -> String {
    match value {
        ThemeAwareValue::Size(size) => to_size_css_value(property, *size),
        ThemeAwareValue::Number(value) => value.clone(),
        ThemeAwareValue::ColorValue(value) => CssColorValue(*value).value(),
        ThemeAwareValue::CssVar(css_var) => css_var.value(),
        ThemeAwareValue::String(value) => value.clone(),
    }
}

fn to_size_css_value(property: &SxPropertyKey, size: Size) -> String {
    match property {
        SxPropertyKey::Known(Property::PaddingTop) | SxPropertyKey::Known(Property::Gap) => {
            SizeCss::SPACING.value(size)
        }
        _ => size.as_str().to_string(),
    }
}
