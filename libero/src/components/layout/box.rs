use std::marker::PhantomData;

use dioxus::core::{AttributeValue, IntoAttributeValue, ListenerCallback};
use dioxus::html::{EventHandlerValue, PlatformEventData};
use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States, Variables,
        common::{
            IntoChildren, StyleAttributes, attr, base_props, render_polymorphic,
            use_style_attributes,
        },
    },
    sx::{StaticSx, Sx},
};

base_props! {
    // Covers the non-global attributes callers set on a `Box`. Names shared
    // by two of these tags are ambiguous at the call site, hence the explicit
    // `alt`/`r#type` fields - an inherent builder method wins over both.
    extends(img, a, button);
    pub struct BoxProps {
        /// Per-instance CSS custom properties on the `style` attribute, so
        /// `sx` can reference a varying value without a class per value.
        #[props(default, into)]
        variables: Input<Variables>,
        /// Which element to render as - `div` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Base styles of a component built on `Box`, on its own CSS layer.
        #[props(default)]
        framework_sx: Option<&'static StaticSx>,
        /// Merged after `variables`, not overwritten by it.
        #[props(default)]
        style: Option<String>,
        #[props(default)]
        alt: Option<String>,
        #[props(default)]
        r#type: Option<String>,
        children: Element,
    }
}

/// A `Box` element built inline, in the caller's own scope.
///
/// Composing `rsx! { Box { .. } }` costs ~950 ns per instance for a scope that
/// emits no markup of its own. Building the same element here costs nothing
/// extra.
///
/// ```ignore
/// use_box()
///     .framework_sx(&TEXT_BASE_SX)
///     .class(&props.class)
///     .sx(&props.sx)
///     .states(&states)
///     .prepare()
///     .render(HtmlTag::P, props.attributes, props.children)
/// ```
///
/// A component with more than one return shape calls `.prepare()` once at the
/// top and renders from the result in each branch - see `Button`.
///
/// `use_` because [`BoxBuilder::prepare`] calls [`use_style_attributes`]. The
/// whole chain is one expression, so a conditional `use_box()` is a
/// conditional hook: build it in the component body and return it - never
/// inside an `if`, a `match` arm, or after an early `return`. The prefix is
/// what the hook-order greps match on.
pub(crate) fn use_box<'a>() -> BoxBuilder<'a> {
    BoxBuilder::default()
}

/// Set only what the component needs; everything else stays defaulted.
///
/// Holds **references** to the styling props, so a component that also has a
/// non-`Box` return path still owns them afterwards - see `Button`.
pub(crate) struct BoxBuilder<'a> {
    framework_sx: Option<&'static StaticSx>,
    class: Option<&'a Input<ClassList>>,
    sx: Option<&'a Input<Sx>>,
    states: Option<&'a Input<States>>,
    variables: Option<&'a Input<Variables>>,
    style: Option<String>,
    focus_ring: bool,
}

impl Default for BoxBuilder<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            framework_sx: None,
            class: None,
            sx: None,
            states: None,
            variables: None,
            style: None,
            focus_ring: true,
        }
    }
}

impl<'a> BoxBuilder<'a> {
    #[inline]
    pub fn framework_sx(mut self, framework_sx: &'static StaticSx) -> Self {
        self.framework_sx = Some(framework_sx);
        self
    }

    #[inline]
    pub fn class(mut self, class: &'a Input<ClassList>) -> Self {
        self.class = Some(class);
        self
    }

    #[inline]
    pub fn sx(mut self, sx: &'a Input<Sx>) -> Self {
        self.sx = Some(sx);
        self
    }

    #[inline]
    pub fn states(mut self, states: &'a Input<States>) -> Self {
        self.states = Some(states);
        self
    }

    #[inline]
    pub fn variables(mut self, variables: &'a Input<Variables>) -> Self {
        self.variables = Some(variables);
        self
    }

    /// Drops the shared `:focus-visible` ring from this element, for one that
    /// delegates its ring to an ancestor - a field's control, whose frame draws
    /// the ring for it. Two elements both carrying it draw two rings.
    #[inline]
    pub fn focus_ring(mut self, focus_ring: bool) -> Self {
        self.focus_ring = focus_ring;
        self
    }

