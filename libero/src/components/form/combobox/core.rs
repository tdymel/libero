use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box, surface::paper_sx},
    hooks::{ElementHandle, PopoverOptions, PopoverWidth, use_element, use_popover, use_theme},
    platform::ElementApi,
    sx::StaticSx,
    theme::{COMBOBOX_PADDING, Size, SizeCss, Z_INDEX_POPOVER},
};

use super::{dropdown::ComboboxDropdown, option::ComboboxContext};

// The dropdown is a surface, so its background, border and corner come from
// `paper_sx()`: the `bordered` and `radius-{step}` tokens below are the ones
// its folds answer. It renders through `use_box` rather than `Paper` because
// it needs the popover's element handle and its own events.
pub(crate) static COMBOBOX_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        // Everything positional - `position`, `left`, `top`, `width` - comes
        // from `use_popover` as an inline style, measured per open.
        .z_index(Z_INDEX_POPOVER.value())
        .display("flex")
        .flex_direction("column")
        .gap("4px")
        .padding(COMBOBOX_PADDING)
        // A row still has square-ish corners next to an `xxl` radius, so the
        // dropdown clips rather than trusting them to nest.
        .overflow("hidden")
        // A dropdown floats over the page, where the surface default rests.
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
});

base_props! {
    pub(crate) struct ComboboxCoreProps {
        /// The rows, already drawn - which is what erases the caller's `T`,
        /// and what stops anything below here from memoizing.
        rows: Vec<Element>,
        /// The highlighted row, or `None` for no highlight - which is what an
        /// autocomplete opens with, so Enter commits the typed text instead of
        /// the top suggestion.
        active: Option<usize>,
        onactive: EventHandler<usize>,
        opened: bool,
        onopened: EventHandler<bool>,
        /// The `ComboboxState`'s id, which the aria wiring is built from.
        id: String,
        #[props(default)]
        empty: Option<Element>,
        /// `Some(label)` while the options are being fetched: the dropdown
        /// shows a labelled `Loader` in place of the rows and `empty`.
        #[props(default)]
        loading: Option<String>,
        /// Above the rows, inside the dropdown and outside its scroll - a
        /// search box. It survives an empty list, which is the whole point:
        /// a query that matches nothing is exactly when the box has to still
        /// be there to edit.
        #[props(default)]
        header: Option<Element>,
        /// Focused once the list is actually on screen.
        ///
        /// The box is `visibility: hidden` until `use_popover` has measured it,
        /// and focusing a hidden element does nothing while still reporting
        /// success - so a caller that focuses its own header the moment it
        /// mounts silently loses the focus every time. Only this component
        /// knows when the box became visible.
        #[props(default)]
        autofocus: Option<ElementHandle>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        radius: Input<Size>,
        disabled: bool,
        /// Whether Enter closes the list after picking. A multi-select keeps
        /// it open, so each pick toggles a row and the next one is one key
        /// away.
        #[props(default = true)]
        close_on_pick: bool,
        /// Sets `aria-multiselectable` on the listbox.
        #[props(default)]
        multiselectable: bool,
        /// What the list's width follows. `Match` reproduces the `width: 100%`
        /// it had while it was nested; a select, whose rows are the content,
        /// takes `Min` so a long row is never clipped.
        #[props(default = PopoverWidth::Match)]
        width: PopoverWidth,
        /// Measures the list again whenever it changes, for a trigger that
        /// resizes while the list is open.
        #[props(default)]
        remeasure: u64,
        children: Element,
    }
}

