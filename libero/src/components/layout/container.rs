use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, Variables, base_props, inset_focus_ring_sx, variables},
        layout::use_box,
    },
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CONTAINER_GUTTERS, CONTAINER_SIZE, FOCUS_RING_WIDTH, SizeCss},
};

static CONTAINER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .margin_left("auto")
        .margin_right("auto")
        .max_width(CONTAINER_SIZE.overridable())
        .padding_left(CONTAINER_GUTTERS.overridable())
        .padding_right(CONTAINER_GUTTERS.overridable())
        // Inset: a full-width container's outer ring is clipped at the viewport sides.
        .focus_visible(inset_focus_ring_sx(&format!(
            "calc(-1 * {})",
            FOCUS_RING_WIDTH.value()
        )))
});

fn container_variables(props: &ContainerProps) -> Variables {
    variables()
        .with(
            CONTAINER_SIZE.override_var(),
            props.size.resolve(Some(SizeCss::BREAKPOINT)),
        )
        .with(
            CONTAINER_GUTTERS.override_var(),
            props.gutters.resolve(Some(SizeCss::SPACING)),
        )
}

base_props! {
    pub struct ContainerProps {
        /// Which element to render as - `div` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Maximum width, a breakpoint size or any CSS length.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// Inline padding on both sides.
        #[props(default, into)]
        gutters: Input<ThemeAwareValue>,
        children: Element,
    }
}

/// Centers content horizontally with a maximum width and side gutters.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Container;
/// # fn app() -> Element {
/// rsx! {
///     Container { size: "md", "Page content" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/container>
#[component]
pub fn Container(props: ContainerProps) -> Element {
    let variables: Input<Variables> = container_variables(&props).into();

    use_box()
        .framework_sx(&CONTAINER_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        // The inset ring above replaces the box's outer one.
        .focus_ring(false)
        .prepare()
        .render(
            props.component.copied_or_default(),
            props.attributes,
            props.children,
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::Size;

    #[test]
    fn size_and_gutters_set_the_override_twins() {
        let props = ContainerProps {
            component: Input::None,
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            size: Size::Md.into(),
            gutters: Size::Lg.into(),
            children: rsx! {},
        };

        assert_eq!(
            container_variables(&props).to_string(),
            format!(
                "{}:{};{}:{};",
                CONTAINER_SIZE.override_var().name(),
                SizeCss::BREAKPOINT.value(Size::Md),
                CONTAINER_GUTTERS.override_var().name(),
                SizeCss::SPACING.value(Size::Lg)
            )
        );
    }
}
