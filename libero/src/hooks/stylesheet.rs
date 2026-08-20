use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    rc::Rc,
};

use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{ClassList, Input},
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
    /// A `'static` `Sx` renders to byte-identical CSS however many components
    /// mount it, so the conversion is done once per static instead of once
    /// per mount. Keyed by address, the way `regex_api`'s `CompiledKey` keys
    /// a pattern - sound because every key comes from a `&'static Sx`, so an
    /// entry's address can never be reused by something else. Unbounded on
    /// purpose: every one of them is a `static` item, so the map reaches a
    /// fixed size.
    static STATIC_SX_CSS: RefCell<HashMap<usize, Stylesheet>> = RefCell::new(HashMap::new());
}

fn build_static(sx: &'static Sx) -> Stylesheet {
    STATIC_SX_CSS.with(|cache| {
        cache
            .borrow_mut()
            .entry(std::ptr::from_ref(sx) as usize)
            .or_insert_with(|| Stylesheet::from(sx))
            .clone()
    })
}

/// `'static` so the address is a stable identity - see [`STATIC_SX_CSS`].
impl CssSource for &'static StaticSx {
    /// The address, not a hash of the entries: a `static` is its own identity,
    /// and hashing the entry tree on every render is the cost being removed.
    /// The *inner* `Sx`'s address, so a static reached through `framework_sx`
    /// and through `sx` shares one cache entry.
    fn identity_hash(&self) -> u64 {
        std::ptr::from_ref::<Sx>(self) as u64
    }

    fn build(self) -> Stylesheet {
        build_static(self)
    }
}

