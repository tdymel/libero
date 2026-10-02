use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::VISUALLY_HIDDEN_FIXED_SX,
        common::{
            ClassList, HtmlTag, Input, NavigationChord, Part, Parts, States, base_props,
            navigation_chord,
        },
        form::DropdownPart,
        layout::{paper_sx, use_box},
    },
    hooks::{
        ElementHandle, POPOVER_AVAILABLE_HEIGHT, PopoverOptions, PopoverWidth,
        current_localization, use_element, use_field_list_layer, use_popover_on, use_theme,
    },
    platform::ElementApi,
    sx::{StaticSx, Sx, sx},
    theme::{COMBOBOX_PADDING, Size, SizeCss, Z_INDEX_POPOVER},
};

use super::{ComboboxState, dropdown::ComboboxDropdown, option::ComboboxContext};

// A `paper_sx()` surface, through `use_box` rather than `Paper` for the popover's handle and events.
pub(crate) static COMBOBOX_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        // Position and width come from `use_popover` as an inline style.
        .z_index(Z_INDEX_POPOVER.value())
        .display("flex")
        .flex_direction("column")
        .gap("4px")
        .padding(COMBOBOX_PADDING)
        // Rows don't nest inside an `xxl` radius, so the dropdown clips.
        .overflow("hidden")
        // Never past the room on its side: the listbox, a scroll container, shrinks.
        .max_height(POPOVER_AVAILABLE_HEIGHT.value_or("none"))
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
        // By selector: headings are drawn in a loop, where `use_box` can't be called. As `Spotlight`.
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
        .selector(
            "& [data-slot='nothing-found']",
            sx().padding("4px 8px").color("text-dimmed"),
        )
});

