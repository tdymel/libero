use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, Orientation, States, common::base_props, layout::use_box},
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{DataListDefaults, Size},
};

// Pins what the UA stylesheet would otherwise decide: `dl`/`dd` margins and
// `dt`'s implicit bold.
fn data_list_reset_sx() -> Sx {
    sx().margin("0")
        // A long word in a term or description wraps instead of widening the page (1.4.10).
        .with("overflow-wrap", "anywhere")
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
            // A term takes its own width up to half the list, then wraps, so
            // the description column keeps room at 320px (1.4.10).
            sx().display("grid")
                .grid_template_columns("fit-content(50%) minmax(0, 1fr)")
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

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(gap.state_name(), true)
        .with("horizontal", is_horizontal)
        .into();

    use_box()
        .framework_sx(&DATA_LIST_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .render(HtmlTag::Dl, props.attributes, props.children)
}
