use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{attr, base_props, render_polymorphic, use_style_attributes},
    },
    sx::{StaticSx, Sx},
};

base_props! {
    // `img`/`a`/`button` cover the non-global attributes callers actually
    // set on a `Box` (`src`, `alt`, `href`, `target`, `value`, `disabled`).
    // An attribute name two of them share (or one `GlobalAttributes`
    // already carries) is ambiguous at the call site, hence the explicit
    // `alt`/`r#type` fields below - an inherent builder method wins over
    // both. `referrerpolicy` (`img` and `a`) is unused so far.
    extends(img, a, button);
    pub struct BoxProps {
        /// CSS custom properties set directly on this element's `style`
        /// attribute - lets `sx`/`framework_sx` reference a per-instance value
        /// via `var(--name, fallback)` without generating a new class for every
        /// distinct value a caller passes.
        #[props(default, into)]
        variables: Input<Variables>,
        /// Which element to render as - `div` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Framework-layer `Sx` for a component built on top of `Box` (e.g.
        /// `Divider`'s own base styles), registered alongside `Box`'s own.
        #[props(default)]
        framework_sx: Option<&'static StaticSx>,
        /// A raw literal `style` - merged with (not overwritten by) whatever
        /// `variables` produces, `variables`' declarations first.
        #[props(default)]
        style: Option<String>,
        #[props(default)]
        alt: Option<String>,
        #[props(default)]
        r#type: Option<String>,
        children: Element,
    }
}

#[component]
pub fn Box(props: BoxProps) -> Element {
    let style_attributes = use_style_attributes(
        &props.class,
        props.framework_sx,
        &props.sx,
        &props.states,
        &props.variables,
        props.style,
    );
    let component = props.component.copied_or_default();

    let attributes = props
        .attributes
        .into_iter()
        .chain(
            [
                props.alt.map(|value| attr("alt", value)),
                props.r#type.map(|value| attr("type", value)),
            ]
            .into_iter()
            .flatten(),
        )
        .collect::<Vec<_>>();

    render_polymorphic(
        component,
        style_attributes.class,
        style_attributes.data_state,
        style_attributes.style,
        attributes,
        props.children,
    )
}