base_props! {
    parts(DropdownPart);
    pub(crate) struct ComboboxCoreProps {
        /// Already drawn: erases the caller's `T` and stops memoizing below.
        rows: Vec<Element>,
        /// One stable key per row, parallel to `rows`, so a filter keeps each kept row's scope.
        /// Empty keys rows by position.
        #[props(default)]
        row_keys: Vec<usize>,
        /// One group label per row, parallel to `rows`. Wraps rows, never changes their index.
        #[props(default)]
        groups: Vec<Option<String>>,
        /// Parallel to `rows`. Drawn and `aria-disabled`, never reachable by the keyboard.
        #[props(default)]
        row_disabled: Vec<bool>,
        /// `None` is no highlight, as an autocomplete opens, so Enter commits the typed text.
        active: Option<usize>,
        onactive: EventHandler<usize>,
        opened: bool,
        onopened: EventHandler<bool>,
        /// Its id builds the aria wiring; it is told the row count.
        state: ComboboxState,
        #[props(default)]
        empty: Option<Element>,
        /// `Some(label)` while fetching: a `Loader` replaces rows and `empty`; the status region says it.
        #[props(default)]
        loading: Option<String>,
        /// `Some(text)` while a query filters. No rows left shows `empty`, else `text`, and says it.
        #[props(default)]
        nothing_found: Option<String>,
        /// A search box above the rows, outside the scroll. It survives an empty list.
        #[props(default)]
        header: Option<Element>,
        /// Focused once visible: before `use_popover` measures, focus silently fails.
        #[props(default)]
        autofocus: Option<ElementHandle>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        radius: Input<Size>,
        /// Disabled or read-only: the list is neither drawn nor opened by a key.
        disabled: bool,
        /// Enter closes after picking. A multi-select keeps the list open.
        #[props(default = true)]
        close_on_pick: bool,
        /// When Home and End move the text caret (APG editable combobox).
        #[props(default)]
        caret_keys: CaretKeys,
        /// APG select-only: Tab and Alt+ArrowUp pick the highlight before closing.
        #[props(default)]
        commit_on_leave: bool,
        /// Space picks the highlight, like Enter: a list with no text to type in.
        #[props(default)]
        space_picks: bool,
        #[props(default)]
        multiselectable: bool,
        /// Usually the field's label id.
        #[props(default)]
        labelled_by: Option<String>,
        /// `Match` the trigger; a select takes `Min` so a long row is never clipped.
        #[props(default = PopoverWidth::Match)]
        width: PopoverWidth,
        /// Remeasures on change, for a trigger that resizes while open.
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
    // For a row inside the trigger; the portaled dropdown gets it as a prop.
    use_context_provider(|| context);

    // Disabled or read-only never draws: pointer and ARIA pass this gate, not only the keys.
    let opened = props.opened && !props.disabled;
    let loading = props.loading.is_some();
    // Said outside the `aria-busy` dropdown, whose changes some readers hold back. While
    // fetching, the rows are the previous query's, so none are drawn or reachable.
    let count = if loading { 0 } else { props.rows.len() };
    let filtering = opened && !loading && props.nothing_found.is_some();
    let nothing_found = props
        .nothing_found
        .clone()
        .filter(|_| filtering && count == 0);
    // What a query left, said too (WCAG 4.1.3, todo 1574).
    let results =
        (filtering && count > 0).then(|| (current_localization().combobox.results)(count));
    let status = props
        .loading
        .clone()
        .filter(|_| opened)
        .or_else(|| nothing_found.clone())
        .or(results);
    let empty = props
        .empty
        .or_else(|| nothing_found.map(|text| nothing_found_row(&text)));
    // Only the open list's count is read, and a skin may draw no rows while closed.
    if opened {
        props.state.set_rows(count);
    }
    // The rows the arrows move through.
    let enabled: Vec<usize> = (0..count)
        .filter(|row| !props.row_disabled.get(*row).copied().unwrap_or(false))
        .collect();
    let active_row = snap_row(props.active, &enabled);
    if let (true, Some(row)) = (opened, active_row) {
        props.state.snap_active(props.active, row);
    }
    let set_active = props.onactive;
    let mut enabled_rows = use_hook(|| CopyValue::new(Vec::new()));
    if *enabled_rows.peek() != enabled {
        enabled_rows.set(enabled);
    }

    let keys = ComboboxKeys {
        state: props.state,
        enabled: enabled_rows,
        set_active,
        onopened: props.onopened,
        picks: context.picks,
        opened,
        disabled: props.disabled,
        close_on_pick: props.close_on_pick,
        caret_keys: props.caret_keys,
        commit_on_leave: props.commit_on_leave,
        space_picks: props.space_picks,
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

    // On the Escape stack while open, so a surrounding `HoverCard` leaves the press to it.
    let onopened = props.onopened;
    use_field_list_layer(opened, use_callback(move |()| onopened.call(false)));
    let anchor = use_element();
    // Unstyled: catches the keys and anchors the dropdown, so `sx` goes to the dropdown.
    let wrapper = use_box().prepare();
    // A span, not `VisuallyHidden`: one scope fewer to re-render.
    let status_box = use_box().framework_sx(&VISUALLY_HIDDEN_FIXED_SX).prepare();

    // Linear over the scroll range: no measuring, and the row always lands in view.
    let scroll_y = active_row
        .filter(|_| count > 1)
        .map(|row| row as f64 / (count - 1) as f64 * 100.0);

    // Nothing to show draws nothing, but a `header` keeps the box: a failed search must stay editable.
    let showing = opened && (loading || count > 0 || empty.is_some() || props.header.is_some());
    // Open in the state but nothing on screen: the trigger must not claim `aria-expanded`.
    props.state.set_held(props.state.is_open() && !showing);
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
                parts: props.parts,
                states,
                attributes: props.attributes,
                rows: props.rows,
                row_keys: props.row_keys,
                groups: props.groups,
                row_disabled: props.row_disabled,
                active: active_row,
                scroll_y,
                empty,
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
        // The trigger is the caller's, so keys are caught as they bubble. The dropdown has the same handler.
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

/// The dropdown's text for a query that matched nothing. `Cascader` draws it too.
pub(crate) fn nothing_found_row(text: &str) -> Element {
    rsx! {
        div { "data-slot": DropdownPart::Empty.slot(), "{text}" }
    }
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
    parts: Input<Parts<DropdownPart>>,
    states: Input<States>,
    attributes: Vec<Attribute>,
    rows: Vec<Element>,
    row_keys: Vec<usize>,
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
    let popover = use_popover_on(
        props.anchor,
        use_element(),
        true,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            // Portaled, so no wrapper for `width: 100%`: measured instead.
            .width(props.width)
            .remeasure(props.remeasure),
    );
    let dropdown = use_box()
        .framework_sx(&COMBOBOX_DROPDOWN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .style(Some(popover.style()))
        .prepare();

    // Placed means measured and visible, the first moment focus can take.
    let autofocus = props.autofocus;
    let placed = popover.placed();
    use_effect(use_reactive!(|placed| {
        if let (true, Some(target)) = (placed, autofocus) {
            // Deferred: the opening click ends by focusing the trigger.
            spawn(async move {
                let _ = target.focus();
            });
        }
    }));

    let keys = props.keys;
    let context = props.context;
    // Portaled, so no `overflow: hidden` ancestor clips it and it can flip above the trigger.
    popover.show(Some(
        dropdown
            .element(popover.floating())
            // A click on the padding or scrollbar must not blur the trigger, which would close the list.
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            // Portaled, so keys bubble to `PortalOutlet`; this lets a search box inside answer the arrows.
            .event("onkeydown", move |event: KeyboardEvent| keys.handle(event))
            .attr_default("data-slot", DropdownPart::Panel.slot())
            .attr("aria-busy", props.loading.then_some("true"))
            .render(
                HtmlTag::Div,
                props.attributes,
                rsx! {
                    ComboboxDropdown {
                        rows: props.rows,
                        row_keys: props.row_keys,
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

/// The signals every portaled row reads. Owned by the root scope and dropped by hand:
/// a component-owned signal read from a portaled row can be dropped while still held.
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
    let picks = use_hook(|| CopyValue::new_in_scope(HashMap::new(), ScopeId::ROOT));
    use_drop(move || {
        id.manually_drop();
        shared_size.manually_drop();
        shared_radius.manually_drop();
        picks.manually_drop();
    });

    ComboboxContext {
        id,
        size: shared_size,
        radius: shared_radius,
        picks,
    }
}

/// Where each arrow lands from the highlight.
#[derive(Clone, Copy, PartialEq)]
struct Arrows {
    down: Option<usize>,
    up: Option<usize>,
    first: Option<usize>,
    last: Option<usize>,
}

/// Snaps a disabled or missing highlight (from open, filter or `set_active`) to the next
/// enabled row, else back.
fn snap_row(active: Option<usize>, enabled: &[usize]) -> Option<usize> {
    active.and_then(|row| {
        enabled
            .iter()
            .copied()
            .find(|candidate| *candidate >= row)
            .or_else(|| enabled.last().copied())
    })
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
        // From no highlight, the first press arms the first enabled row.
        down: match (opened, active_row) {
            (true, Some(_)) => step(1),
            _ => active_row.or(first),
        },
        up: step(-1),
        first,
        last: enabled.last().copied(),
    }
}

/// The one keyboard, shared by the wrapper and the portaled dropdown; `Copy` throughout.
#[derive(Clone, Copy, PartialEq)]
struct ComboboxKeys {
    /// Its highlight is read at the press: a fast Enter can beat the arrow's re-render (todo 1367).
    state: ComboboxState,
    /// The rows the arrows move through, as last drawn.
    enabled: CopyValue<Vec<usize>>,
    set_active: EventHandler<usize>,
    onopened: EventHandler<bool>,
    picks: CopyValue<HashMap<usize, Callback<()>>>,
    opened: bool,
    disabled: bool,
    close_on_pick: bool,
    caret_keys: CaretKeys,
    commit_on_leave: bool,
    space_picks: bool,
}

/// Who owns Home and End while the list is open.
#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum CaretKeys {
    /// The rows: a trigger with no text caret.
    #[default]
    Off,
    /// The caret until a row is highlighted: a trigger whose text is the value.
    Unhighlighted,
    /// The caret: a search box, which always arms a row as it filters.
    Always,
}

impl ComboboxKeys {
    fn handle(self, event: KeyboardEvent) {
        if self.disabled {
            return;
        }
        let opened = self.opened;
        let enabled = self.enabled.peek().clone();
        let active_row = snap_row(self.state.active_now(), &enabled);
        let arrows = arrow_targets(&enabled, active_row, opened);
        let request = |next: bool| self.onopened.call(next);
        let pick = || {
            let pick = active_row.and_then(|row| self.picks.peek().get(&row).copied());
            if let Some(pick) = pick {
                pick.call(());
            }
        };
        // Leaving an open select-only list keeps what the highlight is on.
        let leave = || {
            if self.commit_on_leave && active_row.is_some() {
                pick();
            }
            request(false);
        };
        // With only disabled rows, the arrows just open the list.
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
                leave();
                return;
            }
            Some(_) => return,
            None => {}
        }
        // Typing leaves no highlight, and then Home/End are text editing keys.
        let row_keys = opened
            && match self.caret_keys {
                CaretKeys::Off => true,
                CaretKeys::Unhighlighted => active_row.is_some(),
                CaretKeys::Always => false,
            };
        match event.key() {
            Key::ArrowDown => {
                event.prevent_default();
                go_to(arrows.down);
            }
            Key::ArrowUp if opened => {
                event.prevent_default();
                go_to(arrows.up);
            }
            Key::Home if row_keys => {
                event.prevent_default();
                go_to(arrows.first);
            }
            Key::End if row_keys => {
                event.prevent_default();
                go_to(arrows.last);
            }
            // Nothing highlighted: Enter bubbles, so a form still submits.
            Key::Enter if opened && active_row.is_some() => {
                event.prevent_default();
                pick();
                if self.close_on_pick {
                    request(false);
                }
            }
            Key::Character(ref key)
                if key == " " && self.space_picks && opened && active_row.is_some() =>
            {
                event.prevent_default();
                pick();
                if self.close_on_pick {
                    request(false);
                }
            }
            Key::Escape if opened => {
                event.prevent_default();
                request(false);
            }
            Key::Tab if opened => leave(),
            _ => {}
        }
    }
}
