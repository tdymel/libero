use dioxus::dioxus_core::AttributeValue;
use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, variables},
        inputs::ActionIcon,
        layout::use_box,
    },
    hooks::use_localization,
    sx::{REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        BURGER_COLOR, BURGER_LINE_SIZE, BURGER_SIZE, BURGER_SIZES, BURGER_TRANSITION_DURATION,
        BURGER_TRANSITION_TIMING, Size, SizeCss,
    },
    utils::warn,
};

/// One bar. The middle one *is* the glyph element; the outer two are its
/// `::before`/`::after`, so the whole thing is a single `<span>`.
///
/// `background-color` and `transform` only - never `all`. The middle bar
/// fading while the other two rotate is what makes the morph read as a morph
/// and not as a swap.
fn bar_sx() -> Sx {
    let transition = format!(
        "background-color {duration} {timing}, transform {duration} {timing}",
        duration = BURGER_TRANSITION_DURATION.value(),
        timing = BURGER_TRANSITION_TIMING.value(),
    );

    sx().display("block")
        .width(BURGER_SIZE.overridable())
        .height(BURGER_LINE_SIZE.value())
        // The fallback is the point: `sx().color("surface")` on the button
        // still reaches the bars, because nothing here overrides it.
        .background_color(BURGER_COLOR.value_or("currentColor"))
        // A `background-color` is not painted in forced-colors mode; a
        // transparent outline is, so the bars survive there. Costs no layout.
        .outline("1px solid transparent")
        .transition(transition)
        // The two states differ in shape, not only in position, so snapping
        // between them stays legible.
        .media(REDUCED_MOTION, sx().transition("none"))
}

static BURGER_GLYPH_SX: StaticSx = StaticSx::new(|| {
    let size = BURGER_SIZE.overridable();
    let down = format!("calc({size} / 3)");
    let up = format!("calc({size} / -3)");

    // Absolutely positioned against the middle bar, which is why that one is
    // `position: relative`.
    let outer = bar_sx().position("absolute").content("\"\"").left("0");

    bar_sx()
        // Declared here rather than in the theme: a custom property resolves
        // against the element it is declared on, and the caller's size
        // override lives on this element.
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

/// Mantine's formula, named: the button is the glyph plus one spacing step,
/// which is what makes the tap target bigger than the bars. `None` means the
/// caller named no size, so the theme's active step is the glyph.
fn button_size(glyph_size: Option<&String>) -> String {
    let glyph = match glyph_size {
        Some(size) => size.clone(),
        None => BURGER_SIZE.value(),
    };
    format!("calc({glyph} + {})", SizeCss::SPACING.value(Size::Xs))
}

/// A burger that announces itself as expanded while naming no panel. Almost
/// always an oversight - `aria-controls` is the caller's to spread (it rides
/// `GlobalAttributes`), so this is the only place that can notice it missing.
fn is_orphan_disclosure(open: Option<bool>, attributes: &[Attribute]) -> bool {
    open.is_some()
        && !attributes
            .iter()
            .any(|attribute| attribute.name == "aria-controls")
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

base_props! {
    pub struct BurgerProps {
        /// `Some(true)` draws the X and emits `aria-expanded="true"`.
        /// `None` emits no `aria-expanded` at all, so a `Burger` can open
        /// something that is not a disclosure. `Button::selected`'s rule.
        #[props(default)]
        open: Option<bool>,
        /// `Option`, not a bare `EventHandler` - see `Button`.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Replaces the localization's two labels,
        /// [`BurgerLabels`](crate::localization::BurgerLabels). Runs during
        /// render. A spread `"aria-label"` wins over both.
        #[props(default)]
        label: Option<Callback<bool, String>>,
        /// The glyph's width and height. The button around it is one spacing
        /// step larger.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// The bars. Unset, they are `currentColor`.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default)]
        disabled: Option<bool>,
    }
}

/// A three-bar button that morphs into an X: an [`ActionIcon`] with an
/// animated glyph and the three ARIA facts a disclosure needs.
///
/// It does not own `open` - the panel does, and the caller already holds
/// that signal to drive the panel itself. `aria-controls` rides
/// `GlobalAttributes`, so spread it:
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::Burger;
/// # fn app() -> Element {
/// # let mut open = use_signal(|| false);
/// # rsx! {
/// Burger {
///     open: open(),
///     "aria-controls": "site-nav",
///     onclick: move |_| open.toggle(),
/// }
/// # } }
/// ```
///
/// Focus stays on the burger when the panel opens. Moving it into the panel
/// is the panel's decision (`Drawer` traps already), and a burger that opens
/// a static sidebar must not steal focus.
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
        (None, None) => match open {
            true => labels.close.to_string(),
            false => labels.open.to_string(),
        },
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
        .render(HtmlTag::Span, Vec::new(), rsx! {});

    let onclick = props.onclick;

    rsx! {
        ActionIcon {
            aria_label,
            // Only when `open` is `Some`: without it this is a plain
            // button, which is what opening a modal wants.
            "aria-expanded": props.open.map(|open| open.to_string()),
            onclick: move |event| {
                if let Some(onclick) = onclick {
                    onclick.call(event);
                }
            },
            disabled: props.disabled,
            size: button_size,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes,
            {glyph}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;
    use crate::localization::BurgerLabels;

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

    /// The whole glyph is one element: the outer two bars are its pseudo-
    /// elements, and only `open` moves them.
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

    /// Only `background-color` and `transform` - a bare `all` would animate
    /// the layout the size override changes too.
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
        assert_eq!(BurgerLabels::ENGLISH.close, "Close navigation");
    }
}
