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

fn get_size_sx(size: Size) -> &'static Sx {
    static XS: StaticSx = StaticSx::new(DataListDefaults::xs_sx);
    static SM: StaticSx = StaticSx::new(DataListDefaults::sm_sx);
    static MD: StaticSx = StaticSx::new(DataListDefaults::md_sx);
    static LG: StaticSx = StaticSx::new(DataListDefaults::lg_sx);
    static XL: StaticSx = StaticSx::new(DataListDefaults::xl_sx);
    static XXL: StaticSx = StaticSx::new(DataListDefaults::xxl_sx);

    match size {
        Size::Xs => &XS,
        Size::Sm => &SM,
        Size::Md => &MD,
        Size::Lg => &LG,
        Size::Xl => &XL,
        Size::Xxl => &XXL,
    }
}

fn data_list_dynamic_sx(props: &DataListProps) -> Sx {
    sx().apply_if(props.gap.as_ref(), |sx, gap| sx.gap(gap.clone()))
}

base_props! {
    pub struct DataListProps {
        /// `"horizontal"` places each description beside its term (a grid,
        /// term column then value column); `"vertical"` (the default) stacks
        /// the description below it.
        #[props(default, into)]
        orientation: Input<ThemeAwareValue>,
        /// Overrides the gap between rows - `theme.data_list.size` by default.
        #[props(default, into)]
        gap: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
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

    let size = match props.size.as_ref() {
        Some(ThemeAwareValue::Size(size)) => *size,
        _ => theme.data_list.size,
    };
    let size_class = use_css(get_size_sx(size), crate::CssLayer::Framework);
    let dynamic_class = use_css(&data_list_dynamic_sx(&props), crate::CssLayer::UserDynamic);

    rsx! {
        Box {
            component: "dl",
            class: props.class.unwrap_or_default().with(size_class).with(dynamic_class),
            sx: props.sx,
            states: props.states,
            framework_sx,
            attributes: props.attributes,
            {props.children}
        }
    }
}
