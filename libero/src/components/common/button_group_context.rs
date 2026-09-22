use dioxus::prelude::*;

use crate::{components::common::Variant, sx::ThemeAwareValue, theme::Size};

/// A `ButtonGroup`'s defaults for the buttons inside it; a button's own prop wins.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ButtonGroupContext {
    pub size: Option<Size>,
    pub radius: Option<Size>,
    pub variant: Option<Variant>,
    pub color: Option<ThemeAwareValue>,
    pub disabled: Option<bool>,
}

// A signal: a group re-rendered with new props must reach buttons its parent memoized.
#[derive(Clone, Copy)]
struct ButtonGroupScope(Signal<ButtonGroupContext>);

/// Hands `defaults` to the buttons below, updating them when they change.
pub(crate) fn use_provide_button_group(defaults: ButtonGroupContext) {
    let mut scope = use_context_provider(|| ButtonGroupScope(Signal::new(defaults.clone())));
    if *scope.0.peek() != defaults {
        scope.0.set(defaults);
    }
}

/// The enclosing group's defaults, all unset outside one.
pub(crate) fn use_button_group() -> ButtonGroupContext {
    let scope = use_hook(try_consume_context::<ButtonGroupScope>);
    scope
        .map(|scope| scope.0.read().clone())
        .unwrap_or_default()
}
