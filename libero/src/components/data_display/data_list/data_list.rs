use dioxus::prelude::*;

use crate::{
    components::{Box, Input, Orientation, States, common::base_props},
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{DataListDefaults, Size},
};

// Shared regardless of orientation: browsers give `dl`/`dd` their own default
// margin, and `dt` an implicit bold weight is worth making explicit rather
// than relying on the UA stylesheet.
fn data_list_reset_sx() -> Sx {
    sx().margin("0")
        .selector("& dt", sx().font_weight("600"))
        .selector("& dd", sx().margin("0"))
}

// Vertical (the default): `dt`/`dd` pairs stack top-to-bottom, in document
// order - no grid needed, a plain column flow already does this. Horizontal
// overrides to a grid: a `dt` can be followed by any number of `dd`s (one
// term, several descriptions), so column position is pinned explicitly on
// every `dt`/`dd` rather than left to the grid's row-major auto-placement -
// otherwise a second `dd` for the same term would auto-flow into the term's
// own column instead of stacking under the first `dd`, in the value column
// where it belongs.
static DATA_LIST_SX: StaticSx = StaticSx::new(|| {
    DataListDefaults::theme_vars()
        .display("flex")
        .flex_direction("column")
        .and(data_list_reset_sx())
        .when(
            "horizontal",
            sx().display("grid")
                .grid_template_columns("max-content 1fr")
                .align_items("baseline")
                .selector("& dt", sx().grid_column("1"))
                .selector("& dd", sx().grid_column("2")),
        )
});

base_props! {
    pub struct DataListProps {
        /// `"horizontal"` places each description beside its term (a grid,
        /// term column then value column); `"vertical"` (the default) stacks
        /// the description below it.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// The gap between rows - `theme.data_list.size` by default. For a
        /// custom (non-scale) gap, use `sx` instead.
        #[props(default, into)]
        gap: Input<Size>,
        /// [`DataListItem`](super::DataListItem)s - or anything else that
        /// renders `dt`/`dd` content.
        children: Element,
    }
}

/// A `<dl>` of term/description pairs - the semantic element for key/value
/// data (e.g. a details panel: "Status" / "Active", "Created" / a date).
/// Unlike [`List`](crate::components::List), a single term can have more
/// than one description - see [`DataListItem`](super::DataListItem).
#[component]
pub fn DataList(props: DataListProps) -> Element {
    let theme = use_theme();

    let is_horizontal =
        props.orientation.as_ref().copied().unwrap_or_default() == Orientation::Horizontal;

    let gap = props.gap.as_ref().copied().unwrap_or(theme.data_list.size);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(gap.state_name(), true)
        .with("horizontal", is_horizontal);

    rsx! {
        Box {
            component: "dl",
            class: props.class,
            sx: props.sx,
            states,
            framework_sx: &DATA_LIST_SX,
            attributes: props.attributes,
            {props.children}
        }
    }
}
