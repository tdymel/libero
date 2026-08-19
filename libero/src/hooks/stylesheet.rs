use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    CssLayer,
    context::{LiberoContext, StylesheetKey},
    css::Stylesheet,
    sx::{StaticSx, Sx},
};

/// Registers anything that converts into a [`Stylesheet`] and returns its
/// class name, on the `UserCustom` layer so it always wins the cascade. Raw
/// `&str`/`String` CSS has no single selector, so it returns no class name.
pub fn use_stylesheet(stylesheet: impl Into<Stylesheet>) -> Option<String> {
    let stylesheet: Stylesheet = stylesheet.into();
    use_css(Some(stylesheet), CssLayer::UserCustom)
}

/// A value [`use_css`] can register. Separates a cheap identity check
/// (`identity_hash`) from the expensive CSS text build (`build`), so a
/// render whose `Sx` is unchanged from the last one never has to walk its
/// entry tree or format any CSS - `Sx::hash` is a cheap structural hash over
/// the already-in-memory entries, unlike `Stylesheet::hash`, which needs the
/// rendered text to exist first. That is also why the previous render's
/// class name is cached in `CssRegistration` rather than recomputed: getting
/// it from an `Sx` means rendering the CSS.
pub(crate) trait CssSource {
    fn identity_hash(&self) -> u64;
    fn build(self) -> Stylesheet;
}

impl CssSource for &Sx {
    fn identity_hash(&self) -> u64 {
        self.content_hash()
    }

    fn build(self) -> Stylesheet {
        Stylesheet::from(self)
    }
}

impl CssSource for &StaticSx {
    fn identity_hash(&self) -> u64 {
        self.content_hash()
    }

    fn build(self) -> Stylesheet {
        Stylesheet::from(self)
    }
}

impl CssSource for Stylesheet {
    fn identity_hash(&self) -> u64 {
        self.hash()
    }

    fn build(self) -> Stylesheet {
        self
    }
}

struct CssRegistration {
    /// The source's own hash, to detect an unchanged re-render - `None`
    /// when there was no source at all, which never compares equal to a
    /// source that is present.
    identity_hash: Option<u64>,
    /// The registry's key for this registration - `None` when the source
    /// built empty CSS and was never registered. Not derived from
    /// `identity_hash`: that one is structural, the registry keys by
    /// rendered-CSS hash.
    key: Option<StylesheetKey>,
    /// The class name handed out last render, cached so an unchanged
    /// re-render doesn't have to rebuild the CSS to recompute it.
    class_name: Option<String>,
}

/// Same as [`use_stylesheet`], but on a caller-chosen layer.
///
/// Takes an `Option` so a caller with nothing to register still calls it
/// unconditionally: this is three hooks, and Dioxus hook slots are
/// positional, so branching around the call would hand a later hook the
/// slot an earlier one used as soon as the condition flips.
pub(crate) fn use_css(source: Option<impl CssSource>, layer: CssLayer) -> Option<String> {
    let identity_hash = source.as_ref().map(CssSource::identity_hash);
    let mut context = use_context::<LiberoContext>();
    let state = use_hook(|| Rc::new(RefCell::new(None::<CssRegistration>)));

    let class_name = {
        let mut state_ref = state.borrow_mut();

        match state_ref.take() {
            // Same `Sx` as last render (by content, not identity) - the
            // registry entry (or lack of one) is already correct, so skip
            // rebuilding the CSS text entirely.
            Some(prev) if prev.identity_hash == identity_hash => {
                let class_name = prev.class_name.clone();
                *state_ref = Some(prev);
                class_name
            }
            prev => {
                let stylesheet = source.map(CssSource::build);
                let class_name = stylesheet
                    .as_ref()
                    .and_then(|stylesheet| stylesheet.class_name())
                    .map(str::to_string);
                let mut changed = false;

                if let Some(prev_key) = prev.and_then(|prev| prev.key) {
                    context.stylesheet_registry.release(prev_key);
                    changed = true;
                }

                let key = match stylesheet {
                    Some(stylesheet) if !stylesheet.as_str().is_empty() => {
                        changed = true;
                        Some(context.stylesheet_registry.acquire(stylesheet, layer))
                    }
                    _ => None,
                };

                let class_name = key.and(class_name);
                *state_ref = Some(CssRegistration {
                    identity_hash,
                    key,
                    class_name: class_name.clone(),
                });

                if changed {
                    *context.stylesheet_registry_version.write() += 1;
                }

                class_name
            }
        }
    };

    {
        let state = state.clone();
        let stylesheet_registry = context.stylesheet_registry.clone();
        let mut stylesheet_registry_version = context.stylesheet_registry_version;
        use_drop(move || {
            if let Some(key) = state.borrow_mut().take().and_then(|prev| prev.key) {
                stylesheet_registry.release(key);
                *stylesheet_registry_version.write() += 1;
            }
        });
    }

    class_name
}
