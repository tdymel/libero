use std::marker::PhantomData;

use dioxus::core::{AttributeValue, IntoAttributeValue, ListenerCallback};
use dioxus::html::{EventHandlerValue, PlatformEventData};
use dioxus::prelude::*;

use crate::{
    components::common::{
        ClassList, HtmlTag, Input, IntoChildren, Part, Parts, States, StyleAttributes, Variables,
        attr, base_props, parts_source, render_polymorphic, sx_source, use_style_attributes,
        with_parts,
    },
    hooks::SxSource,
    sx::{StaticSx, Sx},
};

base_props! {
    // Names shared by two of these tags are ambiguous, hence the explicit `alt`/`r#type`.
    extends(img, a, button);
    pub struct BoxProps {
        /// Per-instance CSS custom properties, for `sx` to reference.
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

/// A `Box` element built inline, saving the ~950 ns scope of `rsx! { Box {..} }`.
/// A hook ([`BoxBuilder::prepare`]): never call it inside a branch or after an early return.
pub(crate) fn use_box<'a>() -> BoxBuilder<'a> {
    BoxBuilder::default()
}

/// Holds references to the styling props, so a component with a non-`Box`
/// return path still owns them afterwards - see `Button`.
pub(crate) struct BoxBuilder<'a> {
    framework_sx: Option<&'static StaticSx>,
    class: Option<&'a Input<ClassList>>,
    sx: Option<&'a Input<Sx>>,
    parts: Option<SxSource<'a>>,
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
            parts: None,
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

    /// The `parts` prop, merged under `sx`.
    #[inline]
    pub fn parts<P: Part>(mut self, parts: &'a Input<Parts<P>>) -> Self {
        self.parts = parts_source(parts);
        self
    }

    /// [`parts`](Self::parts), already resolved by a builder that holds it.
    #[inline]
    pub fn parts_source(mut self, parts: Option<SxSource<'a>>) -> Self {
        self.parts = parts;
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

    /// Drops the shared focus ring, for an element whose ancestor draws it
    /// (a field's control and its frame); both would draw two rings.
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

    /// Resolves the styling; the hook. Split from [`BoxStyle::render`] so a
    /// component with several return shapes runs it once, above the branch.
    pub fn prepare(self) -> BoxStyle {
        const NONE_CLASS: Input<ClassList> = Input::None;
        const NONE_STATES: Input<States> = Input::None;
        const NONE_VARIABLES: Input<Variables> = Input::None;

        let mut merged = None;
        let sx = with_parts(self.parts, self.sx.and_then(sx_source), &mut merged);
        BoxStyle {
            own: Vec::new(),
            fallback: Vec::new(),
            style: use_style_attributes(
                self.class.unwrap_or(&NONE_CLASS),
                self.framework_sx,
                sx,
                self.states.unwrap_or(&NONE_STATES),
                self.variables.unwrap_or(&NONE_VARIABLES),
                self.style,
                self.focus_ring,
            ),
        }
    }
}

/// A [`BoxStyle`] from already resolved styling, for a component with a
/// non-element path (`InternalAnchor`'s `Link`), without resolving twice.
pub(crate) fn box_style(style: StyleAttributes) -> BoxStyle {
    BoxStyle {
        own: Vec::new(),
        fallback: Vec::new(),
        style,
    }
}

/// A handler for [`BoxStyle::event`], or an `Option` of one. The two `Marker`s
/// keep the impls from overlapping, as in `EventHandlerValue`.
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

/// The attribute, unless `false` or `None`: those cost ~135 ns to diff for no
/// markup. A shrinking list still clears them (`tests/all/attributes.rs`).
fn meaningful<T>(name: &'static str, value: impl IntoAttributeValue<T>) -> Option<Attribute> {
    let attribute = attr(name, value);
    match attribute.value {
        AttributeValue::None | AttributeValue::Bool(false) => None,
        _ => Some(attribute),
    }
}

/// Resolved styling, ready to render as any element. No hooks, so usable in a
/// branch; `Clone` so one frame renders several elements (`PinField`'s cells).
#[derive(Clone, PartialEq)]
pub(crate) struct BoxStyle {
    style: StyleAttributes,
    /// Rendered after the caller's, so the component wins a duplicate (not `id`).
    own: Vec<Attribute>,
    /// Applied in `render` only where the caller set none.
    fallback: Vec<Attribute>,
}

impl BoxStyle {
    /// The resolved styling alone, for a path that renders something other
    /// than an element (`render_anchor`). Drops attributes set on `self`.
    pub fn into_style_attributes(self) -> StyleAttributes {
        self.style
    }

    /// An attribute the component sets itself, e.g. `type="button"`. A `false`
    /// or `None` is not pushed (see `meaningful`).
    pub fn attr<T>(mut self, name: &'static str, value: impl IntoAttributeValue<T>) -> Self {
        if let Some(attribute) = meaningful(name, value) {
            self.own.push(attribute);
        }
        self
    }

    /// A default the caller's own attribute overrides; [`BoxStyle::attr`] would
    /// win instead (the bug `Button`'s hardcoded `type="button"` was).
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

    /// An event handler the component sets itself. Goes through `EventHandlerValue`:
    /// a bare `AttributeValue::listener::<T>` panics on the first real event.
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
        // A component's own root id plus the caller's: `use_root_id` makes both
        // the caller's, so keeping the first is enough.
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

/// The styled element every component is built from: any tag, `sx`, states and
/// the focus ring. A filled focusable `Box` rings itself in its fill's colours (todo 630).
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::{components::{Box, HtmlTag}, sx::sx};
/// # fn app() -> Element {
/// rsx! {
///     Box { component: HtmlTag::Section, sx: sx().padding("md"), "Content" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/box>
#[component]
pub fn Box(props: BoxProps) -> Element {
    let mut style = BoxBuilder {
        framework_sx: props.framework_sx,
        class: Some(&props.class),
        sx: Some(&props.sx),
        parts: None,
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
