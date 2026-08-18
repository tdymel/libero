use dioxus::prelude::*;

use crate::{
    components::{
        Input, States,
        common::{base_props, input_from_str},
    },
    str_enum::str_enum,
    sx::{StaticSx, Sx, sx},
    theme::{Size, TextDefaults},
};

use super::InternalAnchor;

str_enum! {
    pub enum AnchorUnderline {
        Always = "always",
        #[default]
        Hover = "hover",
        Never = "never",
    }
}

input_from_str!(AnchorUnderline);

fn underline_token(underline: AnchorUnderline) -> &'static str {
    match underline {
        AnchorUnderline::Always => "underline-always",
        AnchorUnderline::Hover => "underline-hover",
        AnchorUnderline::Never => "underline-never",
    }
}

fn underline_sx(underline: AnchorUnderline) -> Sx {
    match underline {
        AnchorUnderline::Always => sx().text_decoration("underline"),
        AnchorUnderline::Never => sx().text_decoration("none"),
        AnchorUnderline::Hover => sx()
            .text_decoration("none")
            .hover(sx().text_decoration("underline")),
    }
}

// Reuses Text's own theme-level sizing (TextDefaults::theme_vars), not Text
// the component, since Text has no href/target/rel escape hatch to render a
// real anchor with.
static ANCHOR_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = TextDefaults::theme_vars().color("primary.6").margin("0");

    [
        AnchorUnderline::Always,
        AnchorUnderline::Hover,
        AnchorUnderline::Never,
    ]
    .into_iter()
    .fold(base, |base, underline| {
        base.when(underline_token(underline), underline_sx(underline))
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
    let size = props.size.copied_or(Size::Md);
    let underline = props.underline.copied_or_default();

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(underline_token(underline), true);

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
