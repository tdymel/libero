use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_props, input_from_str, responsive_sx,
            variables, with_own,
        },
        layout::use_box,
    },
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        FLEX_ALIGN_VAR, FLEX_JUSTIFY_VAR, FLEX_WRAP_VAR, FlexDefaults, Responsive, Size, SizeCss,
    },
};

str_enum! {
    pub enum FlexDirection {
        Row = "row",
        #[default]
        Column = "column",
        /// A row drawn right to left. Only the look changes: reading order, Tab order and
        /// a screen reader's order stay the source order (WCAG 1.3.2, 2.4.3).
        RowReverse = "row-reverse",
        /// A column drawn bottom to top, with the same catch as `RowReverse`.
        ColumnReverse = "column-reverse",
    }
}

impl FlexDirection {
    /// Whether the main axis runs along a row, which takes the row axis defaults.
    pub fn is_row(self) -> bool {
        matches!(self, Self::Row | Self::RowReverse)
    }
}

input_from_str!(FlexDirection);

/// One direction, or one per breakpoint: `direction: responsive(FlexDirection::Column).md(FlexDirection::Row)`.
impl From<&str> for Input<Responsive<FlexDirection>> {
    fn from(value: &str) -> Self {
        Self::Value(Responsive::new(value.into()))
    }
}

impl From<String> for Input<Responsive<FlexDirection>> {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<FlexDirection> for Input<Responsive<FlexDirection>> {
    fn from(value: FlexDirection) -> Self {
        Self::Value(Responsive::new(value))
    }
}

impl From<Option<FlexDirection>> for Input<Responsive<FlexDirection>> {
    fn from(value: Option<FlexDirection>) -> Self {
        value.map_or(Self::None, Into::into)
    }
}

impl From<Responsive<FlexDirection>> for Input<Responsive<FlexDirection>> {
    fn from(value: Responsive<FlexDirection>) -> Self {
        Self::Value(value)
    }
}

str_enum! {
    pub enum FlexWrap {
        Wrap = "wrap",
        #[default]
        NoWrap = "nowrap",
        /// Wraps, the new lines stacking before the first: bottom to top in a row.
        WrapReverse = "wrap-reverse",
    }
}

impl From<bool> for FlexWrap {
    fn from(value: bool) -> Self {
        if value { Self::Wrap } else { Self::NoWrap }
    }
}

impl From<bool> for Input<FlexWrap> {
    fn from(value: bool) -> Self {
        Input::Value(FlexWrap::from(value))
    }
}

input_from_str!(FlexWrap);

// `gap` folds in after `row`, so it beats either axis's default spacing.
// The per-instance vars reset to `initial` so a nested Flex does not inherit them.
static FLEX_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .display("flex")
        .var(FLEX_ALIGN_VAR, "initial")
        .var(FLEX_JUSTIFY_VAR, "initial")
        .var(FLEX_WRAP_VAR, "initial")
        .and(FlexDefaults::default_sx(false))
        .when("row", FlexDefaults::default_sx(true))
        // After `row`, which sets the plain direction.
        .when("row-reverse", sx().flex_direction("row-reverse"))
        .when("column-reverse", sx().flex_direction("column-reverse"));

    Size::ALL.into_iter().fold(base, |base, size| {
        base.when(size.state_name(), sx().gap(SizeCss::SPACING.value(size)))
    })
});

fn flex_variables(props: &FlexProps) -> Variables {
    variables()
        .with(FLEX_ALIGN_VAR, props.align.resolve(None))
        .with(FLEX_JUSTIFY_VAR, props.justify.resolve(None))
        .with(
            FLEX_WRAP_VAR,
            props.wrap.as_ref().map(|wrap| wrap.as_str().to_string()),
        )
}

base_props! {
    pub struct FlexProps {
        /// Which element to render as - `div` by default, as `component: HtmlTag::Ul` for a list.
        #[props(default, into)]
        component: Input<HtmlTag>,
        /// `align-items`.
        #[props(default, into)]
        align: Input<ThemeAwareValue>,
        /// `justify-content`.
        #[props(default, into)]
        justify: Input<ThemeAwareValue>,
        /// A size word or any CSS, as `gap: "sm"`, `gap: "0"` or one per breakpoint,
        /// `gap: responsive(Size::Xs).md(Size::Lg)`. Breakpoints follow the window.
        #[props(default, into)]
        gap: Input<Responsive<ThemeAwareValue>>,
        /// `column` by default, or one per breakpoint,
        /// `direction: responsive(FlexDirection::Column).md(FlexDirection::Row)`.
        /// `row-reverse` and `column-reverse` reorder the look only, not the reading or Tab order.
        #[props(default, into)]
        direction: Input<Responsive<FlexDirection>>,
        /// `true` or `FlexWrap::Wrap` wraps; `FlexWrap::WrapReverse` stacks the new lines before the first.
        #[props(default, into)]
        wrap: Input<FlexWrap>,
        children: Element,
    }
}

