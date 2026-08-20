use dioxus::core::{AttributeValue, IntoAttributeValue};
use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States, Variables,
        common::{StyleAttributes, attr, base_props, render_polymorphic, use_style_attributes},
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
            style: use_style_attributes(
                self.class.unwrap_or(&NONE_CLASS),
                self.framework_sx,
                self.sx.unwrap_or(&NONE_SX),
                self.states.unwrap_or(&NONE_STATES),
                self.variables.unwrap_or(&NONE_VARIABLES),
                self.style,
            ),
        }
    }
}

/// Resolved styling, ready to render as any element. Pure - no hooks - so it
/// can be used in a branch, or not at all.
pub(crate) struct BoxStyle {
    style: StyleAttributes,
    /// The component's own attributes, rendered **after** the caller's so the
    /// caller keeps winning a duplicate, which is what `Box` always did.
    own: Vec<Attribute>,
}

impl BoxStyle {
    /// An attribute the component sets itself, e.g. `type="button"`.
    pub fn attr<T>(mut self, name: &'static str, value: impl IntoAttributeValue<T>) -> Self {
        self.own.push(attr(name, value));
        self
    }

    /// An event handler the component sets itself. Hand-building the
    /// `Attribute` is the one thing `rsx!` does that a plain call cannot.
    pub fn event<T: 'static>(
        mut self,
        name: &'static str,
        handler: impl FnMut(Event<T>) + 'static,
    ) -> Self {
        self.own.push(Attribute::new(
            name,
            AttributeValue::listener(handler),
            None,
            false,
        ));
        self
    }

    pub fn render(
        self,
        component: HtmlTag,
        attributes: Vec<Attribute>,
        children: Element,
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

        render_polymorphic(
            component,
            self.style.class,
            self.style.data_state,
            self.style.style,
            attributes,
            children,
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
