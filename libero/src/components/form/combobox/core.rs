use dioxus::prelude::*;

use crate::{
    components::{
        ClassList, HtmlTag, Input, States,
        a11y::VISUALLY_HIDDEN_FIXED_SX,
        common::{NavigationChord, base_props, navigation_chord},
        layout::use_box,
        surface::paper_sx,
    },
    hooks::{
        ElementHandle, PopoverOptions, PopoverWidth, use_element, use_field_list_layer,
        use_popover, use_theme,
    },
    platform::ElementApi,
    sx::{StaticSx, Sx, sx},
    theme::{COMBOBOX_PADDING, Size, SizeCss, Z_INDEX_POPOVER},
};

use super::{dropdown::ComboboxDropdown, option::ComboboxContext, state::ComboboxState};

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
        // A group's heading. Styled from here, by a selector, because the
        // headings are drawn inside a loop where a `use_box` would be a hook
        // called a variable number of times - the shape `Spotlight`'s own group
        // labels use.
        .selector(
            "& [data-slot='group-label']",
            sx().padding("4px 8px")
                .font_size("0.75em")
                .font_weight("600")
                .line_height("1.5")
                .color("text-dimmed")
                .white_space("nowrap")
                .overflow("hidden")
                .text_overflow("ellipsis"),
        )
});

