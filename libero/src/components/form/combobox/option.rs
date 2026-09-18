use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, base_props, forced_on_sx, inset_focus_ring_sx, on_start_bar_sx,
            on_tint_color, option_id,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, ComboboxDefaults, Size},
};

/// What the rows share with the `Combobox` around them. Signals, not values: a
/// provider runs once, so a plain field would freeze on the first render.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ComboboxContext {
    pub id: Signal<String>,
    pub size: Signal<Size>,
    pub radius: Signal<Size>,
    /// The highlighted row's `onpick`, so Enter can fire it.
    pub active_pick: Signal<Option<Callback<()>>>,
}

/// Where a row sits, provided per row so `ComboboxOption` needs no props to
/// know its own `id` or whether the arrows are on it.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ComboboxRowContext {
    pub index: usize,
    pub active: bool,
    /// The list refuses this row. It reaches the row the same way `active`
    /// does, so a caller's `option` callback needs to pass nothing on.
    pub disabled: bool,
}

/// One option, handed to `Combobox`'s `option` callback.
#[derive(Clone, PartialEq)]
pub struct ComboboxOptionArgs<T> {
    pub value: T,
    pub index: usize,
    /// Whether the arrow keys are on this row. `ComboboxOption` reads it for
    /// itself - this is for a row drawn without one.
    pub active: bool,
    /// Whether the list refuses this row. `ComboboxOption` reads it for itself
    /// too - this is for a row drawn without one, which then owes its reader
    /// both the greying and `aria-disabled`.
    pub disabled: bool,
}

const ACTIVE_RING_OFFSET: &str = "-2px";

static COMBOBOX_ROW_SX: StaticSx = StaticSx::new(|| {
    // The label colour made to read on the tints: 4.5:1 on either, hover included.
    let primary = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, ColorShade::S6));
    let bar = on_tint_color(&primary).unwrap_or_default();
    ComboboxDefaults::row_theme_vars()
        .display("flex")
        .align_items("center")
        .gap("8px")
        .width("100%")
        .cursor("pointer")
        .user_select("none")
        .white_space("nowrap")
        .overflow("hidden")
        // The row is a flex root, where `text-overflow` never applies, so a
        // bare-text row was cut mid-glyph (todo 94). A label in a span of its
        // own is a flex item, and so a block container the ellipsis works in.
        // Every default row wraps its text this way; a caller's row can too.
        .selector(
            "& > [data-slot='label']",
            sx().min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis"),
        )
        .hover(sx().background("muted.1"))
        .when("active", sx().background("muted.2"))
        // The shade's own contrast twin, not `primary.7`: blue text on a light
        // blue tint was hard to read. A blue start bar marks it, as `NavLink`'s:
        // a full ring would read as the active row's.
        .when(
            "selected",
            sx().background("primary.1")
                .color("primary-contrast.1")
                .and(on_start_bar_sx("0px", &bar))
                .and(forced_on_sx()),
        )
        // Folded *after* the selected tint, or it never lands: equal
        // specificity, so source order decides, and a selected row would
        // answer neither the mouse nor the keyboard. The bar again after the
        // `background`, which resets it.
        .when(
            "selected",
            sx().hover(
                sx().background("primary.2")
                    .and(on_start_bar_sx("0px", &bar)),
            ),
        )
        // The keyboard's own mark, on top of any background - a tint alone
        // cannot say "highlighted" on a row that is already tinted. Inset, so
        // it neither overlaps the row above nor is clipped by the dropdown.
        .when("active", inset_focus_ring_sx(ACTIVE_RING_OFFSET))
        // Every `active` arm folds into the first, ahead of the bar: this one
        // outranks it, moving the bar clear of the ring's stripe.
        .when(
            "selected",
            sx().when("active", on_start_bar_sx("2px", &bar)),
        )
        // Folded last, so it wins over the tints above it: a disabled row is
        // drawn and read out, and answers nothing. The hover has to be undone
        // by hand - a row the pointer cannot pick must not light up under it.
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .hover(sx().background("transparent")),
        )
});

base_props! {
    pub struct ComboboxOptionProps {
        /// The current selection - `aria-selected` and a tint. Leave it unset
        /// for a suggestion list, where "selected" means nothing.
        #[props(default)]
        selected: Option<bool>,
        /// Overrides the keyboard highlight, which otherwise comes from the
        /// `Combobox` drawing this row.
        #[props(default)]
        active: Option<bool>,
        /// Overrides whether the list refuses this row, which otherwise comes
        /// from the `Combobox` drawing it. A refused row is greyed and
        /// `aria-disabled`, and answers neither the click nor Enter.
        #[props(default)]
        disabled: Option<bool>,
        #[props(default)]
        onpick: Option<EventHandler<()>>,
        /// Row height and font size. Defaults to the `Combobox`'s own `size`.
        #[props(default, into)]
        size: Input<Size>,
        /// Corner radius. Defaults to the `Combobox`'s own, tightened by the
        /// dropdown's padding so the row nests inside it.
        #[props(default, into)]
        radius: Input<Size>,
        children: Element,
    }
}

/// A themed row for `Combobox`'s `option` callback. Registers itself as the
/// Enter target while it is the active row, and takes its `id` from the
/// `Combobox` so `aria-activedescendant` can point at it.
///
/// The row is a flex row, so bare text in it is cut at the edge. Put a long
/// label in `span { "data-slot": "label" }` and it ends in an ellipsis instead.
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
    // A disabled row is never the highlight, whatever the list thinks: the
    // core snaps the highlight off one already, and this is the second lock,
    // for a caller who sets `active` by hand.
    let active = !disabled
        && props
            .active
            .unwrap_or_else(|| row.is_some_and(|row| row().active));
    let id = combobox
        .zip(row)
        .map(|(combobox, row)| option_id(&(combobox.id)(), row().index));

    let onpick = props.onpick;
    // The one gate both the click and Enter go through: Enter fires this same
    // callback, which the active row registers as the `active_pick`.
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
        .attr_default("role", "option")
        .attr("aria-selected", props.selected.map(|on| on.to_string()))
        // Not `disabled`, which is no attribute on a `div` - and an option a
        // screen reader can still reach and read is the point of `aria-`.
        .attr("aria-disabled", disabled.then_some("true"))
        // Or the click blurs whatever the caller focused first.
        .event("onmousedown", move |event: MouseEvent| {
            event.prevent_default();
        })
        .event("onclick", move |_: MouseEvent| pick.call(()))
        .render(HtmlTag::Div, props.attributes, props.children)
}

/// A default row's text: the one element `COMBOBOX_ROW_SX` can ellipsise, since
/// the row itself is a flex root. `Select`, `MultiSelect`, `Autocomplete` and
/// `TagsField` draw their own rows through it.
pub(crate) fn row_label(label: String) -> Element {
    rsx! {
        span { "data-slot": "label", "{label}" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The ellipsis lives on the label, not on the row: `text-overflow` on a
    /// flex root never applies, and a bare-text row was cut mid-glyph.
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

    /// The greying beats the active and selected tints by **source order at
    /// equal specificity**, not by specificity, so it only works while the
    /// `disabled` fold is the last one. A reorder - or a merge conflict in
    /// this static, which happened once - would leave a disabled row tinted
    /// and pointer-hovering with every other test still green.
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
