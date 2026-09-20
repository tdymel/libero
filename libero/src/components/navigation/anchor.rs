use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        accessibility::VisuallyHidden,
        common::{
            ExternalLinkIcon, Input, States, base_props, input_from_str, use_style_attributes,
        },
    },
    hooks::{use_css, use_localization, use_theme},
    sx::{StaticSx, Sx, sx},
    theme::{AnchorDefaults, Size},
};

use crate::components::layout::render_anchor;

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

static NEW_TAB_HINT_SX: StaticSx = StaticSx::new(|| {
    // Never shrunk in a flex root such as `Chip`'s.
    sx().white_space("nowrap").flex_shrink("0").selector(
        "& > svg",
        sx().width("0.8em")
            .height("0.8em")
            .vertical_align("-0.05em"),
    )
});

pub(crate) fn wants_new_tab_hint(target: Option<&str>, new_tab_hint: bool) -> bool {
    target == Some("_blank") && new_tab_hint
}

/// Todo 579: the icon plus a hidden "(opens in a new tab)", after a link's text.
/// `in_text`: the link lays out inline, not as a flex row such as `Chip`'s.
#[component]
pub(crate) fn NewTabHint(#[props(default)] in_text: bool) -> Element {
    let class = use_css(Some(&NEW_TAB_HINT_SX), CssLayer::Framework);
    let new_tab = use_localization().anchor.new_tab;

    // The no-break space keeps the icon on the last word's line. Natively in text it
    // goes before the span, as the name's space: Blitz drops it at an inline element's start (891).
    let native = cfg!(all(not(target_arch = "wasm32"), feature = "native"));
    let (before, inside, gap) = if in_text && native {
        ("\u{a0}", "", "")
    } else {
        ("", "\u{a0}", " ")
    };
    rsx! {
        {before}
        span { class, "data-anchor-new-tab": "", "aria-hidden": "true", {inside}, ExternalLinkIcon {} }
        VisuallyHidden { "{gap}{new_tab}" }
    }
}

base_props! {
    pub struct AnchorProps {
        #[props(default, into)]
        size: Input<Size>,
        /// A path/URL or a typed route (`Route::Foo {}`); router navigation when mounted.
        #[props(into)]
        to: NavigationTarget,
        #[props(default)]
        target: Option<String>,
        #[props(default, into)]
        underline: Input<AnchorUnderline>,
        /// With `target: "_blank"`, an icon and a hidden "(opens in a new tab)".
        #[props(default = true)]
        new_tab_hint: bool,
        children: Element,
    }
}

/// A text link to a URL or a router route.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Anchor;
/// # fn app() -> Element {
/// rsx! {
///     Anchor { to: "https://dioxuslabs.com", target: "_blank", "Dioxus" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/anchor>
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

    // Resolved here: an `InternalAnchor` scope would re-render with `children`.
    let style_attributes = use_style_attributes(
        &props.class,
        Some(&ANCHOR_BASE_SX),
        &props.sx,
        &states,
        &Input::None,
        None,
        true,
    );

    let children = match wants_new_tab_hint(props.target.as_deref(), props.new_tab_hint) {
        true => rsx! {
            {props.children}
            NewTabHint { in_text: true }
        },
        false => props.children,
    };

    render_anchor(
        style_attributes,
        props.to,
        props.target,
        None::<fn(MountedEvent)>,
        props.attributes,
        children,
    )
}
