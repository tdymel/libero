use dioxus::prelude::*;

use crate::{
    components::{Box, Input, Orientation, States, common::base_props},
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{DataListDefaults, Size},
};

// Pins what the UA stylesheet would otherwise decide: `dl`/`dd` margins and
// `dt`'s implicit bold.
fn data_list_reset_sx() -> Sx {
    sx().margin("0")
        .selector("& dt", sx().font_weight("600"))
        .selector("& dd", sx().margin("0"))
}

// Horizontal pins `grid-column` explicitly instead of letting the grid
// auto-place: a term may have several `dd`s, and row-major auto-placement
// would flow the second one into the term column.
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
        /// `"horizontal"` puts each description beside its term;
        /// `"vertical"` (default) stacks it below.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Row gap. Off-scale values go through `sx`.
        #[props(default, into)]
        gap: Input<Size>,
        /// [`DataListItem`](super::DataListItem)s, or any `dt`/`dd` content.
        children: Element,
    }
}

/// A `<dl>` of term/description pairs. Unlike
/// [`List`](crate::components::List), one term can carry several
/// descriptions - see [`DataListItem`](super::DataListItem).
#[component]
pub fn DataList(props: DataListProps) -> Element {
    let theme = use_theme();

    let is_horizontal = props.orientation.copied_or_default() == Orientation::Horizontal;

    let gap = props.gap.copied_or(theme.data_list.size);

    let states = props
        .states
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
