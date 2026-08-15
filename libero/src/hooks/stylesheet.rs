use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{CssLayer, context::LiberoContext, css::Stylesheet};

/// Registers anything that converts into a [`Stylesheet`] and returns the
/// class name to put on any element - including ones libero doesn't provide
/// a component for (e.g. a raw `select`). Uses the `UserCustom` layer, which
/// has priority over every other layer (including a component's own dynamic
/// prop-driven styles), so it always wins. Raw CSS built via `Stylesheet`'s
/// `&str`/`String` escape hatch carries no class name, since it isn't scoped
/// to a single selector.
pub fn use_stylesheet(stylesheet: impl Into<Stylesheet>) -> Option<String> {
    use_css(stylesheet, CssLayer::UserCustom)
}

/// Same as [`use_stylesheet`], but on a caller-chosen layer instead of
/// always `UserCustom` - used internally so component-provided styles can
/// sit below user overrides in the cascade.
pub(crate) fn use_css(stylesheet: impl Into<Stylesheet>, layer: CssLayer) -> Option<String> {
    let stylesheet = stylesheet.into();
    let is_empty = stylesheet.as_str().is_empty();
    let class_name = stylesheet.class_name().map(str::to_string);

    let mut context = use_context::<LiberoContext>();
    let key = (layer, stylesheet.hash());
    let active_registration = use_hook(|| Rc::new(RefCell::new(None::<(CssLayer, u64)>)));

    {
        let mut active_registration = active_registration.borrow_mut();

        if is_empty {
            if let Some(active_key) = active_registration.take() {
                context.stylesheet_registry.release(active_key);
                *context.stylesheet_registry_version.write() += 1;
            }
        } else {
            match *active_registration {
                Some(active_key) if active_key == key => {}
                Some(active_key) => {
                    context.stylesheet_registry.release(active_key);
                    context.stylesheet_registry.acquire(stylesheet, layer);
                    *active_registration = Some(key);
                    *context.stylesheet_registry_version.write() += 1;
                }
                None => {
                    context.stylesheet_registry.acquire(stylesheet, layer);
                    *active_registration = Some(key);
                    *context.stylesheet_registry_version.write() += 1;
                }
            }
        }
    }

    {
        let active_registration = active_registration.clone();
        let stylesheet_registry = context.stylesheet_registry.clone();
        let mut stylesheet_registry_version = context.stylesheet_registry_version;
        use_drop(move || {
            if let Some(active_key) = active_registration.borrow_mut().take() {
                stylesheet_registry.release(active_key);
                *stylesheet_registry_version.write() += 1;
            }
        });
    }

    if is_empty { None } else { class_name }
}
