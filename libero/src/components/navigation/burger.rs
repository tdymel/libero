use dioxus::dioxus_core::AttributeValue;
use dioxus::prelude::*;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            HtmlTag, Input, Part, States, Variables, base_props, parts_enum, parts_under_sx,
            variables,
        },
        layout::use_box,
    },
    hooks::use_localization,
    localization::BurgerLabels,
    sx::{REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        BURGER_COLOR, BURGER_LINE_SIZE, BURGER_SIZE, BURGER_SIZES, BURGER_TRANSITION_DURATION,
        BURGER_TRANSITION_TIMING, Size, SizeCss,
    },
    utils::warn,
};

/// One bar: the middle one is the glyph `<span>`, the outer two its pseudo-elements.
/// Never `transition: all`: the middle bar fades while the others rotate.
fn bar_sx() -> Sx {
    let transition = format!(
        "background-color {duration} {timing}, transform {duration} {timing}",
        duration = BURGER_TRANSITION_DURATION.value(),
        timing = BURGER_TRANSITION_TIMING.value(),
    );

    sx().display("block")
        .width(BURGER_SIZE.overridable())
        .height(BURGER_LINE_SIZE.value())
        // `currentColor`, so a colour set on the button still reaches the bars.
        .background_color(BURGER_COLOR.value_or("currentColor"))
        // Forced colours drop the background but paint an outline.
        .outline("1px solid transparent")
        .transition(transition)
        .media(REDUCED_MOTION, sx().transition("none"))
}

static BURGER_GLYPH_SX: StaticSx = StaticSx::new(|| {
    let size = BURGER_SIZE.overridable();
    let down = format!("calc({size} / 3)");
    let up = format!("calc({size} / -3)");

    let outer = bar_sx().position("absolute").content("\"\"").left("0");

    bar_sx()
        // Here, not in the theme: it must resolve against this element's size override.
        .var(BURGER_LINE_SIZE, format!("calc({size} / 12)"))
        .position("relative")
        .selector("&::before", outer.clone().top(up.clone()))
        .selector("&::after", outer.top(down.clone()))
        .when(
            "open",
            sx().background_color("transparent")
                .selector(
                    "&::before",
                    sx().transform(format!("translateY({down}) rotate(45deg)")),
                )
                .selector(
                    "&::after",
                    sx().transform(format!("translateY({up}) rotate(-45deg)")),
                ),
        )
});

/// The glyph plus one spacing step, for a tap target bigger than the bars.
fn button_size(glyph_size: Option<&String>) -> String {
    let glyph = match glyph_size {
        Some(size) => size.clone(),
        None => BURGER_SIZE.value(),
    };
    format!("calc({glyph} + {})", SizeCss::SPACING.value(Size::Xs))
}

/// A burger announcing `aria-expanded` with no spread `aria-controls`: almost
/// always an oversight.
fn is_orphan_disclosure(open: Option<bool>, attributes: &[Attribute]) -> bool {
    open.is_some()
        && !attributes
            .iter()
            .any(|attribute| attribute.name == "aria-controls")
}

/// Static while `aria-expanded` carries the state, so it is not announced
/// twice (APG disclosure). Unset `open` names the action instead.
fn default_label(labels: &BurgerLabels, open: Option<bool>) -> &'static str {
    match open {
        Some(_) => labels.toggle,
        None => labels.open,
    }
}

/// Removes a spread text `aria-label` and returns it.
fn take_spread_label(attributes: &mut Vec<Attribute>) -> Option<String> {
    let index = attributes.iter().position(|attribute| {
        attribute.name == "aria-label" && matches!(attribute.value, AttributeValue::Text(_))
    })?;
    match attributes.remove(index).value {
        AttributeValue::Text(label) => Some(label),
        _ => None,
    }
}

parts_enum! {
    /// [`Burger`]'s inner parts, for its `parts` prop.
    pub enum BurgerPart {
        /// The middle bar; the outer two are its `::before` and `::after`.
        Glyph = "glyph" => "& > [data-slot='glyph']",
    }
}

base_props! {
    parts(BurgerPart);
    pub struct BurgerProps {
        /// `Some(true)` draws the X; `None` emits no `aria-expanded`, for a non-disclosure.
        #[props(default)]
        open: Option<bool>,
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// The accessible name from the open state, replacing [`BurgerLabels`].
        #[props(default)]
        label: Option<Callback<bool, String>>,
        /// The glyph's width and height; the button is one spacing step larger.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// The bars. Unset, they are `currentColor`.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default)]
        disabled: Option<bool>,
    }
}

