use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, variables},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::{GRID_AREAS_VAR, GRID_COLUMNS_VAR, GRID_GAP, Size, SizeCss},
};

use super::GridTemplate;

/// What a `GridZone` reads to place itself. Signal-backed because
/// `use_context_provider`/`use_context` each run once - a plain value would be
/// the first render's snapshot forever.
#[derive(Clone, Copy)]
pub(crate) struct GridContext {
    pub template: Signal<GridTemplate>,
}

static GRID_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .display("grid")
        .grid_template_areas(GRID_AREAS_VAR.value())
        // `minmax(0, 1fr)`, or a wide child blows its own track out.
        .grid_template_columns(format!(
            "repeat({}, minmax(0, 1fr))",
            GRID_COLUMNS_VAR.value()
        ))
        .gap(GRID_GAP.value());

    Size::ALL.into_iter().fold(base, |base, size| {
        base.when(size.state_name(), sx().gap(SizeCss::SPACING.value(size)))
    })
});

base_props! {
    pub struct GridProps {
        /// `GridZone`s.
        children: Element,
        /// The named-area matrix. Build it once outside the render.
        template: GridTemplate,
        /// Between zones.
        #[props(default, into)]
        gap: Input<Size>,
        #[props(default, into)]
        component: Input<HtmlTag>,
    }
}

/// Lays named zones out in a matrix. Each zone packs its own items - see
/// [`GridZone`](super::GridZone).
#[component]
pub fn Grid(props: GridProps) -> Element {
    let context = use_context_provider(|| GridContext {
        template: Signal::new(props.template.clone()),
    });

    // Render-time write, which costs no extra pass; guarded so an unchanged
    // render does not wake every zone.
    if *context.template.peek() != props.template {
        let mut template = context.template;
        template.set(props.template.clone());
    }

    let variables: Input<Variables> = variables()
        .with(GRID_AREAS_VAR, props.template.0.areas.clone())
        .with(GRID_COLUMNS_VAR, props.template.0.columns.to_string())
        .into();

    let mut states = props.states.unwrap_or_default();
    if let Some(gap) = props.gap.as_ref().copied() {
        states = states.with(gap.state_name(), true);
    }
    let states: Input<States> = states.into();

    use_box()
        .framework_sx(&GRID_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(
            props.component.copied_or(HtmlTag::Div),
            props.attributes,
            props.children,
        )
}
