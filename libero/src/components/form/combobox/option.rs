use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, States, base_props, forced_on_sx, inset_focus_ring_sx, option_id,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{ComboboxDefaults, Size},
};

use super::ComboboxPart;

/// Shared with the rows. Signals: a provider runs once, so plain fields would freeze.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ComboboxContext {
    pub id: Signal<String>,
    pub size: Signal<Size>,
    pub radius: Signal<Size>,
    /// The highlighted row's `onpick`, so Enter can fire it.
    pub active_pick: Signal<Option<Callback<()>>>,
}

/// Provided per row, so `ComboboxOption` needs no props for its `id`, highlight or `disabled`.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ComboboxRowContext {
    pub index: usize,
    pub active: bool,
    pub disabled: bool,
}

/// One option, handed to `Combobox`'s `option` callback.
#[derive(Clone, PartialEq)]
pub struct ComboboxOptionArgs<T> {
    pub value: T,
    pub index: usize,
    /// The arrow keys are on this row. For rows drawn without `ComboboxOption`.
    pub active: bool,
    /// The list refuses this row. A row without `ComboboxOption` owes the greying and `aria-disabled`.
    pub disabled: bool,
}

const ACTIVE_RING_OFFSET: &str = "-2px";

static COMBOBOX_ROW_SX: StaticSx = StaticSx::new(|| {
    ComboboxDefaults::row_theme_vars()
        .display("flex")
        .align_items("center")
        .gap("8px")
        .width("100%")
        .cursor("pointer")
        .user_select("none")
        .white_space("nowrap")
        .overflow("hidden")
        // `text-overflow` never applies on a flex root (todo 94); the label span carries the ellipsis.
        .selector(
            "& > [data-slot='label']",
            sx().min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis"),
        )
        .hover(sx().background("muted.1"))
        .when("active", sx().background("muted.2"))
        // The contrast twin, not `primary.7` (hard to read). A ring means active.
        .when(
            "selected",
            sx().background("primary.1")
                .color("primary-contrast.1")
                .and(forced_on_sx()),
        )
        // After the selected tint: equal specificity, so source order decides.
        .when("selected", sx().hover(sx().background("primary.2")))
        // A ring, since a tint can't mark an already tinted row. Inset, so the dropdown doesn't clip it.
        .when("active", inset_focus_ring_sx(ACTIVE_RING_OFFSET))
        // Last, so it beats the tints by source order. The hover is undone by hand.
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .hover(sx().background("transparent")),
        )
});

base_props! {
    pub struct ComboboxOptionProps {
        /// `aria-selected` and a tint. Leave unset for a suggestion list.
        #[props(default)]
        selected: Option<bool>,
        /// Overrides the keyboard highlight from the `Combobox`.
        #[props(default)]
        active: Option<bool>,
        /// Overrides the `Combobox`'s `disabled` for this row: greyed, `aria-disabled`, not pickable.
        #[props(default)]
        disabled: Option<bool>,
        #[props(default)]
        onpick: Option<EventHandler<()>>,
        /// Defaults to the `Combobox`'s `size`.
        #[props(default, into)]
        size: Input<Size>,
        /// Defaults to the `Combobox`'s, tightened by the dropdown's padding.
        #[props(default, into)]
        radius: Input<Size>,
        children: Element,
    }
}

/// A themed row for `Combobox`'s `option` callback; the active one is Enter's target.
///
/// Bare text is cut at the edge; wrap a long label in `span { "data-slot": "label" }` for an ellipsis.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::ComboboxOption;
/// # fn app() -> Element {
/// rsx! {
///     ComboboxOption { selected: true, onpick: move |_| {}, "Apple" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/combobox>
#[component]
pub fn ComboboxOption(props: ComboboxOptionProps) -> Element {
    let theme = use_theme();
    let combobox = try_consume_context::<ComboboxContext>();
    let row = try_consume_context::<Signal<ComboboxRowContext>>();

    let size = props.size.copied_or(match combobox {
        Some(context) => (context.size)(),
        None => theme.combobox.size,
    });
    let radius = props.radius.copied_or(match combobox {
        Some(context) => (context.radius)(),
        None => theme.combobox.radius,
    });

    let disabled = props
        .disabled
        .unwrap_or_else(|| row.is_some_and(|row| row().disabled));
    // Never the highlight when disabled, even with a hand-set `active`.
    let active = !disabled
        && props
            .active
            .unwrap_or_else(|| row.is_some_and(|row| row().active));
    let id = combobox
        .zip(row)
        .map(|(combobox, row)| option_id(&(combobox.id)(), row().index));

    let onpick = props.onpick;
    // The one gate for click and Enter; the active row registers it as `active_pick`.
    let pick = use_callback(move |()| {
        if disabled {
            return;
        }
        if let Some(onpick) = &onpick {
            onpick.call(());
        }
    });
    use_effect(use_reactive!(|active| {
        if let (true, Some(mut context)) = (active, combobox) {
            context.active_pick.set(Some(pick));
        }
    }));

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("active", active)
        .with("selected", props.selected.unwrap_or(false))
        .with("disabled", disabled)
        .into();

    use_box()
        .framework_sx(&COMBOBOX_ROW_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare()
        .attr_default("id", id)
        .attr_default("data-slot", ComboboxPart::Option.slot())
        .attr_default("role", "option")
        .attr("aria-selected", props.selected.map(|on| on.to_string()))
        // `disabled` is no `div` attribute, and the row must stay readable.
        .attr("aria-disabled", disabled.then_some("true"))
        // Or the click blurs whatever the caller focused first.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default();
        })
        .event("onclick", move |_: MouseEvent| pick.call(()))
        .render(HtmlTag::Div, props.attributes, props.children)
}

/// A default row's text, the element that can ellipsise. Shared by the pickers' own rows.
pub(crate) fn row_label(label: String) -> Element {
    rsx! {
        span { "data-slot": ComboboxPart::OptionLabel.slot(), "{label}" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The ellipsis lives on the label: `text-overflow` never applies on a flex root.
    #[test]
    fn the_label_slot_carries_the_ellipsis_and_the_row_does_not() {
        let css = Stylesheet::from(&COMBOBOX_ROW_SX);
        let css = css.as_str();
        let label = css.find("[data-slot='label']").expect("a label rule");
        let rule = &css[label..][..css[label..].find('}').unwrap()];
        for declaration in ["min-width:0", "overflow:hidden", "text-overflow:ellipsis"] {
            assert!(rule.contains(declaration), "{declaration} missing: {rule}");
        }
        assert_eq!(css.matches("text-overflow").count(), 1, "{css}");
    }

    /// The greying wins by source order at equal specificity, so `disabled` must fold last.
    /// A reorder (once, by a merge conflict) passes every other test.
    #[test]
    fn the_disabled_greying_folds_after_the_tints_it_has_to_beat() {
        let css = Stylesheet::from(&COMBOBOX_ROW_SX);
        let css = css.as_str();

        let greying = css.find("opacity:0.5").expect("the disabled greying");
        for state in ["active", "selected"] {
            let tint = css
                .rfind(&format!("[data-state~=\"{state}\"]"))
                .unwrap_or_else(|| panic!("a {state} rule: {css}"));
            assert!(
                greying > tint,
                "the disabled greying folds before the last {state} rule: {css}"
            );
        }
    }

    #[test]
    fn a_row_label_is_a_label_slot_holding_the_text() {
        assert_eq!(
            dioxus_ssr::render_element(row_label("Apple".to_string())),
            r#"<span data-slot="label">Apple</span>"#
        );
    }
}