/// A caller's `sx` prop, which carries whether it was built this render or
/// declared as a `static` - see [`SxSource::identity_hash`]. Overlapping impls
/// on `&Sx` and `&'static Sx` would not compile, so the two share one type.
pub(crate) enum SxSource<'a> {
    Owned(&'a Sx),
    Static(&'static Sx),
}

impl CssSource for SxSource<'_> {
    /// Hashed with the variant, so a `Static` address can never be mistaken
    /// for an `Owned` content hash in a slot whose caller alternates.
    fn identity_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        std::mem::discriminant(self).hash(&mut hasher);
        match self {
            // Its entries, since a fresh `Sx` has no identity but its content.
            Self::Owned(sx) => sx.content_hash().hash(&mut hasher),
            Self::Static(sx) => std::ptr::from_ref(*sx).hash(&mut hasher),
        }
        hasher.finish()
    }

    fn build(self) -> Stylesheet {
        match self {
            Self::Owned(sx) => Stylesheet::from(sx),
            Self::Static(sx) => build_static(sx),
        }
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

/// Registers one source into the registry, reusing the previous registration
/// when the source is unchanged. Returns whether the registry was touched; the
/// class name stays in the slot, so composing borrows it instead of cloning.
fn register(
    slot: &mut Option<CssRegistration>,
    source: Option<impl CssSource>,
    layer: CssLayer,
    context: &mut LiberoContext,
) -> bool {
    let identity_hash = source.as_ref().map(CssSource::identity_hash);

    match slot.take() {
        // Same content as last render - the registry is already correct.
        Some(prev) if prev.identity_hash == identity_hash => {
            *slot = Some(prev);
            false
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

            *slot = Some(CssRegistration {
                identity_hash,
                class_name: key.and(class_name),
                key,
            });

            changed
        }
    }
}

/// The registrations one component holds, released when its scope goes.
///
/// One hook slot for the context, the slots and the teardown together. A slot
/// costs ~51 ns per render, and `use_context`/`use_hook`/`use_drop` are three
/// of them - `use_drop` is itself just a `use_hook` holding an `Rc` with a
/// `Drop`, so doing it by hand runs at exactly the same point in teardown.
struct CssRegistrations {
    context: RefCell<LiberoContext>,
    slots: RefCell<Vec<Option<CssRegistration>>>,
}

impl Drop for CssRegistrations {
    fn drop(&mut self) {
        let context = self.context.get_mut();
        let released = self
            .slots
            .get_mut()
            .drain(..)
            .filter_map(|slot| slot.and_then(|prev| prev.key))
            .fold(false, |_, key| {
                context.stylesheet_registry.release(key);
                true
            });

        if released {
            *context.stylesheet_registry_version.write() += 1;
        }
    }
}

fn use_css_registrations(slots: usize) -> Rc<CssRegistrations> {
    use_hook(|| {
        Rc::new(CssRegistrations {
            context: RefCell::new(consume_context::<LiberoContext>()),
            slots: RefCell::new((0..slots).map(|_| None).collect()),
        })
    })
}

/// A signal write during render, sound only because `StyleOutlet` renders
/// after `{children}` and so reads it once every child has registered.
/// Load-bearing ordering.
fn bump_if_changed(changed: bool, context: &mut LiberoContext) {
    if changed {
        *context.stylesheet_registry_version.write() += 1;
    }
}

/// Same as [`use_stylesheet`], but on a caller-chosen layer.
///
/// Takes an `Option` so a caller with nothing to register still calls it:
/// hook slots are positional.
pub(crate) fn use_css(source: Option<impl CssSource>, layer: CssLayer) -> Option<String> {
    let state = use_css_registrations(1);
    let context = &mut *state.context.borrow_mut();
    let slots = &mut *state.slots.borrow_mut();

    let changed = register(&mut slots[0], source, layer, context);
    bump_if_changed(changed, context);

    class_name(&slots[0]).map(str::to_string)
}

fn class_name(slot: &Option<CssRegistration>) -> Option<&str> {
    slot.as_ref()
        .and_then(|registration| registration.class_name.as_deref())
}

/// The three registrations every `Box`-shaped component makes, behind one hook
/// slot, composed straight into the element's `class` value.
///
/// Three `use_css` calls cost ~180 ns each of which only ~28 ns is real work -
/// the rest is per-hook plumbing. Sharing it is worth ~360 ns per component.
pub(crate) fn use_box_css(
    class: &Input<ClassList>,
    focus: &'static StaticSx,
    framework: Option<&'static StaticSx>,
    sx: Option<SxSource<'_>>,
) -> String {
    let state = use_css_registrations(3);
    let context = &mut *state.context.borrow_mut();
    let slots = &mut *state.slots.borrow_mut();
    let (head, sx_slot) = slots.split_at_mut(2);
    let (focus_slot, framework_slot) = head.split_at_mut(1);

    let focus_changed = register(
        &mut focus_slot[0],
        Some(focus),
        CssLayer::Framework,
        context,
    );
    let framework_changed = register(
        &mut framework_slot[0],
        framework,
        CssLayer::Framework,
        context,
    );
    let sx_changed = register(&mut sx_slot[0], sx, CssLayer::UserStatic, context);
    bump_if_changed(focus_changed || framework_changed || sx_changed, context);

    // Composed straight into the attribute value. Going through `ClassList`
    // meant a `Vec`, a `String` per registration and a joining pass - five
    // allocations for what one `String` holds. The order is the caller's
    // classes, then framework, focus, static.
    let caller = class.as_ref();
    let names = || {
        caller
            .into_iter()
            .flat_map(ClassList::iter)
            .chain(class_name(&framework_slot[0]))
            .chain(class_name(&focus_slot[0]))
            .chain(class_name(&sx_slot[0]))
    };

    // Sized first: growing from empty reallocated three times for a component
    // with a `framework_sx`, which cost more than the `ClassList` it replaced.
    let capacity = names().map(|name| name.len() + 1).sum::<usize>();
    let mut composed = String::with_capacity(capacity);
    for name in names() {
        if !composed.is_empty() {
            composed.push(' ');
        }
        composed.push_str(name);
    }
    composed
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

    /// What a caller's `sx: &STATIC` buys: an unchanged render re-derives an
    /// identity without touching the entries, and the CSS is built once.
    #[test]
    fn a_static_sx_source_identifies_by_address() {
        let first = SxSource::Static(&PADDING);
        let second = SxSource::Static(&PADDING);
        let other = SxSource::Static(&SAME_PADDING);

        assert_eq!(first.identity_hash(), second.identity_hash());
        assert_ne!(first.identity_hash(), other.identity_hash());
    }

    /// Same CSS either way - the variant only decides how the identity is
    /// derived, never what gets registered.
    #[test]
    fn a_static_and_an_owned_sx_source_build_the_same_sheet() {
        let owned = sx().padding("lg");

        assert_eq!(
            SxSource::Static(&PADDING).build(),
            SxSource::Owned(&owned).build()
        );
    }

    /// Equal content, different variant: the discriminant is hashed in, so
    /// alternating between the two in one hook slot can't look unchanged.
    #[test]
    fn a_static_and_an_owned_sx_source_do_not_share_an_identity() {
        let owned = sx().padding("lg");

        assert_ne!(
            SxSource::Static(&PADDING).identity_hash(),
            SxSource::Owned(&owned).identity_hash()
        );
    }

    /// One cache entry per static, whichever prop reached it - `framework_sx`
    /// takes the `&'static StaticSx` impl, `sx` the `SxSource` one.
    #[test]
    fn a_static_reached_through_either_prop_shares_one_cache_entry() {
        let framework: &'static StaticSx = &COLOR;
        framework.build();
        SxSource::Static(&COLOR).build();

        let entries = STATIC_SX_CSS.with(|cache| {
            cache
                .borrow()
                .keys()
                .filter(|key| **key == std::ptr::from_ref::<Sx>(&COLOR) as usize)
                .count()
        });

        assert_eq!(entries, 1);
    }
}
