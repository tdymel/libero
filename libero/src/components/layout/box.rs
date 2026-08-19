use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{attr, base_props, render_polymorphic, use_style_attributes},
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