#[component]
pub(crate) fn ComboboxCore(props: ComboboxCoreProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.combobox.size);
    let radius = props.radius.copied_or(theme.combobox.radius);

    // Owned by the root scope, not by this one, and dropped by hand on unmount.
    // The rows that read them are portaled, so they mount under `PortalOutlet`
    // and are *not* descendants of this component - a signal created here would
    // be read from outside the scope that owns it, which dioxus warns about and
    // which really can drop the value while a row still holds it.
    let id = use_hook(|| Signal::new_in_scope(props.id.clone(), ScopeId::ROOT));
    let mut shared_size = use_hook(|| Signal::new_in_scope(size, ScopeId::ROOT));
    if *shared_size.peek() != size {
        shared_size.set(size);
    }
    let mut shared_radius = use_hook(|| Signal::new_in_scope(radius, ScopeId::ROOT));
    if *shared_radius.peek() != radius {
        shared_radius.set(radius);
    }
    let active_pick = use_hook(|| Signal::new_in_scope(None, ScopeId::ROOT));
    use_drop(move || {
        id.manually_drop();
        shared_size.manually_drop();
        shared_radius.manually_drop();
        active_pick.manually_drop();
    });
    let context = ComboboxContext {
        id,
        size: shared_size,
        radius: shared_radius,
        active_pick,
    };
    // Provided here for a row drawn inside the trigger, and handed to the
    // portaled dropdown as a prop, which mounts outside this scope entirely.
    use_context_provider(|| context);

    let opened = props.opened;
    let loading = props.loading.is_some();
    // The rows belong to the previous query while a fetch runs, so they are
    // neither drawn nor reachable by the arrows.
    let count = if loading { 0 } else { props.rows.len() };
    let active_row = props
        .active
        .filter(|_| count > 0)
        .map(|row| row.min(count - 1));
    let set_active = props.onactive;

    let disabled = props.disabled;
    let close_on_pick = props.close_on_pick;
    let onopened = props.onopened;
    let request = move |next: bool| onopened.call(next);

    let onkeydown = move |event: KeyboardEvent| {
        if disabled {
            return;
        }
        let last = count.saturating_sub(1);
        let go_to = |row: usize| {
            set_active.call(row);
            if !opened {
                request(true);
            }
        };

        match event.key() {
            // From no highlight, both arrows land on row 0 - the first press
            // arms the list rather than moving inside it.
            Key::ArrowDown => {
                event.prevent_default();
                match (opened, active_row) {
                    (true, Some(row)) => go_to((row + 1).min(last)),
                    _ => go_to(active_row.unwrap_or(0)),
                }
            }
            Key::ArrowUp if opened => {
                event.prevent_default();
                go_to(active_row.map_or(0, |row| row.saturating_sub(1)));
            }
            Key::Home if opened => {
                event.prevent_default();
                go_to(0);
            }
            Key::End if opened => {
                event.prevent_default();
                go_to(last);
            }
            // Nothing highlighted means Enter is not ours: it bubbles, so a
            // form still submits.
            Key::Enter if opened && active_row.is_some() => {
                event.prevent_default();
                if let Some(pick) = active_pick() {
                    pick.call(());
                }
                if close_on_pick {
                    request(false);
                }
            }
            Key::Escape if opened => {
                event.prevent_default();
                request(false);
            }
            Key::Tab if opened => request(false),
            _ => {}
        }
    };

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("bordered", true)
        .with("disabled", disabled)
        .into();

    // Every one of these is a hook, so all of them run before anything branches
    // on `opened`.
    let anchor = use_element();
    let popover = use_popover(
        anchor,
        opened,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            // Portaling takes away the positioned wrapper a `width: 100%` used
            // to resolve against, so the width is measured instead.
            .width(props.width)
            .remeasure(props.remeasure),
    );
    // Unstyled, and not user-facing: it is only what the keys are caught on and
    // what the dropdown is measured against, which is why `sx` lands on the
    // dropdown instead. Its width is the trigger's, so `PopoverWidth::Match`
    // reproduces the `width: 100%` the dropdown had while it was nested.
    let wrapper = use_box().prepare();
    let dropdown = use_box()
        .framework_sx(&COMBOBOX_DROPDOWN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(popover.style())
        .prepare();

    // Placed means measured, which means visible - the first moment at which
    // focusing anything inside the box can take.
    let autofocus = props.autofocus;
    let placed = popover.placed();
    use_effect(use_reactive!(|(opened, placed)| {
        if let (true, true, Some(target)) = (opened, placed, autofocus) {
            // Out of this dispatch: the click that opened the list ends by
            // focusing the trigger, so focusing inline is undone a moment
            // later.
            spawn(async move {
                let _ = target.focus();
            });
        }
    }));

    // A linear map of the active row across the scroll range. It costs no
    // measurement and still always lands the row inside the viewport - the
    // row's offset from the top works out to `row * (viewport - row height) /
    // (rows - 1)`, which never exceeds the viewport.
    let scroll_y = active_row
        .filter(|_| count > 1)
        .map(|row| row as f64 / (count - 1) as f64 * 100.0);

    // An open list with nothing to show and no `empty` renders nothing at all -
    // an empty bordered box is not a state worth drawing, and it is what would
    // otherwise force callers to derive `opened` from the option count.
    // A `header` keeps the box open through a list of nothing: a search that
    // matches no row must not close the thing holding the search.
    let showing =
        opened && (loading || count > 0 || props.empty.is_some() || props.header.is_some());
    // Portaled rather than nested, so no `overflow: hidden` ancestor can clip
    // it and it can flip above the trigger when the page runs out of room.
    popover.show(showing.then(|| {
        dropdown
            .element(popover.floating())
            // Clicking the list's padding or its scrollbar must not move focus
            // off the trigger: a trigger that closes on blur would close under
            // the click. The rows cancel it for themselves already.
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            // The same handler as the wrapper's, because the dropdown is
            // portaled: it is no descendant of that wrapper, so a key pressed
            // inside it bubbles to `PortalOutlet` instead. Nothing in a plain
            // dropdown can hold focus, so this is dead weight there - it is
            // what lets a search box inside the list answer the arrows. The
            // closure captures only `Copy` values, so it is `Copy` too.
            .event("onkeydown", onkeydown)
            .attr("aria-busy", loading.then_some("true"))
            .render(
                HtmlTag::Div,
                props.attributes,
                rsx! {
                    ComboboxDropdown {
                        rows: props.rows,
                        active: active_row,
                        id: id(),
                        max_height: theme.combobox.max_dropdown_height,
                        scroll_y,
                        empty: props.empty,
                        loading: props.loading,
                        header: props.header,
                        multiselectable: props.multiselectable,
                        context,
                    }
                },
            )
    }));

    wrapper
        .element(&anchor)
        // The trigger is the caller's, so the keys are caught where they
        // bubble to rather than on a field this component owns. The portaled
        // dropdown carries the very same handler, for focus that moved inside
        // it.
        .event("onkeydown", onkeydown)
        .render(HtmlTag::Div, Vec::new(), props.children)
}
