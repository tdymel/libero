use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    CssLayer,
    components::{
        accessibility::VisuallyHidden,
        common::{
            Glyph, HtmlTag, Input, Part, States, base_props, input_from_str, parts_enum,
            parts_source, sx_source, use_style_attributes, with_parts,
        },
    },
    context::IconSlot,
    hooks::{use_css, use_localization, use_theme},
    sx::{StaticSx, Sx, sx},
    theme::{AnchorDefaults, Size},
};

use crate::components::layout::{box_style, listener_attribute, render_anchor};
use crate::utils::warn;

pub use crate::theme::AnchorUnderline;

input_from_str!(AnchorUnderline);

fn underline_sx(underline: AnchorUnderline) -> Sx {
    match underline {
        AnchorUnderline::Always => sx().text_decoration("underline"),
        AnchorUnderline::NotHover => sx()
            .text_decoration("underline")
            .hover_tint(":hover", sx().text_decoration("none")),
        AnchorUnderline::Never => sx().text_decoration("none"),
        AnchorUnderline::Hover => sx()
            .text_decoration("none")
            .hover_tint(":hover", sx().text_decoration("underline")),
    }
}

// Text's theme-level sizing, not `Text` itself - it has no href/target/rel
// escape hatch to render a real anchor with.
static ANCHOR_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = AnchorDefaults::theme_vars()
        .margin("0")
        .overflow_wrap("anywhere");

    let base = AnchorUnderline::ALL.iter().fold(base, |base, &underline| {
        base.when(underline.state_name(), underline_sx(underline))
    });
    // Without `to` it is a `<button>`: drop the native chrome, keep the link look.
    base.selector(
        ":is(button)",
        sx().display("inline")
            .background("none")
            .border("0")
            .padding("0")
            .text_align("inherit")
            .cursor("pointer"),
    )
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
    target.is_some_and(|target| target.eq_ignore_ascii_case("_blank")) && new_tab_hint
}

/// Todo 579: a hidden "(opens in a new tab)" and, with `icon` (unset: the theme's
/// `new_tab_icon`, todo 3324), the icon, after a link's text.
/// `in_text`: the link lays out inline, not as a flex row such as `Chip`'s.
#[component]
pub(crate) fn NewTabHint(
    #[props(default)] in_text: bool,
    #[props(default)] icon: Option<bool>,
) -> Element {
    let class = use_css(Some(&NEW_TAB_HINT_SX), CssLayer::Framework);
    let new_tab = use_localization().anchor.new_tab;
    let drawn = icon.unwrap_or(use_theme().anchor.new_tab_icon);

    // The no-break space keeps the icon on the last word's line. Natively in text it
    // goes before the span, as the name's space: Blitz drops it at an inline element's start (891).
    // Without the icon it is a plain space, which a link's end collapses instead of underlining.
    let native = cfg!(all(not(target_arch = "wasm32"), feature = "native"));
    let (before, inside, gap) = match (in_text && native, drawn) {
        (true, true) => ("\u{a0}", "", ""),
        (true, false) => (" ", "", ""),
        (false, true) => ("", "\u{a0}", " "),
        (false, false) => ("", "", " "),
    };
    rsx! {
        {before}
        if drawn {
            span {
                class,
                "data-slot": AnchorPart::NewTab.slot(),
                "aria-hidden": "true",
                {inside}
                Glyph { slot: IconSlot::ExternalLink, icon: lucide::external_link::outlined }
            }
        }
        VisuallyHidden { "{gap}{new_tab}" }
    }
}
parts_enum! {
    /// [`Anchor`]'s inner parts, for its `parts` prop.
    pub enum AnchorPart {
        /// The new-tab icon after the text, with `target: "_blank"` only.
        NewTab = "new-tab" => "& > [data-slot='new-tab']",
    }
}