fn axis_sx(direction: FlexDirection) -> Sx {
    let axis = FlexDefaults::default_sx(direction.is_row());
    match direction {
        FlexDirection::Row | FlexDirection::Column => axis,
        reverse => axis.flex_direction(reverse.as_str()),
    }
}

/// A gap off the size scale and every breakpoint, plus each direction breakpoint's axis
/// defaults, in the user layer so they beat the state classes; before `sx`, so it still wins.
fn layout_sx(
    direction: Option<&Responsive<FlexDirection>>,
    gap: Option<&Responsive<ThemeAwareValue>>,
    own: &Input<Sx>,
) -> Input<Sx> {
    let turns: Vec<_> = direction.map_or(Vec::new(), |direction| direction.breakpoints().collect());
    let Some(gap) = gap else {
        if turns.is_empty() {
            return own.clone();
        }
        let axes = turns.into_iter().fold(sx(), |base, (size, turn)| {
            base.breakpoint(size, axis_sx(turn))
        });
        return with_own(Some(axes), own);
    };
    if turns.is_empty() {
        return with_own(responsive_sx(gap, Sx::gap), own);
    }

    // One ascending pass, so a wider breakpoint's rule still comes last.
    let base = responsive_sx(&Responsive::new(gap.base_ref().clone()), Sx::gap).unwrap_or_default();
    let axes = Size::ALL.into_iter().fold(base, |base, size| {
        let rule = match turns.iter().find(|(at, _)| *at == size) {
            // The axis defaults carry the theme's spacing: the explicit gap goes back on top.
            Some((_, turn)) => axis_sx(*turn).gap(gap.at(size)),
            None => match gap.breakpoints().find(|(at, _)| *at == size) {
                Some((_, value)) => sx().gap(value),
                None => return base,
            },
        };
        base.breakpoint(size, rule)
    });
    with_own(Some(axes), own)
}

/// Lays its children out in a column or a row, with a themed gap.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Flex};
/// # fn app() -> Element {
/// rsx! {
///     Flex { direction: "row", gap: "sm",
///         Button { "Save" }
///         Button { "Cancel" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/flex>
#[component]
pub fn Flex(props: FlexProps) -> Element {
    let direction = props
        .direction
        .as_ref()
        .map_or(FlexDirection::default(), Responsive::base);
    let variables: Input<Variables> = flex_variables(&props).into();

    let mut states = props
        .states
        .unwrap_or_default()
        .with("row", direction.is_row())
        .with("row-reverse", direction == FlexDirection::RowReverse)
        .with("column-reverse", direction == FlexDirection::ColumnReverse);
    if let Some(ThemeAwareValue::Size(gap)) = props.gap.as_ref().map(Responsive::base_ref) {
        states = states.with(gap.state_name(), true);
    }
    let states: Input<States> = states.into();
    let sx = layout_sx(props.direction.as_ref(), props.gap.as_ref(), &props.sx);

    use_box()
        .framework_sx(&FLEX_BASE_SX)
        .class(&props.class)
        .sx(&sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(
            props.component.copied_or_default(),
            props.attributes,
            props.children,
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_wrap_parses_bools_and_strings() {
        assert_eq!(FlexWrap::from(true).as_str(), "wrap");
        assert_eq!(FlexWrap::from(false).as_str(), "nowrap");
        assert_eq!(FlexWrap::from("wrap"), FlexWrap::Wrap);
        assert_eq!(FlexWrap::from("wrap-reverse"), FlexWrap::WrapReverse);
        assert_eq!(FlexWrap::from("nonsense"), FlexWrap::NoWrap);
    }

    #[test]
    fn direction_takes_the_enum_as_well_as_a_string() {
        let direction: Input<FlexDirection> = FlexDirection::Row.into();
        assert_eq!(direction, Input::from("row"));
    }

    fn flex_props(wrap: Input<FlexWrap>, align: Input<ThemeAwareValue>) -> FlexProps {
        FlexProps {
            component: Input::None,
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            align,
            justify: Input::None,
            gap: Input::None,
            direction: Input::None,
            wrap,
            children: rsx! {},
        }
    }

    #[test]
    fn wrap_is_emitted_as_its_css_keyword() {
        let props = flex_props(FlexWrap::NoWrap.into(), Input::None);

        assert_eq!(
            flex_variables(&props).to_string(),
            format!("{}:nowrap;", FLEX_WRAP_VAR.name())
        );
    }

    #[test]
    fn only_the_set_properties_are_emitted() {
        let props = flex_props(Input::None, "center".into());
        let variables = flex_variables(&props).to_string();

        assert_eq!(variables, format!("{}:center;", FLEX_ALIGN_VAR.name()));
    }

    #[test]
    fn nothing_set_emits_nothing() {
        assert_eq!(
            flex_variables(&flex_props(Input::None, Input::None)).to_string(),
            ""
        );
    }
}
