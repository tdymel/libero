use crate::css::{AtRule, CssDeclaration, CssScope, Stylesheet, condition_groups, expand_selector};
use crate::tokens::{NamedColorCss, Size};
use crate::utils::warn;

use super::{
    Property, StaticSx, Sx, SxEntry, SxModifierKey, SxPropertyKey, ThemeAwareValue,
    class_name_from_hash,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct CssContext {
    selectors: Vec<String>,
    /// Ordered outermost-first, since `@media` and `@container` nest.
    at_rules: Vec<AtRule>,
}

impl CssContext {
    /// Appends `at_rule`, folding it into the innermost one where both are
    /// `@media` so nested breakpoints stay a single `and` query.
    fn wrapped_in(&self, at_rule: AtRule) -> Vec<AtRule> {
        let mut at_rules = self.at_rules.clone();
        match at_rules.last().and_then(|last| last.merged(&at_rule)) {
            Some(merged) => *at_rules.last_mut().expect("just merged into it") = merged,
            None => at_rules.push(at_rule),
        }
        at_rules
    }
}

/// Stands in while the CSS is built, so the sheet can be hashed before its
/// own name exists - see [`Sx::class_name`].
pub(crate) const ROOT_CLASS_PLACEHOLDER: &str = "\u{1}";

impl From<&Sx> for Stylesheet {
    fn from(sx: &Sx) -> Self {
        let mut scopes = Vec::new();
        let context = CssContext {
            selectors: vec![format!(".{ROOT_CLASS_PLACEHOLDER}")],
            at_rules: Vec::new(),
        };

        collect_scopes(&mut scopes, sx, &context);
        let stylesheet = Stylesheet::new(scopes);
        let class_name = class_name_from_hash(stylesheet.hash());
        stylesheet.with_root_class(ROOT_CLASS_PLACEHOLDER, class_name)
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
        scopes.push(
            CssScope::new(context.selectors.join(", "), declarations)
                .in_at_rules(context.at_rules.clone()),
        );
    }

    for entry in sx.entries() {
        if let SxEntry::Nested { modifier, sx } = entry {
            let next = apply_modifier(context, modifier);
            // An all-whitespace `when()`/`selector()` expands to nothing, and
            // a scope with no selector renders as a `{..}` block the browser
            // silently discards.
            if next.selectors.is_empty() {
                warn(&format!(
                    "Sx: {modifier:?} names no selector, its block is dropped."
                ));
                continue;
            }
            collect_scopes(scopes, sx, &next);
        }
    }
}

fn apply_modifier(context: &CssContext, modifier: &SxModifierKey) -> CssContext {
    match modifier {
        SxModifierKey::Selector(pattern) => CssContext {
            selectors: expand_selector(pattern, &context.selectors),
            at_rules: context.at_rules.clone(),
        },
        SxModifierKey::Condition(condition) => CssContext {
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
            at_rules: context.at_rules.clone(),
        },
        SxModifierKey::Breakpoint(size) => CssContext {
            selectors: context.selectors.clone(),
            at_rules: context.wrapped_in(breakpoint_at_rule(*size)),
        },
        SxModifierKey::Container { name, condition } => CssContext {
            selectors: context.selectors.clone(),
            at_rules: context.wrapped_in(AtRule::Container {
                name: name.clone(),
                condition: condition.clone(),
            }),
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
                push_breakpoint_declaration_scope(scopes, context, property, *size, value);
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
    value: &ThemeAwareValue,
) {
    scopes.push(
        CssScope::new(
            context.selectors.join(", "),
            property_declarations(property, value),
        )
        .in_at_rules(context.wrapped_in(breakpoint_at_rule(size))),
    );
}

fn breakpoint_at_rule(size: Size) -> AtRule {
    AtRule::Media(format!("(min-width: {})", size.breakpoint_value()))
}

/// `background` also publishes `--lsx-focus-contrast`, which inherits, so a
/// descendant's focus ring can contrast against the nearest ancestor
/// background - see `ThemeAwareValue::focus_contrast`.
fn property_declarations(property: &SxPropertyKey, value: &ThemeAwareValue) -> Vec<CssDeclaration> {
    let mut declarations = vec![CssDeclaration::new(
        property.as_str(),
        to_css_value(property, value),
    )];

    if matches!(property, SxPropertyKey::Known(Property::Background))
        && let Some(contrast) = value.focus_contrast()
    {
        declarations.push(CssDeclaration::new(
            NamedColorCss::FOCUS_CONTRAST.name(),
            contrast,
        ));
    }

    declarations
}

fn to_css_value(property: &SxPropertyKey, value: &ThemeAwareValue) -> String {
    let scale = match property {
        SxPropertyKey::Known(property) => property.size_scale(),
        SxPropertyKey::Raw(_) => None,
    };

    match value.resolve(scale) {
        Some(value) => value,
        // No scale for this property, so emit the bare keyword.
        None => match value {
            ThemeAwareValue::Size(size) => size.as_str().to_string(),
            ThemeAwareValue::BreakpointValue(_) => {
                unreachable!("breakpoint values are expanded before css value conversion")
            }
            _ => unreachable!("every other variant resolves"),
        },
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
    fn class_name_and_registry_hash_are_one_value() {
        let stylesheet = Stylesheet::from(&sx().padding("lg"));
        let class_name = class_name_from_hash(stylesheet.hash());

        assert_eq!(stylesheet.class_name(), Some(class_name.as_str()));
        assert!(stylesheet.as_str().contains(&format!(".{class_name}")));
        assert!(!stylesheet.as_str().contains(ROOT_CLASS_PLACEHOLDER));
    }

    #[test]
    fn identical_css_gets_one_class_name_however_it_was_built() {
        let direct = Stylesheet::from(&sx().padding("lg").color("red"));
        let composed = Stylesheet::from(&sx().padding("lg").and(sx().color("red")));

        assert_eq!(direct.as_str(), composed.as_str());
        assert_eq!(direct.hash(), composed.hash());
        assert_eq!(direct.class_name(), composed.class_name());
    }

    #[test]
    fn a_nested_block_is_merged_however_it_was_composed() {
        let direct = Stylesheet::from(&sx().hover(sx().color("red").background("blue")));
        let composed = Stylesheet::from(
            &sx()
                .hover(sx().color("red"))
                .and(sx().hover(sx().background("blue"))),
        );

        assert_eq!(direct.as_str(), composed.as_str());
        assert_eq!(direct.hash(), composed.hash());
    }

    #[test]
    fn merging_a_nested_block_lets_the_later_value_win() {
        let stylesheet = Stylesheet::from(&sx().hover(sx().color("red")).hover(sx().color("blue")));

        assert_eq!(stylesheet.as_str().matches(":hover").count(), 1);
        assert!(stylesheet.as_str().contains("blue"));
        assert!(!stylesheet.as_str().contains("red"));
    }

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
    fn sx_container_marks_and_queries_a_named_container() {
        let marker = Stylesheet::from(&sx().container("demo-card"));
        assert!(
            marker
                .as_str()
                .contains("container-type:inline-size;container-name:demo-card;")
        );

        let css = Stylesheet::from(&sx().container_query(
            "demo-card",
            "(min-width: 640px)",
            sx().width("388px"),
        ));
        assert!(
            css.as_str()
                .contains("@container demo-card (min-width: 640px){")
        );
        assert!(css.as_str().contains("width:388px;"));
    }

    #[test]
    fn sx_container_breakpoint_uses_the_size_scale() {
        let css = Stylesheet::from(&sx().container_breakpoint("card", Size::Md, sx().gap("lg")));

        assert!(css.as_str().contains("@container card (min-width: 62rem){"));
    }

    /// The `" and "` fold is media-only: a container nested with a breakpoint
    /// has to come out as two nested at-rules, whichever way round it is.
    #[test]
    fn a_container_and_a_breakpoint_nest_in_either_order() {
        let media_outside = Stylesheet::from(&sx().breakpoint(
            Size::Md,
            sx().container_query("card", "(min-width: 640px)", sx().width("388px")),
        ));
        assert!(
            media_outside
                .as_str()
                .contains("@media (min-width: 62rem){@container card (min-width: 640px){")
        );

        let container_outside = Stylesheet::from(&sx().container_query(
            "card",
            "(min-width: 640px)",
            sx().breakpoint(Size::Md, sx().width("388px")),
        ));
        assert!(
            container_outside
                .as_str()
                .contains("@container card (min-width: 640px){@media (min-width: 62rem){")
        );

        assert_eq!(
            media_outside.as_str().matches("@media").count()
                + media_outside.as_str().matches("@container").count(),
            2
        );
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
    fn sx_bare_color_defaults_to_shade_6_in_css() {
        let stylesheet = Stylesheet::from(&sx().color("primary"));
        let css = stylesheet.as_str();

        assert!(css.contains("color:var(--lsx-primary-6);"));
    }

    #[test]
    fn sx_when_and_condition_chains_attribute_selectors() {
        let stylesheet = Stylesheet::from(&sx().when("horizontal && label", sx().height("1px")));
        let css = stylesheet.as_str();

        assert!(css.contains("[data-state~=\"horizontal\"][data-state~=\"label\"]{height:1px;}"));
    }

    /// Without the guard these render `.lsx-x{color:blue;}{color:red;}` - a
    /// selectorless block the browser drops with no diagnostic.
    #[test]
    fn a_modifier_naming_no_selector_drops_its_block() {
        for base in [
            sx().color("blue").when("", sx().color("red")),
            sx().color("blue").when("  ", sx().color("red")),
            sx().color("blue").selector("", sx().color("red")),
            sx().color("blue").selector(" , ", sx().color("red")),
        ] {
            let css = Stylesheet::from(&base).as_str().to_string();

            assert!(!css.contains("{color:red;}"), "kept a dropped block: {css}");
            assert!(!css.contains("}{"), "emitted a selectorless block: {css}");
        }
    }

    #[test]
    fn sx_when_equivalent_spellings_share_one_class() {
        assert_eq!(
            sx().when("a&&b", sx().color("red")).class_name(),
            sx().when("  a &&  b ", sx().color("red")).class_name()
        );
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
        // A plain pattern is trimmed before being appended, so a leading
        // space meant as a descendant combinator is silently lost. `&` is the
        // correct form.
        let descendant_sx = sx().selector("& ul", sx().height("1px"));
        let css = Stylesheet::from(&descendant_sx).as_str().to_string();

        assert!(css.contains(&format!(
            ".{} ul{{height:1px;}}",
            descendant_sx.class_name()
        )));
    }
}
