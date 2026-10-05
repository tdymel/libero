use std::rc::Rc;

use dioxus::prelude::*;

use super::{CssLayer, LiberoContext, SheetRank};
use crate::{
    platform::{self, A11yAnswers, answer_a11y_media, document, focus_selectors},
    theme::{physical_text_align, themed_form_controls},
};

/// The theme's own `<style>`, its own component so a rebuilt sheet re-renders
/// this leaf instead of everything under `{children}`. `outer` is an enclosing
/// provider's sheet: the same text is already in the document (todo 2207).
#[component]
pub(super) fn ThemeStyle(outer: Option<Signal<Rc<str>>>) -> Element {
    let context = use_context::<LiberoContext>();
    let sheet = context.theme_css.read().clone();
    let shared = outer.is_some_and(|outer| {
        let outer = outer.read();
        Rc::ptr_eq(&outer, &sheet) || **outer == *sheet
    });
    let css = (!shared).then(|| focus_selectors(&sheet));
    // Bumped per rebuilt sheet, for `SheetWatch` (todo 871).
    let version = use_hook(|| Rc::new(std::cell::Cell::new((0u64, sheet.clone()))));
    let (mut count, last) = version.take();
    if !Rc::ptr_eq(&last, &sheet) {
        count += 1;
    }
    version.set((count, sheet.clone()));

    // Every rebuilt sheet after the first, once it is in the document.
    let theme_css = context.theme_css;
    let mounted = use_hook(|| Rc::new(std::cell::Cell::new(false)));
    use_effect(move || {
        theme_css.read();
        if mounted.replace(true)
            && let Some(document) = document()
        {
            document.colors_changed();
        }
    });

    // Fixed text, so only the outermost provider writes it.
    let physical = use_hook(|| {
        (outer.is_none() && !platform::aligns_logical_text()).then(physical_text_align)
    });
    let form_controls = use_hook(|| {
        (outer.is_none() && !platform::colors_form_controls()).then(themed_form_controls)
    });

    rsx! {
        if let Some(css) = css {
            style { dangerous_inner_html: "{css}" }
        }
        if let Some(physical) = physical {
            style { dangerous_inner_html: "{physical}" }
        }
        if let Some(form_controls) = form_controls {
            style { dangerous_inner_html: "{form_controls}" }
        }
        {platform::SheetWatch(count)}
    }
}

/// The `<style>` nodes for everything registered so far. A leaf, so registering
/// re-renders only it; after `{children}`, so the first pass has every sheet.
#[component]
pub(super) fn StyleOutlet() -> Element {
    let context = use_context::<LiberoContext>();
    let registry_version = *context.stylesheet_registry_version.read();
    // Blitz's stylo and a forced reduced motion need libero's answers in the text (todo 954).
    let answers = context.a11y_answers();
    // After mount, so a hydrating client's first render matches the server's.
    let mut blocks = use_signal(|| false);
    use_effect(move || {
        if platform::edits_style_rules() {
            blocks.set(true);
        }
    });
    let registry = context.stylesheet_registry.clone();

    rsx! {
        if let Some(properties) = platform::scroll_padding_properties() {
            style { dangerous_inner_html: properties }
        }
        // The focus ring, ahead of every component rule in its layer.
        for (node_key, stylesheet) in context.stylesheet_registry.stylesheets(SheetRank::Default) {
            style {
                key: "{node_key}",
                dangerous_inner_html: "{renderer_css(&stylesheet, &answers)}"
            }
        }
        if blocks() {
            style {
                dangerous_inner_html: CssLayer::blocks_css(),
                onmounted: {
                    let registry = registry.clone();
                    move |event: MountedEvent| {
                        if let Some(rules) = platform::style_rules(&event.data()) {
                            registry.attach(rules);
                        }
                    }
                },
            }
        }
        for (node_key, stylesheet) in context.stylesheet_registry.stylesheets(SheetRank::Component) {
            style {
                key: "{node_key}",
                dangerous_inner_html: "{renderer_css(&stylesheet, &answers)}"
            }
        }
        {platform::SheetWatch(registry_version)}
    }
}

/// `css` as this renderer matches it.
fn renderer_css(css: &str, answers: &A11yAnswers) -> String {
    answer_a11y_media(&focus_selectors(css), answers).into_owned()
}
