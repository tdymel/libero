use dioxus::prelude::*;
use indexmap::IndexMap;
use std::fmt;

use crate::LiberoContext;

use super::{NestedRule, SxDyn, optimize_styles};

#[derive(Clone, Copy)]
pub struct SxContext {
    registry: Signal<IndexMap<String, SxDyn>>,
    stylesheet: Signal<String>,
}

impl Default for SxContext {
    fn default() -> Self {
        let registry = Signal::new(IndexMap::new());
        let stylesheet = Signal::new(String::new());

        Self {
            registry,
            stylesheet,
        }
    }
}

pub(crate) fn class_name_from_id(id: &str) -> String {
    format!("lsx-{}", id)
}

fn style_id(sx: &SxDyn) -> String {
    // TODO: Not the most efficient way!
    let hash = blake3::hash(sx.to_string().as_bytes());
    hash.to_hex()[..16].to_string()
}

fn render_rule(context: &LiberoContext, selector: &str, sx: &SxDyn) -> Vec<String> {
    render_rule_with_media(context, selector, sx, &[])
}

fn declarations_to_string(sx: &SxDyn) -> String {
    struct DeclarationsOnly<'a>(&'a SxDyn);

    impl fmt::Display for DeclarationsOnly<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt_declarations_only(f)
        }
    }

    DeclarationsOnly(sx).to_string()
}

fn resolve_media_query<'a>(
    context: &LiberoContext,
    nested_rule: &'a NestedRule,
) -> Option<(String, &'a SxDyn)> {
    match nested_rule {
        NestedRule::Media { query, sx } => Some((query.clone(), sx)),
        NestedRule::MediaUp { size, sx } => Some((context.theme.media_up(*size), sx)),
        NestedRule::MediaDown { size, sx } => Some((context.theme.media_down(*size), sx)),
        NestedRule::MediaBetween { min, max, sx } => {
            Some((context.theme.media_between(*min, *max), sx))
        }
        _ => None,
    }
}

fn render_rule_with_media(
    context: &LiberoContext,
    selector: &str,
    sx: &SxDyn,
    media_stack: &[String],
) -> Vec<String> {
    let mut rules = Vec::new();

    if !sx.declarations.is_empty() {
        let mut rule = format!("{} {{ {} }}", selector, declarations_to_string(sx));

        for query in media_stack.iter().rev() {
            rule = format!("@media {} {{ {} }}", query, rule);
        }

        rules.push(rule);
    }

    // Reuse a single owned media stack to avoid allocating a new Vec for every nested media rule.
    let mut owned_stack = media_stack.to_vec();

    for nested_rule in &sx.nested_rules {
        if let NestedRule::Selector {
            fragment,
            sx: nested_sx,
        } = nested_rule
        {
            rules.extend(render_rule_with_media(
                context,
                &fragment.replace('&', selector),
                nested_sx,
                &owned_stack,
            ));
            continue;
        }

        if let Some((query, nested_sx)) = resolve_media_query(context, nested_rule) {
            owned_stack.push(query);
            rules.extend(render_rule_with_media(
                context,
                selector,
                nested_sx,
                &owned_stack,
            ));
            owned_stack.pop();
        }
    }

    rules
}

impl SxContext {
    pub fn upsert(&self, context: &LiberoContext, id: String, sx: SxDyn) {
        let mut registry = self.registry;
        let should_update = registry.read().get(&id) != Some(&sx);

        if should_update {
            registry.write().insert(id, sx);
            self.rebuild_stylesheet(context);
        }
    }

    fn rebuild_stylesheet(&self, context: &LiberoContext) {
        let stylesheet = optimize_styles(&self.registry.read())
            .into_iter()
            .flat_map(|rule| render_rule(context, &rule.selector, &rule.sx))
            .collect::<Vec<_>>()
            .join("\n");

        let mut stylesheet_signal = self.stylesheet;
        *stylesheet_signal.write() = stylesheet;
    }

    pub fn stylesheet(&self) -> ReadSignal<String> {
        self.stylesheet.into()
    }
}

pub fn use_sx<S>(sx: S) -> String
where
    S: Into<SxDyn> + Clone + PartialEq + 'static,
{
    let context = use_context::<LiberoContext>();
    let snapshot = use_memo(move || sx.clone().into());
    let resolved_sx = snapshot.read().clone();
    let id = style_id(&resolved_sx);

    // TODO: avoid cloning and re-registering on every render; this is currently
    // done synchronously so SSR/fullstack can emit styles before hydration.
    context.sx.upsert(&context, id.clone(), resolved_sx);

    class_name_from_id(id.as_str())
}
