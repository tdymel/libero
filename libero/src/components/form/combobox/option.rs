use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, ROW_HOVER_TINT, ScaleOrCss, States, Variables, base_props,
            disabled_look_sx, forced_on_sx, group_id, inset_focus_ring_sx, option_id, variables,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ComboboxDefaults, Size, SizeCss},
};

use crate::components::form::DropdownPart;

/// Shared with the rows. Signals: a provider runs once, so plain fields would freeze.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ComboboxContext {
    pub id: Signal<String>,
    pub size: Signal<Size>,
    pub radius: Signal<ScaleOrCss>,
    /// Each drawn row's `onpick` by row key, so Enter fires the highlighted one. Not reactive.
    pub picks: CopyValue<HashMap<usize, Callback<()>>>,
}

/// Provided per row, so `ComboboxOption` needs no props for its `id`, highlight or `disabled`.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ComboboxRowContext {
    /// The row's stable key (its index without `row_keys`): names its `id` and pick slot.
    pub key: usize,
    pub active: bool,
    pub disabled: bool,
    /// A windowed list's `(aria-posinset, aria-setsize)`: the browser can't count unmounted rows.
    pub set: Option<(usize, usize)>,
    /// A windowed list's group label key: no `role="group"` wraps a window, so it describes the row.
    pub group: Option<usize>,
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
        // Hover and the keyboard's row share the tint; only the active row takes the ring.
        .hover(sx().background(ROW_HOVER_TINT))
        .when("active", sx().background(ROW_HOVER_TINT))
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
            disabled_look_sx("not-allowed").hover(sx().background("transparent")),
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
        /// A size word or any CSS, as `radius: "0"`.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
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
    let radius = match (props.radius.as_ref(), combobox) {
        (None, Some(context)) => (context.radius)(),
        (value, _) => ScaleOrCss::new(value, theme.combobox.radius),
    };
    let radius_variables: Input<Variables> = variables()
        .with(
            SizeCss::RADIUS.override_var(),
            radius.custom_css(SizeCss::RADIUS),
        )
        .into();

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
        .map(|(combobox, row)| option_id(&(combobox.id)(), row().key));
    let set = row.and_then(|row| row().set);
    let described_by = combobox
        .zip(row.and_then(|row| row().group))
        .map(|(combobox, group)| group_id(&(combobox.id)(), group));

    let onpick = props.onpick;
    // The one gate for click and Enter.
    let pick = use_callback(move |()| {
        if disabled {
            return;
        }
        if let Some(onpick) = &onpick {
            onpick.call(());
        }
    });
    // Registered in render by key: an Enter right after an arrow must not wait for this row
    // to re-render as the highlight (todo 1367).
    if let (Some(context), Some(row)) = (combobox, row) {
        let (mut picks, key) = (context.picks, row().key);
        if picks.peek().get(&key) != Some(&pick) {
            picks.write().insert(key, pick);
        }
    }
    use_drop(move || {
        if let Some(mut context) = combobox
            && let Ok(mut picks) = context.picks.try_write()
        {
            picks.retain(|_, registered| *registered != pick);
        }
    });

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.size.radius_state_name(), true)
        .with("active", active)
        .with("selected", props.selected.unwrap_or(false))
        .with("disabled", disabled)
        .into();

    use_box()
        .framework_sx(&COMBOBOX_ROW_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&radius_variables)
        .prepare()
        .attr_default("id", id)
        .attr_default("data-slot", DropdownPart::Option.slot())
        .attr_default("role", "option")
        .attr("aria-selected", props.selected.map(|on| on.to_string()))
        // `disabled` is no `div` attribute, and the row must stay readable.
        .attr("aria-disabled", disabled.then_some("true"))
        .attr("aria-posinset", set.map(|(at, _)| at.to_string()))
        .attr("aria-setsize", set.map(|(_, size)| size.to_string()))
        .attr_default("aria-describedby", described_by)
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
        span { "data-slot": DropdownPart::OptionLabel.slot(), "{label}" }
    }
}

/// Drawn rows by key, handed out again while their inputs compare equal: the dropdown then gets
/// the very same `Element`, so an unchanged `ComboboxRow` skips its render (todo 2022).
pub(crate) struct RowCache<I: 'static>(Rc<RefCell<HashMap<usize, (I, Element)>>>);

impl<I: 'static> Clone for RowCache<I> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub(crate) fn use_row_cache<I: 'static>() -> RowCache<I> {
    use_hook(RowCache::default)
}

impl<I: 'static> Default for RowCache<I> {
    fn default() -> Self {
        Self(Rc::default())
    }
}

impl<I: PartialEq + 'static> RowCache<I> {
    /// The rows in order. `draw` is a `fn`, so a row depends on `inputs` alone and every change
    /// redraws it; a key missing from one call is forgotten.
    pub(crate) fn rows(
        &self,
        rows: impl IntoIterator<Item = (usize, I)>,
        draw: fn(usize, &I) -> Element,
    ) -> Vec<Element> {
        let mut old = self.0.take();
        let mut kept = HashMap::with_capacity(old.len());
        let drawn = rows
            .into_iter()
            .map(|(key, inputs)| {
                let element = match old.remove(&key) {
                    Some((cached, element)) if cached == inputs => element,
                    _ => draw(key, &inputs),
                };
                kept.insert(key, (inputs, element.clone()));
                element
            })
            .collect();
        *self.0.borrow_mut() = kept;
        drawn
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

    fn draw_label(key: usize, label: &String) -> Element {
        rsx! {
            span { "{key}: {label}" }
        }
    }

    fn rows(cache: &RowCache<String>, labels: &[(usize, &str)]) -> Vec<Element> {
        cache.rows(
            labels.iter().map(|(key, label)| (*key, label.to_string())),
            draw_label,
        )
    }

    /// The same `Element`, pointer-equal, is what lets the dropdown's row skip.
    #[test]
    fn an_unchanged_row_is_handed_out_again() {
        let cache = RowCache::default();
        let first = rows(&cache, &[(0, "Apple"), (1, "Pear")]);
        let second = rows(&cache, &[(1, "Pear"), (0, "Apple")]);
        assert!(first[0] == second[1] && first[1] == second[0]);
    }

    #[test]
    fn a_changed_input_redraws_only_that_row() {
        let cache = RowCache::default();
        let first = rows(&cache, &[(0, "Apple"), (1, "Pear")]);
        let second = rows(&cache, &[(0, "Apple"), (1, "Plum")]);
        assert!(first[0] == second[0]);
        assert!(first[1] != second[1]);
        assert_eq!(
            dioxus_ssr::render_element(second[1].clone()),
            "<span>1: Plum</span>"
        );
    }

    /// A row filtered out is forgotten, so it never comes back drawn from stale inputs.
    #[test]
    fn a_row_missing_from_one_call_is_redrawn_when_it_returns() {
        let cache = RowCache::default();
        let first = rows(&cache, &[(0, "Apple"), (1, "Pear")]);
        rows(&cache, &[(1, "Pear")]);
        let third = rows(&cache, &[(0, "Apple"), (1, "Pear")]);
        assert!(first[0] != third[0]);
        assert!(first[1] == third[1]);
    }

    #[test]
    fn a_row_label_is_a_label_slot_holding_the_text() {
        assert_eq!(
            dioxus_ssr::render_element(row_label("Apple".to_string())),
            r#"<span data-slot="label">Apple</span>"#
        );
    }
}
