//! The registration plumbing under `use_stylesheet`: each source keeps its
//! registry entry across renders and gives it back when its scope goes.

use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    rc::Rc,
};

use dioxus::prelude::*;

use crate::{
    CssLayer,
    context::{LiberoContext, SheetRank, StylesheetKey},
    css::Stylesheet,
    sx::{ClassList, Input, StaticSx, Sx},
};

/// A value [`use_css`] can register. The cheap `identity_hash` apart from the
/// CSS build, so an unchanged re-render formats nothing.
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
    /// A `'static` `Sx`'s CSS, built once. Keyed by address, never reused by a
    /// `static`; unbounded, since the statics are a fixed set.
    static STATIC_SX_CSS: RefCell<HashMap<usize, Stylesheet>> = RefCell::new(HashMap::new());
}

fn build_static(sx: &'static Sx) -> Stylesheet {
    with_static(sx, Stylesheet::clone)
}

fn with_static<R>(sx: &'static Sx, f: impl FnOnce(&Stylesheet) -> R) -> R {
    STATIC_SX_CSS.with(|cache| {
        f(cache
            .borrow_mut()
            .entry(std::ptr::from_ref(sx) as usize)
            .or_insert_with(|| Stylesheet::from(sx)))
    })
}

/// `'static` so the address is a stable identity - see [`STATIC_SX_CSS`].
impl CssSource for &'static StaticSx {
    /// The *inner* `Sx`'s address, not an entry hash: so a static reached
    /// through `framework_sx` and through `sx` shares one cache entry.
    fn identity_hash(&self) -> u64 {
        std::ptr::from_ref::<Sx>(self) as u64
    }

    fn build(self) -> Stylesheet {
        build_static(self)
    }
}