    /// Raw `style` declarations, appended after `variables`. For a component
    /// that caches its own custom properties as a rendered string.
    #[inline]
    pub fn style(mut self, style: Option<String>) -> Self {
        self.style = style;
        self
    }

    /// Resolves the styling. **This is the hook** - see [`use_box`].
    ///
    /// Split from [`BoxStyle::render`] so a component with more than one
    /// return shape can run it once, above the branch, and render from the
    /// result on every path.
    pub fn prepare(self) -> BoxStyle {
        const NONE_CLASS: Input<ClassList> = Input::None;
        const NONE_SX: Input<Sx> = Input::None;
        const NONE_STATES: Input<States> = Input::None;
        const NONE_VARIABLES: Input<Variables> = Input::None;

        BoxStyle {
            own: Vec::new(),
            fallback: Vec::new(),
            style: use_style_attributes(
                self.class.unwrap_or(&NONE_CLASS),
                self.framework_sx,
                self.sx.unwrap_or(&NONE_SX),
                self.states.unwrap_or(&NONE_STATES),
                self.variables.unwrap_or(&NONE_VARIABLES),
                self.style,
                self.focus_ring,
            ),
        }
    }
}

/// A [`BoxStyle`] from styling a component already resolved itself with
/// `use_style_attributes` - so it can hand the pieces to something that is not
/// an element on one path (`InternalAnchor`'s `Link`) and render the element
/// on the other, without resolving twice.
pub(crate) fn box_style(style: StyleAttributes) -> BoxStyle {
    BoxStyle {
        own: Vec::new(),
        fallback: Vec::new(),
        style,
    }
}

/// A handler for [`BoxStyle::event`]: one, or an `Option` of one. A `None`
/// pushes no attribute, for the reason a `None` [`BoxStyle::attr`] does not.
///
/// Two impls of one trait need two distinct `Marker`s, or they overlap - the
/// same trick `EventHandlerValue` itself plays. Neither is ever named.
pub(crate) trait EventValue<T, Marker> {
    fn into_listener(self) -> Option<ListenerCallback<PlatformEventData>>;
}

#[doc(hidden)]
pub(crate) struct Given<Marker>(PhantomData<Marker>);
#[doc(hidden)]
pub(crate) struct Maybe<Marker>(PhantomData<Marker>);

impl<T, Marker, H> EventValue<T, Given<Marker>> for H
where
    T: for<'a> From<&'a PlatformEventData> + 'static,
    H: EventHandlerValue<T, Marker>,
{
    fn into_listener(self) -> Option<ListenerCallback<PlatformEventData>> {
        Some(self.into_platform_listener())
    }
}

impl<T, Marker, H> EventValue<T, Maybe<Marker>> for Option<H>
where
    T: for<'a> From<&'a PlatformEventData> + 'static,
    H: EventHandlerValue<T, Marker>,
{
    fn into_listener(self) -> Option<ListenerCallback<PlatformEventData>> {
        self.map(EventHandlerValue::into_platform_listener)
    }
}

/// The attribute, unless its value renders nothing: a `false` boolean or a
/// `None` costs ~135 ns to diff and produces no markup either way. Dropping it
/// is safe - a shrinking attribute list still clears what went away, which
/// `tests/attributes.rs` pins down.
fn meaningful<T>(name: &'static str, value: impl IntoAttributeValue<T>) -> Option<Attribute> {
    let attribute = attr(name, value);
    match attribute.value {
        AttributeValue::None | AttributeValue::Bool(false) => None,
        _ => Some(attribute),
    }
}

/// Resolved styling, ready to render as any element. Pure - no hooks - so it
/// can be used in a branch, or not at all.
///
/// `Clone` because a component can render the same resolved styling as several
/// elements - `PinField`'s cells are one prepared frame, cloned per cell. The
/// clone is a class, a `data-state` and the attribute list; no hook runs again.
#[derive(Clone)]
pub(crate) struct BoxStyle {
    style: StyleAttributes,
    /// The component's own attributes, rendered **after** the caller's - so on
    /// a duplicate name the component wins, not the caller (`id` is the one
    /// exception, deduped in `render`).
    own: Vec<Attribute>,
    /// Attributes the component supplies only where the caller supplied none -
    /// see [`BoxStyle::attr_default`]. Kept apart from `own` because whether
    /// they apply is not known until `render` receives the caller's.
    fallback: Vec<Attribute>,
}

