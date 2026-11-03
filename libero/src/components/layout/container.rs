use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Variables, base_props, inset_focus_ring_sx, responsive_var_sx,
            variables, with_own,
        },
        layout::use_box,
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CONTAINER_GUTTERS, CONTAINER_SIZE, FOCUS_RING_WIDTH, Responsive, SizeCss},
};

static CONTAINER_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().width("100%")
        .margin_left("auto")
        .margin_right("auto")
        .max_width(CONTAINER_SIZE.overridable())
        // Not edge-bound: beside a Sidebar or in a Drawer it would clear the notch twice (5519).
        .padding_left(CONTAINER_GUTTERS.overridable())
        .padding_right(CONTAINER_GUTTERS.overridable())
        // Inset: a full-width container's outer ring is clipped at the viewport sides.
        .focus_visible(inset_focus_ring_sx(&format!(
            "calc(-1 * {})",
            FOCUS_RING_WIDTH.value()
        )))
});

/// A plain value's override var; a per-breakpoint one goes to [`container_sx`] instead.
fn plain_var(value: &Input<Responsive<ThemeAwareValue>>, scale: SizeCss) -> Option<String> {
    value
        .as_ref()
        .filter(|value| value.is_plain())
        .and_then(|value| value.base_ref().resolve(Some(scale)))
}

fn container_variables(props: &ContainerProps) -> Variables {
    variables()
        .with(
            CONTAINER_SIZE.override_var(),
            plain_var(&props.size, SizeCss::BREAKPOINT),
        )
        .with(
            CONTAINER_GUTTERS.override_var(),
            plain_var(&props.gutters, SizeCss::SPACING),
        )
}

/// The per-breakpoint `size` and `gutters`, in the user layer under the caller's `sx`.
fn container_sx(props: &ContainerProps) -> Option<Sx> {
    let var_sx = |value: &Input<Responsive<ThemeAwareValue>>, var, scale| {
        value
            .as_ref()
            .and_then(|value| responsive_var_sx(value, var, Some(scale)))
    };
    let size = var_sx(
        &props.size,
        CONTAINER_SIZE.override_var(),
        SizeCss::BREAKPOINT,
    );
    let gutters = var_sx(
        &props.gutters,
        CONTAINER_GUTTERS.override_var(),
        SizeCss::SPACING,
    );
    match (size, gutters) {
        (Some(size), Some(gutters)) => Some(size.and(gutters)),
        (size, gutters) => size.or(gutters),
    }
}

base_props! {
    pub struct ContainerProps {
        /// Which element to render as - `div` by default.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// Maximum width, a breakpoint size or any CSS length, or one per breakpoint,
        /// `size: responsive(Size::Sm).lg(Size::Lg)`. Breakpoints follow the window.
        #[props(default, into)]
        size: Input<Responsive<ThemeAwareValue>>,
        /// Inline padding on both sides, or one per breakpoint, `gutters: responsive(Size::Xs).md(Size::Lg)`.
        #[props(default, into)]
        gutters: Input<Responsive<ThemeAwareValue>>,
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
    let sx = with_own(container_sx(&props), &props.sx);

    use_box()
        .framework_sx(&CONTAINER_BASE_SX)
        .class(&props.class)
        .sx(&sx)
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
        assert!(container_sx(&props).is_none());

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
