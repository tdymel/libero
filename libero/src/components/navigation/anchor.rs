use dioxus::prelude::*;

use crate::{
    components::{
        Input, States,
        common::{base_props, input_from_str, use_style_attributes},
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{AnchorDefaults, Size},
};

use super::render_anchor;

pub use crate::theme::AnchorUnderline;

input_from_str!(AnchorUnderline);

fn underline_sx(underline: AnchorUnderline) -> Sx {
    match underline {
        AnchorUnderline::Always => sx().text_decoration("underline"),
        AnchorUnderline::Never => sx().text_decoration("none"),
        AnchorUnderline::Hover => sx()
            .text_decoration("none")
            .hover(sx().text_decoration("underline")),
    }
}

// Text's theme-level sizing, not `Text` itself - it has no href/target/rel
// escape hatch to render a real anchor with.
static ANCHOR_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = AnchorDefaults::theme_vars().margin("0");

    AnchorUnderline::ALL.iter().fold(base, |base, &underline| {
        base.when(underline.state_name(), underline_sx(underline))
    })
});

base_props! {
    pub struct AnchorProps {
        #[props(default, into)]
        size: Input<Size>,
        /// A path/URL or a typed route (`Route::Foo {}`). With a router
        /// mounted and `target` unset or `"_blank"`, an internal target gets
        /// SPA navigation; otherwise a plain `href`.
        #[props(into)]
        to: NavigationTarget,
        #[props(default)]
        target: Option<String>,
        #[props(default, into)]
        underline: Input<AnchorUnderline>,
        children: Element,
    }
}

#[component]
pub fn Anchor(props: AnchorProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.anchor.size);
    let underline = props.underline.copied_or(theme.anchor.underline);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(underline.state_name(), true)
        .into();

    // Resolved here rather than in an `InternalAnchor` scope, which `children`
    // would re-render every time.
    let style_attributes = use_style_attributes(
        &props.class,
        Some(&ANCHOR_BASE_SX),
        &props.sx,
        &states,
        &Input::None,
        None,
        true,
    );

    render_anchor(
        style_attributes,
        props.to,
        props.target,
        None::<fn(MountedEvent)>,
        props.attributes,
        props.children,
    )
}
