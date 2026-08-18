use crate::css::{CssDeclaration, CssScope, Stylesheet, condition_groups, expand_selector};
use crate::tokens::{ColorShade, ColorValue, NamedColorCss, Size};

use super::{Property, StaticSx, Sx, SxEntry, SxModifier, SxPropertyKey, ThemeAwareValue};

#[derive(Clone, Debug, PartialEq, Eq)]
struct CssContext {
    selectors: Vec<String>,
    media_query: Option<String>,
}

impl From<&Sx> for Stylesheet {
    fn from(sx: &Sx) -> Self {
        let mut scopes = Vec::new();
        let context = CssContext {
            selectors: vec![format!(".{}", sx.class_name())],
            media_query: None,
        };

        collect_scopes(&mut scopes, sx, &context);
        Stylesheet::new(scopes).with_class_name(sx.class_name())
    }
}

impl From<&StaticSx> for Stylesheet {
    fn from(sx: &StaticSx) -> Self {
        Stylesheet::from(std::ops::Deref::deref(sx))
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
        let mut scope = CssScope::new(context.selectors.join(", "), declarations);
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
        SxModifier::Selector(pattern) => CssContext {
            selectors: expand_selector(pattern, &context.selectors),
            media_query: context.media_query.clone(),
        },
        SxModifier::Condition(condition) => CssContext {
            selectors: condition_groups(condition)
                .into_iter()
                .flat_map(|group| {
                    context.selectors.iter().map(move |selector| {
                        let mut selector = selector.clone();
                        for state in &group {
                            selector.push_str(&format!("[data-state~=\"{state}\"]"));
                        }
                        selector
                    })
                })
                .collect(),
            media_query: context.media_query.clone(),
        },
        SxModifier::Breakpoint(size) => CssContext {
            selectors: context.selectors.clone(),
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
        _ => declarations.extend(property_declarations(property, value)),
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

    let mut scope = CssScope::new(
        context.selectors.join(", "),
        property_declarations(property, value),
    );
    let breakpoint_query = format!("(min-width: {})", size.breakpoint_value());
    let media_query = match &context.media_query {
        Some(existing) => format!("{existing} and {breakpoint_query}"),
        None => breakpoint_query,
    };
    scope = scope.in_media_query(media_query);
    scopes.push(scope);
}

/// `background`'s declaration doubles as the source for
/// `--lsx-focus-contrast`, an inheriting custom property any focusable
/// descendant's `:focus-visible` ring can read (with a fallback) to
/// contrast against whichever ancestor most recently set a background -
/// see `ThemeAwareValue::focus_contrast`.
fn property_declarations(property: &SxPropertyKey, value: &ThemeAwareValue) -> Vec<CssDeclaration> {
    let mut declarations = vec![CssDeclaration::new(
        property.as_str(),
        to_css_value(property, value),
    )];

    if matches!(property, SxPropertyKey::Known(Property::Background)) {
        if let Some(contrast) = value.focus_contrast() {
            declarations.push(CssDeclaration::new(
                NamedColorCss::FOCUS_CONTRAST.name(),
                contrast,
            ));
        }
    }

    declarations
}

fn to_css_value(property: &SxPropertyKey, value: &ThemeAwareValue) -> String {
    match value {
        ThemeAwareValue::Size(size) => to_size_css_value(property, *size),
        ThemeAwareValue::Number(value) => value.clone(),
        ThemeAwareValue::Color(color) => ColorValue::Shade(*color, ColorShade::S5).value(),
        ThemeAwareValue::ColorValue(value) => value.value(),
        ThemeAwareValue::CssVar(css_var) => css_var.value(),
        ThemeAwareValue::String(value) => value.clone(),
        ThemeAwareValue::RawColor(raw, _) => raw.clone(),
        ThemeAwareValue::BreakpointValue(_) => {
            unreachable!("breakpoint values are expanded before css value conversion")
        }
    }
}

fn to_size_css_value(property: &SxPropertyKey, size: Size) -> String {
    let scale = match property {
        SxPropertyKey::Known(property) => property.size_scale(),
        SxPropertyKey::Raw(_) => None,
    };

    match scale {
        Some(scale) => scale.value(size),
        None => size.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        sx::{bp, sx},
        tokens::Size,
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

    #[test]
    fn sx_font_properties_emit_correctly() {
        let stylesheet = Stylesheet::from(
            &sx()
                .font_family("var(--lsx-h1-font-family)")
                .font_size("var(--lsx-h1-font-size)")
                .font_weight("700")
                .letter_spacing("var(--lsx-h1-letter-spacing)")
                .line_height("var(--lsx-h1-line-height)")
                .margin("0"),
        );
        let css = stylesheet.as_str();

        assert!(css.contains("font-family:var(--lsx-h1-font-family);"));
        assert!(css.contains("font-size:var(--lsx-h1-font-size);"));
        assert!(css.contains("font-weight:700;"));
        assert!(css.contains("letter-spacing:var(--lsx-h1-letter-spacing);"));
        assert!(css.contains("line-height:var(--lsx-h1-line-height);"));
        assert!(css.contains("margin:0;"));
    }

    #[test]
    fn sx_border_radius_size_resolves_to_radius_css_var() {
        let stylesheet = Stylesheet::from(&sx().border_radius(Size::Md));
        let css = stylesheet.as_str();

        assert!(css.contains("border-radius:var(--lsx-radius-md);"));
    }

    #[test]
    fn sx_bare_padding_size_resolves_to_spacing_css_var() {
        let stylesheet = Stylesheet::from(&sx().padding(Size::Lg));
        let css = stylesheet.as_str();

        assert!(css.contains("padding:var(--lsx-spacing-lg);"));
    }

    #[test]
    fn sx_min_width_size_resolves_to_breakpoint_css_var() {
        let stylesheet = Stylesheet::from(&sx().min_width(Size::Sm));
        let css = stylesheet.as_str();

        assert!(css.contains("min-width:var(--lsx-breakpoint-sm);"));
    }

    #[test]
    fn sx_size_on_an_unscaled_property_keeps_its_own_name() {
        let stylesheet = Stylesheet::from(&sx().content(Size::Md));
        let css = stylesheet.as_str();

        assert!(css.contains("content:md;"));
    }

    #[test]
    fn sx_bare_color_defaults_to_shade_5_in_css() {
        let stylesheet = Stylesheet::from(&sx().color("primary"));
        let css = stylesheet.as_str();

        assert!(css.contains("color:var(--lsx-primary-5);"));
    }

    #[test]
    fn sx_when_and_condition_chains_attribute_selectors() {
        let stylesheet = Stylesheet::from(&sx().when("horizontal && label", sx().height("1px")));
        let css = stylesheet.as_str();

        assert!(css.contains("[data-state~=\"horizontal\"][data-state~=\"label\"]{height:1px;}"));
    }

    #[test]
    fn sx_when_or_condition_emits_comma_separated_selectors() {
        let stylesheet = Stylesheet::from(&sx().when("hover || focus", sx().color("red")));
        let css = stylesheet.as_str();
        let class = sx().when("hover || focus", sx().color("red")).class_name();

        assert!(css.contains(&format!(
            ".{class}[data-state~=\"hover\"], .{class}[data-state~=\"focus\"]{{color:red;}}"
        )));
    }

    #[test]
    fn sx_when_nested_conditions_distribute_over_or() {
        let stylesheet =
            Stylesheet::from(&sx().when("a || b", sx().when("c || d", sx().color("red"))));
        let css = stylesheet.as_str();

        for pair in [
            "a\"][data-state~=\"c",
            "a\"][data-state~=\"d",
            "b\"][data-state~=\"c",
            "b\"][data-state~=\"d",
        ] {
            assert!(
                css.contains(&format!("[data-state~=\"{pair}\"]")),
                "missing combination for {pair} in {css}"
            );
        }
    }

    #[test]
    fn sx_selector_ampersand_shares_one_nested_sx_across_a_comma_list() {
        let base = sx().selector("&::before, &::after", sx().height("1px"));
        let stylesheet = Stylesheet::from(&base);
        let css = stylesheet.as_str();
        let class = base.class_name();

        assert!(css.contains(&format!(".{class}::before, .{class}::after{{height:1px;}}")));
    }

    #[test]
    fn sx_selector_ampersand_matches_plain_suffix_form() {
        let ampersand_sx = sx().selector("&::before", sx().height("1px"));
        let plain_sx = sx().selector("::before", sx().height("1px"));

        let ampersand_css = Stylesheet::from(&ampersand_sx).as_str().to_string();
        let plain_css = Stylesheet::from(&plain_sx).as_str().to_string();

        assert!(ampersand_css.contains(&format!(
            ".{}::before{{height:1px;}}",
            ampersand_sx.class_name()
        )));
        assert!(plain_css.contains(&format!(
            ".{}::before{{height:1px;}}",
            plain_sx.class_name()
        )));
    }

    #[test]
    fn sx_selector_ampersand_with_space_expands_to_descendant_combinator() {
        // A plain (non-&) pattern is trimmed and then appended directly, so a
        // leading space meant as a descendant combinator (e.g. " ul") is lost
        // and silently produces an invalid selector. The `&`-form is not
        // trimmed away and is the correct way to express this.
        let descendant_sx = sx().selector("& ul", sx().height("1px"));
        let css = Stylesheet::from(&descendant_sx).as_str().to_string();

        assert!(css.contains(&format!(
            ".{} ul{{height:1px;}}",
            descendant_sx.class_name()
        )));
    }
}
