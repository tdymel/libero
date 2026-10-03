use crate::css::{AtRule, CssDeclaration, CssScope, Stylesheet, condition_groups, expand_selector};
use crate::tokens::{FOCUS_RING_HALO, NamedColorCss, Size};
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
    /// Appends `at_rule`, folding two `@media` into one `and` query.
    fn wrapped_in(&self, at_rule: AtRule) -> Vec<AtRule> {
        let mut at_rules = self.at_rules.clone();
        match at_rules.last().and_then(|last| last.merged(&at_rule)) {
            Some(merged) => *at_rules.last_mut().expect("just merged into it") = merged,
            None => at_rules.push(at_rule),
        }
        at_rules
    }
}

/// The class name's stand-in, so the sheet can be hashed before its name exists.
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
    let mut breakpoint_scopes = Vec::new();

    for entry in sx.entries() {
        match entry {
            SxEntry::Declaration { property, value } => collect_declaration_scopes(
                &mut breakpoint_scopes,
                &mut declarations,
                context,
                property,
                value,
            ),
            SxEntry::Nested { .. } => {}
        }
    }

    if !declarations.is_empty() {
        scopes.push(
            CssScope::new(context.selectors.join(", "), declarations)
                .in_at_rules(context.at_rules.clone()),
        );
    }

    // Mobile-first `min-width` queries, so `bp()` steps sort smallest first (stable):
    // the widest matching one wins, whatever order they were written in.
    breakpoint_scopes.sort_by_key(|(size, _): &(Size, CssScope)| size.index());
    scopes.extend(breakpoint_scopes.into_iter().map(|(_, scope)| scope));

    let mut widest_breakpoint = None::<Size>;
    for entry in sx.entries() {
        if let SxEntry::Nested { modifier, sx } = entry {
            if let SxModifierKey::Breakpoint(size) = modifier {
                match widest_breakpoint {
                    Some(widest) if widest.index() > size.index() => {
                        warn_breakpoint_order(widest, *size);
                    }
                    _ => widest_breakpoint = Some(*size),
                }
            }
            let next = apply_modifier(context, modifier);
            // A blank `when()`/`selector()` would render a selectorless block the browser drops.
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

/// Nested `.breakpoint(..)` blocks keep their position (unlike `bp()`), so a narrower
/// one after a wider one beats it. Their order is the contract, so warn rather than sort.
fn warn_breakpoint_order(widest: Size, size: Size) {
    warn(&format!(
        "Sx: breakpoint({size:?}) comes after breakpoint({widest:?}), so it wins over it \
         wherever both match. Write nested breakpoints smallest first."
    ));
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
        SxModifierKey::Media(query) => CssContext {
            selectors: context.selectors.clone(),
            at_rules: context.wrapped_in(AtRule::Media(query.clone())),
        },
        SxModifierKey::Supports(condition) => CssContext {
            selectors: context.selectors.clone(),
            at_rules: context.wrapped_in(AtRule::Supports(condition.clone())),
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
    breakpoint_scopes: &mut Vec<(Size, CssScope)>,
    declarations: &mut Vec<CssDeclaration>,
    context: &CssContext,
    property: &SxPropertyKey,
    value: &ThemeAwareValue,
) {
    match value {
        ThemeAwareValue::BreakpointValue(breakpoint_value) => {
            for (size, value) in breakpoint_value.values() {
                breakpoint_scopes.push((
                    *size,
                    breakpoint_declaration_scope(context, property, *size, value),
                ));
            }
        }
        _ => declarations.extend(property_declarations(property, value)),
    }
}

fn breakpoint_declaration_scope(
    context: &CssContext,
    property: &SxPropertyKey,
    size: Size,
    value: &ThemeAwareValue,
) -> CssScope {
    CssScope::new(
        context.selectors.join(", "),
        property_declarations(property, value),
    )
    .in_at_rules(context.wrapped_in(breakpoint_at_rule(size)))
}

fn breakpoint_at_rule(size: Size) -> AtRule {
    AtRule::Media(format!("(min-width: {})", size.breakpoint_value()))
}

/// A background also publishes the inherited `--lsx-focus-contrast` for descendants' focus
/// rings, and itself as the ring's halo (todo 630).
fn property_declarations(property: &SxPropertyKey, value: &ThemeAwareValue) -> Vec<CssDeclaration> {
    let css_value = to_css_value(property, value);
    let mut declarations = vec![CssDeclaration::new(property.as_str(), css_value.clone())];

    if matches!(
        property,
        SxPropertyKey::Known(Property::Background | Property::BackgroundColor)
    ) && let Some(contrast) = value.focus_contrast()
    {
        declarations.push(CssDeclaration::new(
            NamedColorCss::FOCUS_CONTRAST.name(),
            contrast,
        ));
        declarations.push(CssDeclaration::new(FOCUS_RING_HALO.name(), css_value));
    }

    declarations
}

fn to_css_value(property: &SxPropertyKey, value: &ThemeAwareValue) -> String {
    let (scale, role) = match property {
        SxPropertyKey::Known(property) => (property.size_scale(), property.color_role()),
        // Unknown target: the writing component picks the role when it computes the value.
        SxPropertyKey::Raw(_) => (None, None),
    };

    #[cfg(debug_assertions)]
    if let (SxPropertyKey::Known(property), ThemeAwareValue::String(raw)) = (property, value) {
        super::unknown_color::warn_unknown_color_name(*property, raw);
    }

    let in_role;
    let value = match role {
        Some(role) => {
            in_role = value.in_color_role(role);
            &in_role
        }
        None => value,
    };

    match value.resolve(scale) {
        Some(value) => value,
        None => match value {
            ThemeAwareValue::Size(size) => size.as_str().to_string(),
            ThemeAwareValue::NegativeSize(size) => format!("-{}", size.as_str()),
            ThemeAwareValue::BreakpointValue(_) => {
                unreachable!("breakpoint values are expanded before css value conversion")
            }
            _ => unreachable!("every other variant resolves"),
        },
    }
}

#[cfg(test)]
mod tests;