/// A three-bar menu button that morphs into an X, a disclosure for the panel
/// named by a spread `aria-controls`.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Burger;
/// # fn app() -> Element {
/// let mut open = use_signal(|| false);
/// rsx! {
///     Burger {
///         open: open(),
///         "aria-controls": "site-nav",
///         onclick: move |_| open.toggle(),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/navigation/burger>
#[component]
pub fn Burger(props: BurgerProps) -> Element {
    let labels = use_localization().burger;
    let open = props.open.unwrap_or(false);

    // `ActionIcon` writes its own `aria-label` over a spread one, so it moves over here.
    let mut attributes = props.attributes;
    let spread_label = take_spread_label(&mut attributes);
    let aria_label = match (spread_label, props.label) {
        (Some(label), _) => label,
        (None, Some(label)) => label.call(open),
        (None, None) => default_label(&labels, props.open).to_string(),
    };

    if is_orphan_disclosure(props.open, &attributes) {
        warn(
            "Burger: `open` is set but no `aria-controls` was spread, so nothing says which panel this button expands.",
        );
    }

    let glyph_size = props.size.resolve(Some(BURGER_SIZES));
    let button_size = button_size(glyph_size.as_ref());

    let glyph_variables: Input<Variables> = variables()
        .with(BURGER_SIZE.override_var(), glyph_size)
        .with(BURGER_COLOR, props.color.resolve(None))
        .into();
    let glyph_states: Input<States> = States::default().with("open", open).into();

    let glyph = use_box()
        .framework_sx(&BURGER_GLYPH_SX)
        .states(&glyph_states)
        .variables(&glyph_variables)
        .prepare()
        .attr("data-slot", BurgerPart::Glyph.slot())
        .render(HtmlTag::Span, Vec::new(), rsx! {});

    let onclick = props.onclick;

    rsx! {
        ActionIcon {
            aria_label,
            // Only when `open` is `Some`: otherwise a plain button, as a modal wants.
            "aria-expanded": props.open.map(|open| open.to_string()),
            onclick: move |event| {
                if let Some(onclick) = onclick {
                    onclick.call(event);
                }
            },
            disabled: props.disabled,
            size: button_size,
            class: props.class,
            sx: parts_under_sx(&props.parts, props.sx),
            states: props.states,
            attributes,
            {glyph}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::common::part_table, css::Stylesheet};

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<BurgerPart>(),
            [("glyph", "& > [data-slot='glyph']")]
        );
    }

    #[test]
    fn an_unsized_burger_grows_the_button_around_the_themed_glyph() {
        assert_eq!(
            button_size(None),
            "calc(var(--lsx-burger-size) + var(--lsx-spacing-xs))"
        );
        assert_eq!(
            button_size(Some(&"var(--lsx-burger-size-lg)".to_string())),
            "calc(var(--lsx-burger-size-lg) + var(--lsx-spacing-xs))"
        );
    }

    /// The `color("white")` trap: the bars fall back to `currentColor`, so a
    /// caller styling the *button* still reaches them.
    #[test]
    fn the_bars_fall_back_to_current_color() {
        let css = Stylesheet::from(&BURGER_GLYPH_SX).as_str().to_string();
        assert!(
            css.contains("background-color:var(--lsx-burger-color, currentColor)"),
            "{css}"
        );
    }

    /// The outer two bars are the glyph's pseudo-elements; only `open` moves them.
    #[test]
    fn opened_fades_the_middle_bar_and_rotates_the_outer_two() {
        let css = Stylesheet::from(&BURGER_GLYPH_SX).as_str().to_string();

        for rule in [
            "[data-state~=\"open\"]{background-color:transparent;}",
            "[data-state~=\"open\"]::before{transform:translateY(",
            "[data-state~=\"open\"]::after{transform:translateY(",
        ] {
            assert!(css.contains(rule), "missing {rule} in {css}");
        }
    }

    /// A bare `all` would also animate the layout the size override changes.
    #[test]
    fn the_transition_names_its_two_properties_and_reduced_motion_drops_it() {
        let css = Stylesheet::from(&BURGER_GLYPH_SX).as_str().to_string();

        assert!(css.contains("transition:background-color var("), "{css}");
        assert!(!css.contains("transition:all"), "{css}");
        assert!(
            css.contains("@media (prefers-reduced-motion: reduce)"),
            "{css}"
        );
    }

    #[test]
    fn only_an_opened_burger_with_no_aria_controls_warns() {
        let names = |name: &'static str| {
            vec![Attribute::new(
                name,
                AttributeValue::Text("nav".to_string()),
                None,
                false,
            )]
        };

        // Unset `open` is not a disclosure, so it owes nothing.
        assert!(!is_orphan_disclosure(None, &[]));
        assert!(!is_orphan_disclosure(None, &names("aria-controls")));
        // `Some(false)` still emits `aria-expanded`, so it does.
        assert!(is_orphan_disclosure(Some(false), &[]));
        assert!(is_orphan_disclosure(Some(true), &[]));
        assert!(is_orphan_disclosure(Some(true), &names("aria-label")));
        assert!(!is_orphan_disclosure(Some(true), &names("aria-controls")));
    }

    #[test]
    fn a_spread_aria_label_is_taken_out_of_the_attributes() {
        let text = |name: &'static str, value: &str| {
            Attribute::new(name, AttributeValue::Text(value.to_string()), None, false)
        };
        let mut attributes = vec![text("aria-controls", "nav"), text("aria-label", "Menu")];

        assert_eq!(take_spread_label(&mut attributes).as_deref(), Some("Menu"));
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].name, "aria-controls");
        assert_eq!(take_spread_label(&mut attributes), None);
    }

    #[test]
    fn the_labels_are_a_locale_struct_of_their_own() {
        assert_eq!(BurgerLabels::ENGLISH.open, "Open navigation");
        assert_eq!(BurgerLabels::ENGLISH.toggle, "Toggle navigation");
    }

    #[test]
    fn a_set_open_names_the_toggle_and_an_unset_one_the_action() {
        let labels = BurgerLabels::ENGLISH;
        assert_eq!(default_label(&labels, Some(true)), "Toggle navigation");
        assert_eq!(default_label(&labels, Some(false)), "Toggle navigation");
        assert_eq!(default_label(&labels, None), "Open navigation");
    }
}
