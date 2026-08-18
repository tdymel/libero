use crate::{
    CssLayer,
    components::{ClassList, Input, States, Variables},
    hooks::use_css,
    sx::{StaticSx, Sx},
};

/// The class / `data-state` / `style` triple every element built from our
/// styling props ends up rendering.
pub(crate) struct StyleAttributes {
    pub class: ClassList,
    pub data_state: Option<String>,
    pub style: Option<String>,
}

/// Resolves the styling props shared by every `Box`-shaped component into
/// what the DOM actually needs.
///
/// Three `use_css` calls, so it is a hook: call it unconditionally, before
/// any early return.
pub(crate) fn use_style_attributes(
    class: &Input<ClassList>,
    framework_sx: Option<&'static StaticSx>,
    sx: &Input<Sx>,
    states: &Input<States>,
    variables: &Input<Variables>,
    style: Option<String>,
) -> StyleAttributes {
    let focus_class = use_css(Some(&BOX_FOCUS_SX), CssLayer::Framework);
    let framework_class = use_css(framework_sx, CssLayer::Framework);
    let static_class = use_css(sx.as_ref(), CssLayer::UserStatic);

    let variables_style = variables
        .as_ref()
        .map(Variables::to_string)
        .filter(|style| !style.is_empty());

    StyleAttributes {
        class: class
            .as_ref()
            .cloned()
            .unwrap_or_default()
            .with(framework_class)
            .with(focus_class)
            .with(static_class),
        data_state: states.as_ref().and_then(States::data_state),
        style: match (variables_style, style) {
            (Some(variables), Some(raw)) => Some(format!("{variables}{raw}")),
            (Some(style), None) | (None, Some(style)) => Some(style),
            (None, None) => None,
        },
    }
}

// Only ever shows up for an element that received a tabindex (e.g. because it
// was made clickable via `onclick`), since a plain div isn't keyboard-
// focusable on its own - but an `<a href>` is, which is why it is shared
// here rather than living in `Box` alone.
static BOX_FOCUS_SX: StaticSx =
    StaticSx::new(|| crate::sx::sx().focus_visible(super::focus_ring_sx()));
