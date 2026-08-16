use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    CssLayer,
    context::LiberoContext,
    css::Stylesheet,
    sx::{StaticSx, Sx},
};

/// Registers anything that converts into a [`Stylesheet`] and returns its
/// class name, on the `UserCustom` layer so it always wins the cascade. Raw
/// `&str`/`String` CSS has no single selector, so it returns no class name.
pub fn use_stylesheet(stylesheet: impl Into<Stylesheet>) -> Option<String> {
    use_css(stylesheet.into(), CssLayer::UserCustom)
}

/// A value [`use_css`] can register. Separates a cheap identity check
/// (`identity_hash`/`class_name`) from the expensive CSS text build
/// (`build`), so a render whose `Sx` is unchanged from the last one never
/// has to walk its entry tree or format any CSS - `Sx::hash` is a cheap
/// structural hash over the already-in-memory entries, unlike
/// `Stylesheet::hash`, which needs the rendered text to exist first.
pub(crate) trait CssSource {
    fn identity_hash(&self) -> u64;
    fn css_class_name(&self) -> Option<String>;
    fn build(self) -> Stylesheet;
}

impl CssSource for &Sx {
    fn identity_hash(&self) -> u64 {
        self.hash()
    }

    fn css_class_name(&self) -> Option<String> {
        Some(self.class_name())
    }

    fn build(self) -> Stylesheet {
        Stylesheet::from(self)
    }
}

impl CssSource for &StaticSx {
    fn identity_hash(&self) -> u64 {
        self.hash()
    }

    fn css_class_name(&self) -> Option<String> {
        Some(self.class_name())
    }

    fn build(self) -> Stylesheet {
        Stylesheet::from(self)
    }
}

impl CssSource for Stylesheet {
    fn identity_hash(&self) -> u64 {
        self.hash()
    }

    fn css_class_name(&self) -> Option<String> {
        self.class_name().map(str::to_string)
    }

    fn build(self) -> Stylesheet {
        self
    }
}

struct CssRegistration {
    hash: u64,
    registered: bool,
}

/// Same as [`use_stylesheet`], but on a caller-chosen layer.
pub(crate) fn use_css(source: impl CssSource, layer: CssLayer) -> Option<String> {
    let identity_hash = source.identity_hash();
    let mut context = use_context::<LiberoContext>();
    let state = use_hook(|| Rc::new(RefCell::new(None::<CssRegistration>)));

    let class_name = {
        let mut state_ref = state.borrow_mut();

        match state_ref.take() {
            // Same `Sx` as last render (by content, not identity) - the
            // registry entry (or lack of one) is already correct, so skip
            // rebuilding the CSS text entirely.
            Some(prev) if prev.hash == identity_hash => {
                let class_name = if prev.registered {
                    source.css_class_name()
                } else {
                    None
                };
                *state_ref = Some(prev);
                class_name
            }
            prev => {
                let stylesheet = source.build();
                let is_empty = stylesheet.as_str().is_empty();
                let class_name = stylesheet.class_name().map(str::to_string);
                let mut changed = false;

                if let Some(prev) = prev {
                    if prev.registered {
                        context.stylesheet_registry.release((layer, prev.hash));
                        changed = true;
                    }
                }

                let registered = if is_empty {
                    false
                } else {
                    context.stylesheet_registry.acquire(stylesheet, layer);
                    changed = true;
                    true
                };

                *state_ref = Some(CssRegistration {
                    hash: identity_hash,
                    registered,
                });

                if changed {
                    *context.stylesheet_registry_version.write() += 1;
                }

                if is_empty { None } else { class_name }
            }
        }
    };

    {
        let state = state.clone();
        let stylesheet_registry = context.stylesheet_registry.clone();
        let mut stylesheet_registry_version = context.stylesheet_registry_version;
        use_drop(move || {
            if let Some(prev) = state.borrow_mut().take() {
                if prev.registered {
                    stylesheet_registry.release((layer, prev.hash));
                    *stylesheet_registry_version.write() += 1;
                }
            }
        });
    }

    class_name
}