base_props! {
    parts(AnchorPart);
    pub struct AnchorProps {
        #[props(default, into)]
        size: Input<Size>,
        /// A path/URL or a typed route (`Route::Foo {}`); router navigation when mounted.
        /// Unset, a `<button>` with the link look, for an action.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        // `Option`: a defaulted `EventHandler` allocates on every render.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        #[props(default)]
        target: Option<String>,
        #[props(default, into)]
        underline: Input<AnchorUnderline>,
        /// With `target: "_blank"`, a hidden "(opens in a new tab)". `false` drops it and the icon.
        #[props(default = true)]
        new_tab_hint: bool,
        /// With the hint, draws the new-tab icon. Unset, the theme's `anchor.new_tab_icon`, off.
        #[props(default)]
        new_tab_icon: Option<bool>,
        children: Element,
    }
}

/// A text link to a URL or a router route; without `to`, a button that looks like one.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Anchor;
/// # fn app() -> Element {
/// rsx! {
///     Anchor { to: "https://dioxuslabs.com", target: "_blank", "Dioxus" }
///     Anchor { onclick: move |_| {}, "Show details" }
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
    let mut merged = None;
    let style_attributes = use_style_attributes(
        &props.class,
        Some(&ANCHOR_BASE_SX),
        with_parts(
            parts_source(&props.parts),
            sx_source(&props.sx),
            &mut merged,
        ),
        &states,
        &Input::None,
        None,
        true,
    );

    // Once per mount, before any branch: hook order must not follow `to`.
    use_hook(|| {
        if props.to.as_ref().is_none() {
            warn_link_only_props(&props.target, props.new_tab_hint, props.new_tab_icon);
        }
    });
    let Some(to) = props.to.as_ref().cloned() else {
        // `<button>` defaults to `submit`; a caller can still ask for it.
        return box_style(style_attributes)
            .event("onclick", props.onclick)
            .attr_default("type", "button")
            .render(HtmlTag::Button, props.attributes, props.children);
    };

    let children = match wants_new_tab_hint(props.target.as_deref(), props.new_tab_hint) {
        true => rsx! {
            {props.children}
            NewTabHint { in_text: true, icon: props.new_tab_icon }
        },
        false => props.children,
    };

    // A caller's list, so it runs after the router's own click and keeps it.
    let mut attributes = props.attributes;
    attributes.extend(listener_attribute("onclick", props.onclick));
    render_anchor(
        style_attributes,
        to,
        props.target,
        None::<fn(MountedEvent)>,
        attributes,
        children,
    )
}

/// Debug builds: the props only a link reads, set on an `Anchor` without `to`.
fn warn_link_only_props(target: &Option<String>, new_tab_hint: bool, new_tab_icon: Option<bool>) {
    let set = [
        ("target", target.is_some()),
        ("new_tab_hint", !new_tab_hint),
        ("new_tab_icon", new_tab_icon.is_some()),
    ];
    for (prop, _) in set.iter().filter(|(_, set)| *set) {
        warn(&format!(
            "Anchor: `{prop}` is ignored without `to` - the anchor renders a `<button>`."
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<AnchorPart>(),
            [("new-tab", "& > [data-slot='new-tab']")]
        );
    }

    /// Todo 5363: the link-only props say they do nothing on the button.
    #[cfg(debug_assertions)]
    #[test]
    fn link_only_props_warn_without_to() {
        fn app() -> Element {
            rsx! {
                crate::LiberoProvider {
                    Anchor { target: "_blank", new_tab_hint: false, onclick: move |_| {}, "Go" }
                    Anchor { to: "https://example.com", target: "_blank", "Link" }
                }
            }
        }
        let warnings = crate::utils::warnings_of(app);
        let ours: Vec<_> = warnings
            .iter()
            .filter(|w| w.starts_with("Anchor:"))
            .collect();
        assert_eq!(ours.len(), 2, "{warnings:?}");
        assert!(ours[0].contains("`target`") && ours[1].contains("`new_tab_hint`"));
    }
}
