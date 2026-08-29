use crate::{
    components::{ClassList, Input, States, Variables},
    hooks::{SxSource, use_box_css},
    sx::{StaticSx, Sx},
};

/// The class / `data-state` / `style` triple the styling props resolve to.
#[derive(Clone)]
pub(crate) struct StyleAttributes {
    pub class: String,
    pub data_state: Option<String>,
    pub style: Option<String>,
}

/// Resolves the styling props every `Box`-shaped component shares.
///
/// Calls [`use_box_css`], so it is a hook: call it before any early return.
pub(crate) fn use_style_attributes(
    class: &Input<ClassList>,
    framework_sx: Option<&'static StaticSx>,
    sx: &Input<Sx>,
    states: &Input<States>,
    variables: &Input<Variables>,
    style: Option<String>,
    focus_ring: bool,
) -> StyleAttributes {
    // Not `sx.as_ref()`: that collapses the lifetime, so a caller's `static`
    // is rehashed and rebuilt as if it were freshly built this render.
    let sx_source = match sx {
        Input::None => None,
        Input::Value(sx) => Some(SxSource::Owned(sx)),
        Input::Static(sx) => Some(SxSource::Static(sx)),
    };
    let class = use_box_css(
        class,
        focus_ring.then_some(&BOX_FOCUS_SX),
        framework_sx,
        sx_source,
    );

    let variables_style = variables
        .as_ref()
        .map(Variables::render)
        .filter(|style| !style.is_empty());

    StyleAttributes {
        class,
        data_state: states.as_ref().and_then(States::data_state),
        style: match (variables_style, style) {
            (Some(variables), Some(raw)) => Some(format!("{variables}{raw}")),
            (Some(style), None) | (None, Some(style)) => Some(style),
            (None, None) => None,
        },
    }
}

// Shared rather than living in `Box`: a plain div only reaches this once it
// has a tabindex, but an `<a href>` is focusable on its own.
static BOX_FOCUS_SX: StaticSx =
    StaticSx::new(|| crate::sx::sx().focus_visible(super::focus_ring_sx()));
