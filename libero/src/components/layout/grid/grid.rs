use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_props, responsive_sx, variables, with_own,
        },
        layout::use_box,
    },
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, GRID_AREAS_VAR, GRID_COLUMNS_VAR, GRID_GAP, Responsive, Size, SizeCss},
};

use super::GridTemplate;

/// What a `GridZone` reads to place itself. A signal, since the context hooks
/// run once and a plain value would stay the first render's.
#[derive(Clone, Copy)]
pub(crate) struct GridContext {
    pub template: Signal<Responsive<GridTemplate>>,
}

/// A template's two vars, for the inline style or a breakpoint's rule.
fn template_vars(template: &GridTemplate) -> [(CssVar, String); 2] {
    [
        (GRID_AREAS_VAR, template.0.areas.clone()),
        (GRID_COLUMNS_VAR, template.0.columns.to_string()),
    ]
}

/// A template per breakpoint, in the user layer: an inline var would beat its breakpoints.
fn template_sx(template: &Responsive<GridTemplate>) -> Option<Sx> {
    if template.is_plain() {
        return None;
    }
    let declare = |template: &GridTemplate| {
        template_vars(template)
            .into_iter()
            .fold(sx(), |sx, (var, value)| {
                sx.var(var, ThemeAwareValue::String(value))
            })
    };
    let base = declare(template.base_ref());
    Some(template.breakpoints().fold(base, |base, (size, template)| {
        base.breakpoint(size, declare(&template))
    }))
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
        /// The named-area matrix. Build it once outside the render. Per breakpoint too,
        /// `template: responsive(STACKED.clone()).md(SIDE_BY_SIDE.clone())`: list every zone in each.
        #[props(into)]
        template: Responsive<GridTemplate>,
        /// Between zones: a size word, any CSS (`gap: "0"`) or one per breakpoint,
        /// `gap: responsive(Size::Xs).md(Size::Lg)`. Breakpoints follow the window.
        #[props(default, into)]
        gap: Input<Responsive<ThemeAwareValue>>,
        #[props(default, into)]
        component: Input<HtmlTag>,
    }
}

/// Lays named zones out in a matrix; each [`GridZone`](super::GridZone) packs its own items.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Grid, GridArea, GridZone, StaticGridTemplate};
/// #[derive(Clone, Copy, PartialEq)]
/// enum Area { Nav, Main }
///
/// impl GridArea for Area {
///     fn name(&self) -> &'static str {
///         match self { Self::Nav => "nav", Self::Main => "main" }
///     }
/// }
///
/// static PAGE: StaticGridTemplate<Area> =
///     StaticGridTemplate::new(|template| template.row(|row| row.cell(Area::Nav).cells(Area::Main, 3)));
///
/// # fn app() -> Element {
/// rsx! {
///     Grid { template: PAGE.clone(),
///         GridZone { area: Area::Nav, "Links" }
///         GridZone { area: Area::Main, "Content" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/grid>
#[component]
pub fn Grid(props: GridProps) -> Element {
    let context = use_context_provider(|| GridContext {
        template: Signal::new(props.template.clone()),
    });

    // Guarded, so an unchanged render does not wake every zone.
    if *context.template.peek() != props.template {
        let mut template = context.template;
        template.set(props.template.clone());
    }

    let variables: Input<Variables> = match props.template.is_plain() {
        true => template_vars(props.template.base_ref())
            .into_iter()
            .fold(variables(), |variables, (var, value)| {
                variables.with(var, value)
            }),
        false => variables(),
    }
    .into();

    let mut states = props.states.unwrap_or_default();
    if let Some(ThemeAwareValue::Size(gap)) = props.gap.as_ref().map(Responsive::base_ref) {
        states = states.with(gap.state_name(), true);
    }
    let states: Input<States> = states.into();
    let sx = with_own(
        props
            .gap
            .as_ref()
            .and_then(|gap| responsive_sx(gap, Sx::gap)),
        &props.sx,
    );
    let sx = with_own(template_sx(&props.template), &sx);

    use_box()
        .framework_sx(&GRID_BASE_SX)
        .class(&props.class)
        .sx(&sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(
            props.component.copied_or(HtmlTag::Div),
            props.attributes,
            props.children,
        )
}
