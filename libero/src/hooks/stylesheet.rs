use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

/// A value [`use_css`] can register. Splits the cheap identity check from the
/// expensive CSS build, so an unchanged re-render formats nothing:
/// `identity_hash` is structural over in-memory entries, where a
/// `Stylesheet` hash would need the rendered text first.
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

thread_local! {
    /// A `StaticSx` renders to byte-identical CSS however many components
    /// mount it, so the conversion is done once per static instead of once
    /// per mount. Keyed by address, the way `regex_api`'s `CompiledKey` keys
    /// a pattern - sound because the impl below is on `&'static StaticSx`, so
    /// an entry's address can never be reused by something else. Unbounded on
    /// purpose: every `StaticSx` is a `static` item, so the map reaches a
    /// fixed size.
    static STATIC_SX_CSS: RefCell<HashMap<usize, Stylesheet>> = RefCell::new(HashMap::new());
}

/// `'static` so the address is a stable identity - see [`STATIC_SX_CSS`].
impl CssSource for &'static StaticSx {
    /// The address, not a hash of the entries: a `static` is its own identity,
    /// and hashing the entry tree on every render is the cost being removed.
    fn identity_hash(&self) -> u64 {
        std::ptr::from_ref(*self) as u64
    }

    fn build(self) -> Stylesheet {
        STATIC_SX_CSS.with(|cache| {
            cache
                .borrow_mut()
                .entry(std::ptr::from_ref(self) as usize)
                .or_insert_with(|| Stylesheet::from(self))
                .clone()
        })
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
    /// `None` means no source, which never equals a source that is present.
    identity_hash: Option<u64>,
    /// `None` when the source built empty CSS. Unrelated to `identity_hash` -
    /// the registry keys by rendered-CSS hash, not structure.
    key: Option<StylesheetKey>,
    /// Cached, because recomputing it from an `Sx` means rendering the CSS.
    class_name: Option<String>,
}

/// Same as [`use_stylesheet`], but on a caller-chosen layer.
///
/// Takes an `Option` so a caller with nothing to register still calls it:
/// hook slots are positional, and this is three of them.
pub(crate) fn use_css(source: Option<impl CssSource>, layer: CssLayer) -> Option<String> {
    let identity_hash = source.as_ref().map(CssSource::identity_hash);
    let mut context = use_context::<LiberoContext>();
    let state = use_hook(|| Rc::new(RefCell::new(None::<CssRegistration>)));

    let class_name = {
        let mut state_ref = state.borrow_mut();

        match state_ref.take() {
            // Same content as last render - the registry is already correct.
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

                // A signal write during render, sound only because
                // `StyleOutlet` renders after `{children}` and so reads it
                // once every child has registered. Load-bearing ordering.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sx::sx;

    static PADDING: StaticSx = StaticSx::new(|| sx().padding("lg"));
    static SAME_PADDING: StaticSx = StaticSx::new(|| sx().padding("lg"));
    static COLOR: StaticSx = StaticSx::new(|| sx().color("red"));

    #[test]
    fn a_memoized_build_matches_an_unmemoized_one() {
        let memoized: &'static StaticSx = &PADDING;

        assert_eq!(memoized.build(), Stylesheet::from(&PADDING));
    }

    #[test]
    fn a_second_build_of_the_same_static_is_the_same_sheet() {
        let first: &'static StaticSx = &COLOR;
        let second: &'static StaticSx = &COLOR;

        assert_eq!(first.build(), second.build());
    }

    /// The cache is keyed by address, so these are two entries - but the CSS
    /// is keyed by content, so they still collapse to one class and one
    /// registry entry downstream.
    #[test]
    fn two_statics_with_equal_content_keep_separate_identities_and_one_class() {
        let padding: &'static StaticSx = &PADDING;
        let same: &'static StaticSx = &SAME_PADDING;

        assert_ne!(padding.identity_hash(), same.identity_hash());
        assert_eq!(padding.build().class_name(), same.build().class_name());
    }

    #[test]
    fn different_statics_do_not_share_a_cache_entry() {
        let padding: &'static StaticSx = &PADDING;
        let color: &'static StaticSx = &COLOR;

        assert_ne!(padding.build().as_str(), color.build().as_str());
    }
}