/// A caller's `sx` prop, built this render or a `static`. One type, since
/// impls on both `&Sx` and `&'static Sx` would overlap.
#[derive(Clone, Copy)]
pub(crate) enum SxSource<'a> {
    Owned(&'a Sx),
    Static(&'static Sx),
}

impl<'a> SxSource<'a> {
    pub(crate) fn sx(self) -> &'a Sx {
        match self {
            Self::Owned(sx) | Self::Static(sx) => sx,
        }
    }
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

/// Registers one source, reusing the previous registration when unchanged.
/// Returns whether the registry was touched.
fn register(
    slot: &mut Option<CssRegistration>,
    source: Option<impl CssSource>,
    layer: CssLayer,
    rank: SheetRank,
    context: &mut LiberoContext,
) -> bool {
    let identity_hash = source.as_ref().map(CssSource::identity_hash);

    match slot.take() {
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
                    Some(context.stylesheet_registry.acquire(stylesheet, layer, rank))
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

/// The registrations one component holds, released when its scope goes. One
/// hook slot (~51 ns per render each) for context, slots and teardown.
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

        if released && context.stylesheet_registry.take_outlet_change() {
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
/// after `{children}`: load-bearing ordering. Only when the outlet's `<style>` list moved.
fn bump_if_changed(changed: bool, context: &mut LiberoContext) {
    if changed && context.stylesheet_registry.take_outlet_change() {
        *context.stylesheet_registry_version.write() += 1;
    }
}

/// `use_stylesheet` on a caller-chosen layer. An `Option`, so a caller with
/// nothing to register still takes its hook slot.
pub(crate) fn use_css(source: Option<impl CssSource>, layer: CssLayer) -> Option<String> {
    let state = use_css_registrations(1);
    let context = &mut *state.context.borrow_mut();
    let slots = &mut *state.slots.borrow_mut();

    let changed = register(&mut slots[0], source, layer, SheetRank::Component, context);
    bump_if_changed(changed, context);

    class_name(&slots[0]).map(str::to_string)
}

fn class_name(slot: &Option<CssRegistration>) -> Option<&str> {
    slot.as_ref()
        .and_then(|registration| registration.class_name.as_deref())
}

/// A [`keyed`](Stylesheet::keyed) sheet named by its `data-state` word, built only
/// when the word changes: `hidden_from`'s rule.
#[derive(Clone, Copy)]
pub(crate) struct WordSheet {
    pub word: &'static str,
    pub build: fn(&'static str) -> Stylesheet,
}

impl CssSource for WordSheet {
    fn identity_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.word.hash(&mut hasher);
        hasher.finish()
    }

    fn build(self) -> Stylesheet {
        (self.build)(self.word)
    }
}

/// The registrations every `Box`-shaped component makes, behind one hook slot
/// (~360 ns saved per component), composed into the `class` value. The `words`
/// slots stay empty unless the box uses `hidden_from` / `visible_from`.
pub(crate) fn use_box_css(
    class: &Input<ClassList>,
    focus: Option<&'static StaticSx>,
    framework: Option<&'static StaticSx>,
    sx: Option<SxSource<'_>>,
    words: [Option<WordSheet>; 2],
) -> String {
    let state = use_css_registrations(5);
    let context = &mut *state.context.borrow_mut();
    let slots = &mut *state.slots.borrow_mut();
    let (slots, word_slots) = slots.split_at_mut(3);
    let words_changed = word_slots
        .iter_mut()
        .zip(words)
        .fold(false, |changed, (slot, word)| {
            register(
                slot,
                word,
                CssLayer::Framework,
                SheetRank::Component,
                context,
            ) || changed
        });
    let (head, sx_slot) = slots.split_at_mut(2);
    let (focus_slot, framework_slot) = head.split_at_mut(1);

    let focus_changed = register(
        &mut focus_slot[0],
        focus,
        CssLayer::Framework,
        SheetRank::Default,
        context,
    );
    let framework_changed = register(
        &mut framework_slot[0],
        framework,
        CssLayer::Framework,
        SheetRank::Component,
        context,
    );
    let sx_changed = register(
        &mut sx_slot[0],
        sx,
        CssLayer::UserStatic,
        SheetRank::Component,
        context,
    );
    bump_if_changed(
        focus_changed || framework_changed || sx_changed || words_changed,
        context,
    );

    // One `String`, not a `ClassList` (five allocations). Order: the caller's
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

/// One range copy for [`use_range_css`]: `block` under `word`, within the viewport range
/// `query`. The block builds only when the word is new to the instance.
pub(crate) struct RangeCopy<'a> {
    word: &'static str,
    query: String,
    block: Box<dyn FnOnce() -> Sx + 'a>,
}

impl RangeCopy<'_> {
    /// The run word, as `md-:size-xl`: lets a caller drop a run whose block is empty.
    pub(crate) fn word(&self) -> &'static str {
        self.word
    }
}

impl<T: Clone + PartialEq> crate::tokens::Responsive<T> {
    /// The copies a responsive axis value needs, one per run; none for a plain value.
    pub(crate) fn range_copies<'a>(
        &self,
        token: impl Fn(T) -> &'static str,
        block: impl Fn(T) -> Sx + Clone + 'a,
    ) -> Vec<RangeCopy<'a>>
    where
        T: 'a,
    {
        if self.is_plain() {
            return Vec::new();
        }
        self.runs()
            .into_iter()
            .map(|(from, until, value)| {
                let block = block.clone();
                RangeCopy {
                    word: crate::tokens::range_token(from, until, token(value.clone())),
                    query: crate::tokens::range_query(from, until),
                    block: Box::new(move || block(value)),
                }
            })
            .collect()
    }
}

/// The range copies one instance holds, released when its scope goes.
struct RangeRegistrations {
    context: RefCell<LiberoContext>,
    held: RefCell<Vec<(&'static str, Option<StylesheetKey>)>>,
}

impl Drop for RangeRegistrations {
    fn drop(&mut self) {
        let context = self.context.get_mut();
        let held = std::mem::take(self.held.get_mut());
        for key in held.iter().filter_map(|(_, key)| *key) {
            context.stylesheet_registry.release(key);
        }
        if !held.is_empty() && context.stylesheet_registry.take_outlet_change() {
            *context.stylesheet_registry_version.write() += 1;
        }
    }
}

/// Registers a responsive instance's range copies of `base`'s axis blocks, rooted at `base`'s
/// class: `@media <run>{.<class>[data-state~="<word>"]{..}}`, the plain word block's specificity,
/// shared by every instance using the word. A plain instance registers nothing. One hook slot.
pub(crate) fn use_range_css(base: &'static StaticSx, copies: Vec<RangeCopy<'_>>) {
    let state = use_hook(|| {
        Rc::new(RangeRegistrations {
            context: RefCell::new(consume_context::<LiberoContext>()),
            held: RefCell::new(Vec::new()),
        })
    });
    let held = &mut *state.held.borrow_mut();
    if held
        .iter()
        .map(|(word, _)| *word)
        .eq(copies.iter().map(|copy| copy.word))
    {
        return;
    }
    let context = &mut *state.context.borrow_mut();
    let Some(class_name) = with_static(base, |sheet| sheet.class_name().map(str::to_string)) else {
        return;
    };
    // Acquire before release, so a copy kept across the change never leaves.
    let next: Vec<_> = copies
        .into_iter()
        .map(|copy| {
            let sx =
                crate::sx::sx().media(copy.query, crate::sx::sx().when(copy.word, (copy.block)()));
            let sheet = Stylesheet::rooted_at(&sx, &class_name);
            // An empty block (Tabs' `top`) is a word with no rule.
            let key = (!sheet.as_str().is_empty()).then(|| {
                context
                    .stylesheet_registry
                    .acquire(sheet, CssLayer::Framework, SheetRank::Range)
            });
            (copy.word, key)
        })
        .collect();
    for key in std::mem::replace(held, next)
        .into_iter()
        .filter_map(|(_, key)| key)
    {
        context.stylesheet_registry.release(key);
    }
    bump_if_changed(true, context);
}
