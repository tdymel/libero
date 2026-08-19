use dioxus::prelude::*;

use crate::{
    components::{
        Input, States,
        common::{base_props, input_from_str},
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{AnchorDefaults, Size},
};

use super::InternalAnchor;

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

// Reuses Text's theme-level sizing (via AnchorDefaults::theme_vars), not Text
// the component, since Text has no href/target/rel escape hatch to render a
// real anchor with.
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
        /// A plain path/URL or a typed route (anything `Into<NavigationTarget>`,
        /// e.g. `Route::Foo {}`). Resolves through the app's Dioxus router when
        /// one is mounted and `target` allows it (unset or `"_blank"`) -
        /// internal targets then get SPA navigation instead of a full page
        /// reload. Falls back to a plain `href` otherwise.
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

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(underline.state_name(), true);

    rsx! {
        InternalAnchor {
            to: props.to,
            target: props.target,
            class: props.class,
            sx: props.sx,
            framework_sx: &ANCHOR_BASE_SX,
            states,
            attributes: props.attributes,
            {props.children}
        }
    }
}
