use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{base_props, focus_ring_sx},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, sx},
    theme::{ComboboxDefaults, Size},
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
}

/// One option, handed to `Combobox`'s `option` callback.
#[derive(Clone, PartialEq)]
pub struct ComboboxOptionArgs<T> {
    pub value: T,
    pub index: usize,
    /// Whether the arrow keys are on this row. `ComboboxOption` reads it for
    /// itself - this is for a row drawn without one.
    pub active: bool,
}

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
        .hover(sx().background("grey.1"))
        .when("active", sx().background("grey.2"))
        // The shade's own contrast twin, not `primary.7`: blue text on a light
        // blue tint was hard to read.
        .when(
            "selected",
            sx().background("primary.1").color("primary-contrast.1"),
        )
        // Folded *after* the selected tint, or it never lands: equal
        // specificity, so source order decides, and a selected row would
        // answer neither the mouse nor the keyboard.
        .when("selected", sx().hover(sx().background("primary.2")))
        // The keyboard's own mark, on top of any background - a tint alone
        // cannot say "highlighted" on a row that is already tinted. Inset, so
        // it neither overlaps the row above nor is clipped by the dropdown.
        .when("active", focus_ring_sx().outline_offset("-2px"))
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

    let active = props
        .active
        .unwrap_or_else(|| row.is_some_and(|row| row().active));
    let id = combobox
        .zip(row)
        .map(|(combobox, row)| super::aria::option_id(&(combobox.id)(), row().index));

    let onpick = props.onpick;
    let pick = use_callback(move |()| {
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

    #[test]
    fn a_row_label_is_a_label_slot_holding_the_text() {
        assert_eq!(
            dioxus_ssr::render_element(row_label("Apple".to_string())),
            r#"<span data-slot="label">Apple</span>"#
        );
    }
}