impl BoxStyle {
    /// An attribute the component sets itself, e.g. `type="button"`.
    ///
    /// A `false` boolean or a `None` is **not** pushed: it renders nothing
    /// either way, but an attribute in the list still costs ~135 ns to diff.
    /// Dropping it is safe - when the list shrinks, dioxus emits a `None` for
    /// the attribute that went away, so a `disabled` button that becomes
    /// enabled still loses the attribute in the DOM (`tests/attributes.rs`).
    pub fn attr<T>(mut self, name: &'static str, value: impl IntoAttributeValue<T>) -> Self {
        if let Some(attribute) = meaningful(name, value) {
            self.own.push(attribute);
        }
        self
    }

    /// An attribute the component supplies only where the caller supplied
    /// none - a default, not an override.
    ///
    /// [`BoxStyle::attr`] would win the duplicate and silently replace what
    /// the caller asked for, which is exactly the bug `Button`'s hardcoded
    /// `type="button"` was.
    pub fn attr_default<T>(
        mut self,
        name: &'static str,
        value: impl IntoAttributeValue<T>,
    ) -> Self {
        if let Some(attribute) = meaningful(name, value) {
            self.fallback.push(attribute);
        }
        self
    }

    /// Wires an [`ElementHandle`](crate::hooks::ElementHandle) to this
    /// element, so the component can measure, focus or query it.
    pub fn element(self, handle: &crate::hooks::ElementHandle) -> Self {
        self.event("onmounted", handle.mount())
    }

    /// An event handler the component sets itself. Hand-building the
    /// `Attribute` is the one thing `rsx!` does that a plain call cannot.
    ///
    /// The handler has to go through `EventHandlerValue`: a renderer delivers
    /// `PlatformEventData`, and only that conversion turns it into `T`. A bare
    /// `AttributeValue::listener::<T>` panics on the first real event.
    pub fn event<T, Marker>(
        mut self,
        name: &'static str,
        handler: impl EventValue<T, Marker>,
    ) -> Self
    where
        T: for<'a> From<&'a PlatformEventData> + 'static,
    {
        if let Some(listener) = handler.into_listener() {
            self.own.push(Attribute::new(
                name,
                AttributeValue::Listener(listener.erase()),
                None,
                false,
            ));
        }
        self
    }

    pub fn render(
        self,
        component: HtmlTag,
        attributes: Vec<Attribute>,
        children: impl IntoChildren,
    ) -> Element {
        // A component that sets its own root id and also spreads `attributes`
        // sends two `id`s here. Browsers keep the first; `use_root_id` makes
        // both the caller's, so dropping the rest is enough.
        let mut seen_id = false;
        let attributes = attributes
            .into_iter()
            .chain(self.own)
            .filter(move |attribute| {
                let duplicate = attribute.name == "id" && seen_id;
                seen_id |= attribute.name == "id";
                !duplicate
            })
            .collect::<Vec<_>>();

        let mut attributes = attributes;
        for attribute in self.fallback {
            if !attributes.iter().any(|set| set.name == attribute.name) {
                attributes.push(attribute);
            }
        }

        render_polymorphic(
            component,
            self.style.class,
            self.style.data_state,
            self.style.style,
            attributes,
            children.into_children(),
        )
    }
}

#[component]
pub fn Box(props: BoxProps) -> Element {
    let mut style = BoxBuilder {
        framework_sx: props.framework_sx,
        class: Some(&props.class),
        sx: Some(&props.sx),
        states: Some(&props.states),
        variables: Some(&props.variables),
        style: props.style,
        focus_ring: true,
    }
    .prepare();

    if let Some(alt) = props.alt {
        style = style.attr("alt", alt);
    }
    if let Some(r#type) = props.r#type {
        style = style.attr("type", r#type);
    }

    style.render(
        props.component.copied_or_default(),
        props.attributes,
        props.children,
    )
}
