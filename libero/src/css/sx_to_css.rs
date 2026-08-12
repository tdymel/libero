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
    let mut declarations = Vec::new();

    for entry in sx.entries() {
        match entry {
            SxEntry::Declaration { property, value } => {
                collect_declaration_scopes(scopes, &mut declarations, context, property, value)
            }
            SxEntry::Nested { .. } => {}
        }
    }

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

fn collect_declaration_scopes(
    scopes: &mut Vec<CssScope>,
    declarations: &mut Vec<CssDeclaration>,
    context: &CssContext,
    property: &SxPropertyKey,
    value: &ThemeAwareValue,
) {
    match value {
        ThemeAwareValue::BreakpointValue(breakpoint_value) => {
            for (size, value) in breakpoint_value.values() {
                push_breakpoint_declaration_scope(scopes, context, property, *size, Some(value));
            }
        }
        _ => declarations.push(CssDeclaration::new(
            property.as_str(),
            to_css_value(property, value),
        )),
    }
}

fn push_breakpoint_declaration_scope(
    scopes: &mut Vec<CssScope>,
    context: &CssContext,
    property: &SxPropertyKey,
    size: Size,
    value: Option<&ThemeAwareValue>,
) {
    let Some(value) = value else {
        return;
    };

    let declaration = CssDeclaration::new(property.as_str(), to_css_value(property, value));
    let mut scope = CssScope::new(context.selector.clone(), vec![declaration]);
    let breakpoint_query = format!("(min-width: {})", size.breakpoint_value());
    let media_query = match &context.media_query {
        Some(existing) => format!("{existing} and {breakpoint_query}"),
        None => breakpoint_query,
    };
    scope = scope.in_media_query(media_query);
    scopes.push(scope);
}

fn to_css_value(property: &SxPropertyKey, value: &ThemeAwareValue) -> String {
    match value {
        ThemeAwareValue::Size(size) => to_size_css_value(property, *size),
        ThemeAwareValue::Number(value) => value.clone(),
        ThemeAwareValue::ColorValue(value) => CssColorValue(*value).value(),
        ThemeAwareValue::CssVar(css_var) => css_var.value(),
        ThemeAwareValue::String(value) => value.clone(),
        ThemeAwareValue::BreakpointValue(_) => {
            unreachable!("breakpoint values are expanded before css value conversion")
        }
    }
}

fn to_size_css_value(property: &SxPropertyKey, size: Size) -> String {
    match property {
        SxPropertyKey::Known(Property::PaddingTop)
        | SxPropertyKey::Known(Property::PaddingLeft)
        | SxPropertyKey::Known(Property::PaddingRight)
        | SxPropertyKey::Known(Property::Gap) => SizeCss::SPACING.value(size),
        SxPropertyKey::Known(Property::MaxWidth) => SizeCss::BREAKPOINT.value(size),
        _ => size.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        sx::{bp, sx},
        theme::Size,
    };

    use super::*;

    #[test]
    fn sx_breakpoint_value_emits_media_scopes() {
        let stylesheet = Stylesheet::from(
            &sx()
                .with(
                    "color",
                    bp().sm("primary.7").xl("secondary.3").sm("primary.1"),
                )
                .breakpoint(
                    Size::Md,
                    sx().with("background", bp().lg("secondary.2").lg("secondary.4")),
                ),
        );
        let css = stylesheet.as_str();

        assert!(css.contains("@media (min-width: 48rem){"));
        assert!(css.contains("@media (min-width: 88rem){"));
        assert!(css.contains("color:var(--lsx-primary-1);"));
        assert!(css.contains("color:var(--lsx-secondary-3);"));
        assert!(!css.contains("color:var(--lsx-primary-7);"));

        assert!(css.contains("@media (min-width: 62rem) and (min-width: 75rem){"));
        assert!(css.contains("background:var(--lsx-secondary-4);"));
        assert!(!css.contains("background:var(--lsx-secondary-2);"));
    }
}
