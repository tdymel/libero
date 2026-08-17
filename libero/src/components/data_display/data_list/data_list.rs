use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    hooks::{use_css, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
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

// `dt`/`dd` pairs stack top-to-bottom, in document order - no grid needed,
// a plain column flow already does this.
static DATA_LIST_VERTICAL_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .and(data_list_reset_sx())
});

// A `dt` can be followed by any number of `dd`s (one term, several
// descriptions), so column position is pinned explicitly on every `dt`/`dd`
// rather than left to the grid's row-major auto-placement - otherwise a
// second `dd` for the same term would auto-flow into the term's own column
// instead of stacking under the first `dd`, in the value column where it
// belongs.
static DATA_LIST_HORIZONTAL_SX: StaticSx = StaticSx::new(|| {
    sx().display("grid")
        .grid_template_columns("max-content 1fr")
        .align_items("baseline")
        .selector("& dt", sx().grid_column("1"))
        .selector("& dd", sx().grid_column("2"))
        .and(data_list_reset_sx())
});

static DATA_LIST_SIZE_SX: StaticSx = StaticSx::new(DataListDefaults::theme_vars);

base_props! {
    pub struct DataListProps {
        /// `"horizontal"` places each description beside its term (a grid,
        /// term column then value column); `"vertical"` (the default) stacks
        /// the description below it.
        #[props(default, into)]
        orientation: Input<ThemeAwareValue>,
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

    let is_horizontal = matches!(
        props.orientation.as_ref(),
        Some(ThemeAwareValue::String(value)) if value == "horizontal"
    );
    let framework_sx: &'static StaticSx = if is_horizontal {
        &DATA_LIST_HORIZONTAL_SX
    } else {
        &DATA_LIST_VERTICAL_SX
    };

    let gap = props.gap.as_ref().copied().unwrap_or(theme.data_list.size);
    let size_class = use_css(&DATA_LIST_SIZE_SX, crate::CssLayer::Framework);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with(gap.state_name(), true);

    rsx! {
        Box {
            component: "dl",
            class: props.class.unwrap_or_default().with(size_class),
            sx: props.sx,
            states,
            framework_sx,
            attributes: props.attributes,
            {props.children}
        }
    }
}