base_props! {
    pub(crate) struct ComboboxCoreProps {
        /// The rows, already drawn - which is what erases the caller's `T`,
        /// and what stops anything below here from memoizing.
        rows: Vec<Element>,
        /// One group label per row, parallel to `rows`, `None` for a row in no
        /// group. Handed straight to the dropdown, which draws adjacent equal
        /// labels as one `role="group"`. Groups change how rows are wrapped,
        /// never which index a row reports.
        #[props(default)]
        groups: Vec<Option<String>>,
        /// One flag per row, parallel to `rows`. A disabled row is drawn and
        /// read out, `aria-disabled`; the arrows pass over it and it registers
        /// no Enter target, so the keyboard can never pick it.
        #[props(default)]
        row_disabled: Vec<bool>,
        /// The highlighted row, or `None` for no highlight - which is what an
        /// autocomplete opens with, so Enter commits the typed text instead of
        /// the top suggestion.
        active: Option<usize>,
        onactive: EventHandler<usize>,
        opened: bool,
        onopened: EventHandler<bool>,
        /// The caller's state: its id is what the aria wiring is built from,
        /// and it is told how many rows are drawn.
        state: ComboboxState,
        #[props(default)]
        empty: Option<Element>,
        /// `Some(label)` while the options are being fetched: the dropdown
        /// shows a `Loader` in place of the rows and `empty`, and the status
        /// region says `label`.
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
        /// Disabled or read-only: the list is neither drawn nor opened by a key.
        disabled: bool,
        /// Whether Enter closes the list after picking. A multi-select keeps
        /// it open, so each pick toggles a row and the next one is one key
        /// away.
        #[props(default = true)]
        close_on_pick: bool,
        /// Sets `aria-multiselectable` on the listbox.
        #[props(default)]
        multiselectable: bool,
        /// The id naming the listbox, usually the field's label.
        #[props(default)]
        labelled_by: Option<String>,
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

    let context = use_combobox_context(props.state.id(), size, radius);
    // Provided here for a row drawn inside the trigger, and handed to the
    // portaled dropdown as a prop, which mounts outside this scope entirely.
    use_context_provider(|| context);

    // A disabled or read-only list is never drawn, whatever the state says:
    // the pointer and the ARIA go through this gate, not only the keys.
    let opened = props.opened && !props.disabled;
    props.state.set_held(props.state.is_open() && !opened);
    let loading = props.loading.is_some();
    // Said by a region outside the dropdown, not by the loader inside it. The
    // dropdown is `aria-busy` while loading, and some screen readers hold a
    // busy subtree's changes back until it is done - by which time the loader
    // is gone. And a region has to be in the tree before its text changes to
    // be announced, so it stays mounted, empty, while there is nothing to say.
    let status = props.loading.clone().filter(|_| opened);
    // The rows belong to the previous query while a fetch runs, so they are
    // neither drawn nor reachable by the arrows.
    let count = if loading { 0 } else { props.rows.len() };
    // Only the open list's count is read, and a skin may draw no rows while closed.
    if opened {
        props.state.set_rows(count);
    }
    // The rows the keyboard is allowed on, in order. Everything the arrows do
    // is a move inside this list, so a disabled row is not a case any of them
    // has to remember.
    let enabled: Vec<usize> = (0..count)
        .filter(|row| !props.row_disabled.get(*row).copied().unwrap_or(false))
        .collect();
    // A disabled row is never the highlight. Opening, a filter change and a
    // caller's own `set_active` can each land on one, so it is snapped here,
    // once, rather than guarded at all three: forward first, because a list
    // opens at or after what is selected, and back only when there is nothing
    // left ahead.
    let active_row = props
        .active
        .filter(|_| count > 0)
        .map(|row| row.min(count - 1))
        .and_then(|row| {
            enabled
                .iter()
                .copied()
                .find(|candidate| *candidate >= row)
                .or_else(|| enabled.last().copied())
        });
    let set_active = props.onactive;

    let keys = ComboboxKeys {
        active_row,
        arrows: arrow_targets(&enabled, active_row, opened),
        set_active,
        onopened: props.onopened,
        active_pick: context.active_pick,
        opened,
        disabled: props.disabled,
        close_on_pick: props.close_on_pick,
    };
    let onkeydown = move |event: KeyboardEvent| keys.handle(event);
    let disabled = props.disabled;

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("bordered", true)
        .with("disabled", disabled)
        .into();

    // On the Escape stack exactly while the key handler above would take
    // Escape, so a `HoverCard` around this field leaves the press to it.
    use_field_list_layer(opened);
    let anchor = use_element();
    // Unstyled, and not user-facing: it is only what the keys are caught on and
    // what the dropdown is measured against, which is why `sx` lands on the
    // dropdown instead. Its width is the trigger's, so `PopoverWidth::Match`
    // reproduces the `width: 100%` the dropdown had while it was nested.
    let wrapper = use_box().prepare();
    // A span in this scope rather than a `VisuallyHidden`, which would be one
    // more scope re-rendered with every list.
    let status_box = use_box().framework_sx(&VISUALLY_HIDDEN_FIXED_SX).prepare();

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
    // Mounted only while showing, so a closed list pays for no popover.
    let popup = showing.then(|| {
        rsx! {
            ComboboxPopup {
                anchor,
                keys,
                width: props.width,
                remeasure: props.remeasure,
                autofocus: props.autofocus,
                class: props.class,
                sx: props.sx,
                states,
                attributes: props.attributes,
                rows: props.rows,
                groups: props.groups,
                row_disabled: props.row_disabled,
                active: active_row,
                scroll_y,
                empty: props.empty,
                loading,
                header: props.header,
                multiselectable: props.multiselectable,
                labelled_by: props.labelled_by,
                context,
            }
        }
    });

    wrapper
        .element(&anchor)
        // The trigger is the caller's, so the keys are caught where they
        // bubble to rather than on a field this component owns. The portaled
        // dropdown carries the very same handler, for focus that moved inside
        // it.
        .event("onkeydown", onkeydown)
        .render(
            HtmlTag::Div,
            Vec::new(),
            rsx! {
                {props.children}
                {status_box.attr("role", "status").render(HtmlTag::Span, Vec::new(), rsx! {
                    if let Some(text) = status {
                        "{text}"
                    }
                })}
                {popup}
            },
        )
}

#[derive(Props, Clone, PartialEq)]
struct ComboboxPopupProps {
    anchor: ElementHandle,
    keys: ComboboxKeys,
    width: PopoverWidth,
    remeasure: u64,
    autofocus: Option<ElementHandle>,
    class: Input<ClassList>,
    sx: Input<Sx>,
    states: Input<States>,
    attributes: Vec<Attribute>,
    rows: Vec<Element>,
    groups: Vec<Option<String>>,
    row_disabled: Vec<bool>,
    active: Option<usize>,
    scroll_y: Option<f64>,
    empty: Option<Element>,
    loading: bool,
    header: Option<Element>,
    multiselectable: bool,
    labelled_by: Option<String>,
    context: ComboboxContext,
}

/// The open dropdown: the popover, its measuring and the portaled box.
#[component]
fn ComboboxPopup(props: ComboboxPopupProps) -> Element {
    let theme = use_theme();
    let popover = use_popover(
        props.anchor,
        true,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            // Portaling takes away the positioned wrapper a `width: 100%` used
            // to resolve against, so the width is measured instead.
            .width(props.width)
            .remeasure(props.remeasure),
    );
    let dropdown = use_box()
        .framework_sx(&COMBOBOX_DROPDOWN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .style(popover.style())
        .prepare();

    // Placed means measured, which means visible - the first moment at which
    // focusing anything inside the box can take.
    let autofocus = props.autofocus;
    let placed = popover.placed();
    use_effect(use_reactive!(|placed| {
        if let (true, Some(target)) = (placed, autofocus) {
            // Out of this dispatch: the click that opened the list ends by
            // focusing the trigger, so focusing inline is undone a moment
            // later.
            spawn(async move {
                let _ = target.focus();
            });
        }
    }));

    let keys = props.keys;
    let context = props.context;
    // Portaled rather than nested, so no `overflow: hidden` ancestor can clip
    // it and it can flip above the trigger when the page runs out of room.
    popover.show(Some(
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
            // what lets a search box inside the list answer the arrows.
            .event("onkeydown", move |event: KeyboardEvent| keys.handle(event))
            .attr("aria-busy", props.loading.then_some("true"))
            .render(
                HtmlTag::Div,
                props.attributes,
                rsx! {
                    ComboboxDropdown {
                        rows: props.rows,
                        groups: props.groups,
                        row_disabled: props.row_disabled,
                        active: props.active,
                        id: (context.id)(),
                        max_height: theme.combobox.max_dropdown_height,
                        scroll_y: props.scroll_y,
                        empty: props.empty,
                        loading: props.loading,
                        header: props.header,
                        multiselectable: props.multiselectable,
                        labelled_by: props.labelled_by,
                        context,
                    }
                },
            ),
    ));

    rsx! {}
}

/// The four signals every portaled row reads.
///
/// Owned by the root scope, not by the component's, and dropped by hand on
/// unmount. The rows that read them are portaled, so they mount under
/// `PortalOutlet` and are *not* descendants of `ComboboxCore` - a signal
/// created there would be read from outside the scope that owns it, which
/// dioxus warns about and which really can drop the value while a row still
/// holds it.
fn use_combobox_context(state_id: String, size: Size, radius: Size) -> ComboboxContext {
    let id = use_hook(|| Signal::new_in_scope(state_id, ScopeId::ROOT));
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

    ComboboxContext {
        id,
        size: shared_size,
        radius: shared_radius,
        active_pick,
    }
}

/// Where each arrow lands, worked out in render rather than inside the
/// handler: they depend only on what this render already knows, and four
/// `Option<usize>` keep the handler `Copy` - which is what lets the wrapper
/// and the portaled dropdown share one closure.
#[derive(Clone, Copy, PartialEq)]
struct Arrows {
    down: Option<usize>,
    up: Option<usize>,
    first: Option<usize>,
    last: Option<usize>,
}

fn arrow_targets(enabled: &[usize], active_row: Option<usize>, opened: bool) -> Arrows {
    let at = active_row.and_then(|row| enabled.iter().position(|row_at| *row_at == row));
    let step = |delta: isize| {
        let at = at?.saturating_add_signed(delta);
        enabled
            .get(at.min(enabled.len().saturating_sub(1)))
            .copied()
    };
    let first = enabled.first().copied();

    Arrows {
        // From no highlight, both arrows land on the first row the keyboard
        // may have - the first press arms the list rather than moving inside
        // it.
        down: match (opened, active_row) {
            (true, Some(_)) => step(1),
            _ => active_row.or(first),
        },
        up: step(-1),
        first,
        last: enabled.last().copied(),
    }
}

/// The list's one keyboard, shared by the wrapper and the portaled dropdown.
/// Every field is `Copy`, so the closure that forwards to it is `Copy` too.
#[derive(Clone, Copy, PartialEq)]
struct ComboboxKeys {
    active_row: Option<usize>,
    arrows: Arrows,
    set_active: EventHandler<usize>,
    onopened: EventHandler<bool>,
    active_pick: Signal<Option<Callback<()>>>,
    opened: bool,
    disabled: bool,
    close_on_pick: bool,
}

impl ComboboxKeys {
    fn handle(self, event: KeyboardEvent) {
        let (opened, active_row) = (self.opened, self.active_row);
        if self.disabled {
            return;
        }
        let request = |next: bool| self.onopened.call(next);
        // A list of nothing but disabled rows has nowhere to go, and the
        // arrows then only open it.
        let go_to = |row: Option<usize>| {
            if let Some(row) = row {
                self.set_active.call(row);
            }
            if !opened {
                request(true);
            }
        };

        // APG: Alt+ArrowDown opens without moving the highlight.
        match navigation_chord(&event) {
            Some(NavigationChord::Open) => {
                event.prevent_default();
                if !opened {
                    request(true);
                }
                return;
            }
            Some(NavigationChord::Close) if opened => {
                event.prevent_default();
                request(false);
                return;
            }
            Some(_) => return,
            None => {}
        }
        match event.key() {
            Key::ArrowDown => {
                event.prevent_default();
                go_to(self.arrows.down);
            }
            Key::ArrowUp if opened => {
                event.prevent_default();
                go_to(self.arrows.up);
            }
            Key::Home if opened => {
                event.prevent_default();
                go_to(self.arrows.first);
            }
            Key::End if opened => {
                event.prevent_default();
                go_to(self.arrows.last);
            }
            // Nothing highlighted means Enter is not ours: it bubbles, so a
            // form still submits.
            Key::Enter if opened && active_row.is_some() => {
                event.prevent_default();
                if let Some(pick) = (self.active_pick)() {
                    pick.call(());
                }
                if self.close_on_pick {
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
    }
}
