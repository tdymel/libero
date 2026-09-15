use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        Box, Dialog, Kbd, Loader, ScrollArea, VisuallyHidden,
        common::{inset_focus_ring_sx, navigation_chord, use_name_warning},
        form::{ComboboxState, use_combobox},
    },
    hooks::{ModalHandle, ModalScope, use_dismiss_layer, use_localization, use_modal, use_theme},
    platform::{KeyChord, KeySubscription, keyboard, warn_reserved_chord},
    sx::{StaticSx, sx},
    theme::{
        SPOTLIGHT_DESCRIPTION_COLOR, SPOTLIGHT_GROUP_COLOR, SPOTLIGHT_MAX_LIST_HEIGHT,
        SPOTLIGHT_PADDING, SPOTLIGHT_SEARCH_FONT_SIZE, SPOTLIGHT_TOP_OFFSET, SPOTLIGHT_WIDTH, Size,
        SizeCss,
    },
    utils::warn,
};

use super::action::{SpotlightAction, group_and_limit};

// The palette's insides, styled from one place - the rows carry no class of
// their own, the `Tabs` and `Menu` shape. The row's three states copy
// `ComboboxOption`'s fold: hover tints, the keyboard's row tints darker and
// takes an inset ring, because a tint alone cannot say "the arrows are here"
// on a row the pointer is also over.
static SPOTLIGHT_BODY_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(SPOTLIGHT_PADDING.value())
        .selector(
            "& > input",
            sx().width("100%")
                .padding("12px")
                .font("inherit")
                .font_size(SPOTLIGHT_SEARCH_FONT_SIZE.value())
                .color("inherit")
                .background("transparent")
                .border("0")
                .border_bottom("2px solid var(--lsx-muted-3)")
                .outline("none"),
        )
        // The input always holds focus, so its indicator is the line under it
        // turning primary rather than a ring round the whole box.
        .selector("& > input:focus", sx().border_bottom_color("primary.6"))
        .selector(
            "& [data-spotlight-group-label]",
            sx().padding("8px 12px 4px")
                .font_size("0.75rem")
                .font_weight("600")
                .color(SPOTLIGHT_GROUP_COLOR.value())
                .user_select("none"),
        )
        .selector(
            "& [role=\"option\"]",
            sx().display("flex")
                .align_items("center")
                .gap("12px")
                .min_height("44px")
                .padding("6px 12px")
                .border_radius(format!(
                    "max(0px, calc({} - {}))",
                    SizeCss::RADIUS.value(crate::theme::Size::Md),
                    SPOTLIGHT_PADDING.value()
                ))
                .cursor("pointer")
                .user_select("none"),
        )
        .selector("& [role=\"option\"]:hover", sx().background("muted.1"))
        .selector(
            "& [role=\"option\"][data-active]",
            sx().background("muted.2"),
        )
        .selector(
            "& [role=\"option\"][data-active]",
            inset_focus_ring_sx("-2px"),
        )
        .selector(
            "& [data-spotlight-icon]",
            sx().display("inline-flex").align_items("center"),
        )
        .selector(
            "& [data-spotlight-text]",
            sx().display("flex")
                .flex_direction("column")
                .flex("1")
                .min_width("0")
                .with("overflow-wrap", "anywhere"),
        )
        .selector(
            "& [data-spotlight-shortcut]",
            sx().display("inline-flex")
                .align_items("center")
                .gap("4px")
                .white_space("nowrap"),
        )
        .selector(
            "& [data-spotlight-description]",
            sx().font_size("0.8125rem")
                .color(SPOTLIGHT_DESCRIPTION_COLOR.value()),
        )
        .selector(
            "& [role=\"status\"]",
            sx().padding("12px").text_align("center"),
        )
        .selector("& [role=\"status\"]:empty", sx().display("none"))
});

/// How a [`use_spotlight`] palette behaves. `Default` is a palette with no
/// actions - set `actions`, or it warns once and stays empty.
#[derive(Clone)]
pub struct SpotlightOptions {
    /// Called with the live query, returns what to show - filtered, sorted and
    /// grouped however you like. [`spotlight_filter`](super::spotlight_filter)
    /// is the common case.
    ///
    /// **Capture a `Signal`, not a `Vec`**, if the list itself changes: the
    /// closure is stored, so a captured `Vec` is the one from when it was made.
    pub actions: Option<Callback<String, Vec<SpotlightAction>>>,
    /// The search box's placeholder. Unset, the localization's.
    pub placeholder: Option<String>,
    /// Drawn, and announced, when a non-empty query matches nothing. Unset,
    /// the localization's text.
    pub nothing_found: Option<Element>,
    /// A cap on the rows drawn, counted through the groups.
    pub limit: Option<usize>,
    /// Close after running an action.
    pub close_on_action: bool,
    /// Start every opening with an empty query.
    pub clear_on_close: bool,
    /// Names the dialog. Unset, the theme's "Command palette".
    pub aria_label: Option<String>,
    /// Ctrl (Cmd on a Mac) plus this key toggles the palette from anywhere on
    /// the page - `'k'` by default, `None` for no hotkey. Web only: no other
    /// renderer has a document-level key listener yet. A key the browser
    /// already uses (L, T, W, R, F, ...) warns in a debug build.
    pub shortcut: Option<char>,
    /// The actions are still being fetched. The rows and "nothing found" give
    /// way to a loader, and the status region says the theme's loading text,
    /// so a search-as-you-type palette never flashes "nothing found" first.
    pub loading: bool,
    /// Highlight the first row after every keystroke, so `Enter` runs it
    /// without an `ArrowDown` first - a page search's common case. Off, a
    /// fresh query arms nothing and `Enter` is inert until the arrows pick a
    /// row, which is safer for a palette whose actions do something.
    pub highlight_first_on_query: bool,
    /// Called with the new query on every keystroke, from the input event
    /// rather than a render - where a search-as-you-type palette sets
    /// `loading` and starts its fetch, so the very next frame is already busy.
    pub onquery: Option<Callback<String>>,
}

impl Default for SpotlightOptions {
    fn default() -> Self {
        Self {
            actions: None,
            placeholder: None,
            nothing_found: None,
            limit: None,
            close_on_action: true,
            clear_on_close: true,
            aria_label: None,
            shortcut: Some('k'),
            loading: false,
            highlight_first_on_query: true,
            onquery: None,
        }
    }
}

/// Opens and closes a [`use_spotlight`] palette. `Copy`.
#[derive(Clone, Copy)]
pub struct SpotlightHandle {
    modal: ModalHandle<()>,
    query: Signal<String>,
    state: ComboboxState,
    clear_on_close: bool,
}

impl SpotlightHandle {
    /// Opens the palette, with focus in its search box. Call it from the
    /// handler of whatever the user acted on, so focus goes back there on
    /// close.
    pub fn open(&self) {
        if self.modal.is_open() {
            return;
        }
        if self.clear_on_close {
            let mut query = self.query;
            query.set(String::new());
        }
        self.state.set_active(None);
        self.state.open();
        self.modal.open();
    }

    pub fn close(&self) {
        self.state.close();
        self.modal.close();
    }

    pub fn toggle(&self) {
        match self.is_open() {
            true => self.close(),
            false => self.open(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.modal.is_open()
    }
}

/// A command palette: a modal search box over a list of actions, with the
/// arrow keys moving a highlight and Enter running it.
///
/// A hook, like [`use_modal`](crate::hooks::use_modal): the palette is
/// portaled from here, so call it in a component that outlives every trigger.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, SpotlightAction, SpotlightOptions, spotlight_filter, use_spotlight};
/// # fn app() -> Element {
/// # fn build_actions() -> Vec<SpotlightAction> { Vec::new() }
/// let pages = use_signal(build_actions);
/// let spotlight = use_spotlight(SpotlightOptions {
///     actions: Some(Callback::new(move |query: String| spotlight_filter(&query, &pages()))),
///     ..Default::default()
/// });
/// rsx! { Button { onclick: move |_| spotlight.open(), "Search" } }
/// # }
/// ```
pub fn use_spotlight(options: SpotlightOptions) -> SpotlightHandle {
    let theme = use_theme();
    let state = use_combobox();
    let query = use_signal(String::new);

    let missing = options.actions.is_none();
    use_hook(|| {
        if missing {
            warn("use_spotlight: no `actions` - the palette will always be empty");
        }
    });
    use_name_warning(
        options.aria_label.is_some(),
        "use_spotlight: no `aria_label`, falling back to the localization's. A dialog needs a name of its own to be told apart.",
    );

    let labels = use_localization().spotlight;
    let close_on_action = options.close_on_action;
    let highlight_first = options.highlight_first_on_query;
    let onquery = options.onquery;
    let options_for_render = options.clone();
    let modal = use_modal(move |scope: ModalScope<()>| {
        let options = &options_for_render;
        let id = state.id();
        let listbox = format!("{id}-listbox");
        let actions = options
            .actions
            .map(|actions| actions.call(query()))
            .unwrap_or_default();
        let loading = options.loading;
        // The rows belong to the last query while a fetch runs, so they are
        // neither drawn nor reachable by the arrows - `Combobox`'s rule.
        let actions = match loading {
            true => Vec::new(),
            false => actions,
        };
        let groups = group_and_limit(actions, options.limit);
        let count = groups
            .iter()
            .map(|(_, members)| members.len())
            .sum::<usize>();
        state.set_rows(count);
        let active = state
            .active()
            .filter(|_| count > 0)
            .map(|row| row.min(count - 1));

        // Callbacks out of *this* render's list. A `Callback` compares equal
        // across renders whatever it captures, so nothing may hold on to an
        // older list's ([[codebase/dioxus-memoization-traps]]).
        let callbacks: Rc<Vec<Option<Callback<()>>>> = Rc::new(
            groups
                .iter()
                .flat_map(|(_, members)| members.iter().map(|action| action.onclick))
                .collect(),
        );
        let run = move |callback: Option<Callback<()>>| {
            if let Some(callback) = callback {
                callback.call(());
            }
            if close_on_action {
                state.close();
                // Out of this dispatch: the modal the event is still bubbling
                // through is what the close tears down.
                spawn(async move { scope.close() });
            }
        };

        let onkeydown = move |event: KeyboardEvent| {
            let callbacks = callbacks.clone();
            spotlight_key(event, state, active, count, move |row| run(callbacks[row]));
        };

        let rows = spotlight_rows(groups, &id, active, run);

        let empty = !loading && count == 0 && !query().trim().is_empty();
        let nothing_found = empty.then(|| {
            options
                .nothing_found
                .clone()
                .unwrap_or_else(|| rsx! { "{labels.nothing_found}" })
        });
        let aria_label = options
            .aria_label
            .clone()
            .unwrap_or_else(|| labels.label.to_string());
        let placeholder = options
            .placeholder
            .clone()
            .unwrap_or_else(|| labels.placeholder.to_string());
        // The same linear map `ComboboxCore` uses: it always lands the active
        // row inside the viewport, with nothing measured.
        let scroll_y = active
            .filter(|_| count > 1)
            .map(|row| row as f64 / (count - 1) as f64 * 100.0);

        rsx! {
            Dialog {
                aria_label: aria_label.clone(),
                close_button: false,
                size: SPOTLIGHT_WIDTH.value(),
                radius: theme.spotlight.radius,
                sx: sx()
                    .align_self("flex-start")
                    .margin_top(SPOTLIGHT_TOP_OFFSET.value())
                    .padding(SPOTLIGHT_PADDING.value()),
                Box { framework_sx: &SPOTLIGHT_BODY_SX,
                    input {
                        r#type: "text",
                        autocomplete: "off",
                        spellcheck: "false",
                        "data-autofocus": "true",
                        "aria-autocomplete": "list",
                        "aria-label": "{aria_label}",
                        placeholder: "{placeholder}",
                        value: "{query}",
                        oninput: move |event: FormEvent| {
                            let mut query = query;
                            query.set(event.value());
                            if let Some(onquery) = onquery {
                                onquery.call(event.value());
                            }
                            // The first row, so Enter runs the obvious hit
                            // without an ArrowDown first. Off, the
                            // `Autocomplete` rule: a new query arms nothing.
                            // Either way the row is clamped against the new
                            // count below, so `Some(0)` on an empty result is
                            // no highlight.
                            state.set_active(highlight_first.then_some(0));
                        },
                        onkeydown,
                        // Its listbox is drawn whenever it is open, rows
                        // or none.
                        ..state.aria(true),
                    }
                    ScrollArea {
                        id: "{listbox}",
                        "role": "listbox",
                        "aria-label": "{aria_label}",
                        sx: sx().max_height(SPOTLIGHT_MAX_LIST_HEIGHT.value()),
                        scroll_position_y: scroll_y,
                        "aria-busy": loading.then_some("true"),
                        {rows.into_iter()}
                    }
                    // Always present, so a screen reader hears it fill: focus
                    // never leaves the search box, and nothing else would
                    // announce that the query matched nothing, or that a
                    // search is running. Outside the busy listbox, which some
                    // screen readers hold back until it is done. The loader
                    // is silent; the hidden text is what is said.
                    div { "role": "status",
                        if loading {
                            Loader { size: Size::Sm }
                            VisuallyHidden { "{labels.loading}" }
                        }
                        {nothing_found}
                    }
                }
            }
        }
    });

    let handle = SpotlightHandle {
        modal,
        query,
        state,
        clear_on_close: options.clear_on_close,
    };
    use_hotkey(handle, options.shortcut);
    handle
}

/// Ctrl/Cmd + `shortcut` toggles the palette.
///
/// Two subscriptions, one at a time. Closed, it listens on the **filtered**
/// stream, so the chord does not open the palette out of a text field the user
/// is typing in. Open, focus sits in the palette's own search box, which the
/// filtered stream would drop - so the closing chord is heard unfiltered.
///
/// It will not open over another surface: while any dismissible layer is open
/// (a `Modal`, a popover), the chord is left alone. A held chord is one press.
fn use_hotkey(handle: SpotlightHandle, shortcut: Option<char>) {
    let layer = use_dismiss_layer();
    // Bumped from the key callback, which runs outside every scope - on the
    // web with no runtime at all - so it only records the press, and the
    // effect below toggles ([[codebase/platform-timer]]).
    let tick = use_hook(|| Signal::new_in_scope(0u64, ScopeId::ROOT));
    let slot: Rc<RefCell<Option<Box<dyn KeySubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let slot = slot.clone();
        move || {
            slot.borrow_mut().take();
            tick.manually_drop();
        }
    });

    use_effect(use_reactive!(|shortcut| {
        if let Some(key) = shortcut {
            warn_reserved_chord(&Key::Character(key.to_string()), Modifiers::CONTROL);
        }
    }));

    let listening = slot.clone();
    use_effect(use_reactive!(|shortcut| {
        // Read here, so opening reruns this effect, not the host's render.
        let open = handle.is_open();
        listening.borrow_mut().take();
        let (Some(key), Some(api)) = (shortcut, keyboard()) else {
            return;
        };
        let callback = Box::new(move |chord: KeyChord| {
            let modifiers = chord.modifiers;
            let pressed = matches!(&chord.key, Key::Character(text) if text.eq_ignore_ascii_case(&key.to_string()));
            if !pressed
                || !(modifiers.ctrl() || modifiers.meta())
                || modifiers.alt()
                || modifiers.shift()
            {
                return false;
            }
            // Something else is open above the page: not ours to cover.
            if !open && layer.any_open() {
                return false;
            }
            if !chord.repeat {
                let mut tick = tick;
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            }
            // Ours either way, repeat included: the browser's own Ctrl+K
            // must not fire underneath.
            true
        });
        *listening.borrow_mut() = Some(match open {
            true => api.on_key_unfiltered(callback),
            false => api.on_key(callback),
        });
    }));

    let mut seen = use_signal(|| 0u64);
    use_effect(move || {
        let pressed = tick();
        if pressed == *seen.peek() {
            return;
        }
        seen.set(pressed);
        handle.toggle();
    });
}

/// The action rows, grouped. A `listbox` may hold only options and groups, so
/// a group's own label is a presentational div named by `aria-labelledby`.
fn spotlight_rows(
    groups: Vec<(Option<String>, Vec<SpotlightAction>)>,
    id: &str,
    active: Option<usize>,
    run: impl Fn(Option<Callback<()>>) + Copy + 'static,
) -> Vec<Element> {
    let mut index = 0usize;
    let mut rows = Vec::with_capacity(groups.len());
    for (group_index, (group, members)) in groups.into_iter().enumerate() {
        let options_rows: Vec<Element> = members
            .into_iter()
            .map(|action| {
                let row = index;
                index += 1;
                let onclick = action.onclick;
                let option_id = format!("{id}-option-{row}");
                // Named by its label alone; the description and shortcut describe it.
                let described = [
                    action.description.as_ref().map(|_| format!("{option_id}-description")),
                    action.shortcut.as_ref().map(|_| format!("{option_id}-shortcut")),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ");
                rsx! {
                    div {
                        key: "{row}",
                        id: "{option_id}",
                        "role": "option",
                        "aria-labelledby": "{option_id}-label",
                        "aria-describedby": (!described.is_empty()).then_some(described),
                        "data-active": (active == Some(row)).then_some("true"),
                        // Or the click takes focus out of the search box.
                        onmousedown: move |event: MouseEvent| event.prevent_default(),
                        onclick: move |_| run(onclick),
                        if let Some(icon) = action.icon {
                            span { "data-spotlight-icon": "", {icon} }
                        }
                        span { "data-spotlight-text": "",
                            span { id: "{option_id}-label", "{action.label}" }
                            if let Some(description) = action.description {
                                span {
                                    id: "{option_id}-description",
                                    "data-spotlight-description": "",
                                    "{description}"
                                }
                            }
                        }
                        if let Some(shortcut) = action.shortcut {
                            span { id: "{option_id}-shortcut", "data-spotlight-shortcut": "",
                                for (position, key) in shortcut_keys(&shortcut).into_iter().enumerate() {
                                    if position > 0 {
                                        " + "
                                    }
                                    Kbd { "{key}" }
                                }
                            }
                        }
                    }
                }
            })
            .collect();
        rows.push(match group {
            Some(group) => {
                let label_id = format!("{id}-group-{group_index}");
                rsx! {
                    div {
                        key: "group-{group_index}",
                        "role": "group",
                        "aria-labelledby": "{label_id}",
                        div {
                            id: "{label_id}",
                            "role": "presentation",
                            "data-spotlight-group-label": "",
                            "{group}"
                        }
                        {options_rows.into_iter()}
                    }
                }
            }
            // Bare rows, as in `Combobox`: an unnamed group adds nothing.
            None => rsx! {
                Fragment { key: "group-{group_index}", {options_rows.into_iter()} }
            },
        });
    }
    rows
}

/// A shortcut hint's keys, split on spaces and `+`: one `Kbd` each.
fn shortcut_keys(shortcut: &str) -> Vec<&str> {
    shortcut
        .split(|c: char| c == '+' || c.is_whitespace())
        .filter(|key| !key.is_empty())
        .collect()
}

/// The search box's keyboard: the two arrows wrap through the flat row list,
/// and Enter runs whatever is highlighted.
///
/// Nothing highlighted, nothing runs. With `highlight_first_on_query` (the
/// default) a query always leaves the first row highlighted, so Enter runs it.
fn spotlight_key(
    event: KeyboardEvent,
    state: ComboboxState,
    active: Option<usize>,
    count: usize,
    run_row: impl Fn(usize),
) {
    // A chord is the caret's or the browser's; the list is always open.
    if count == 0 || navigation_chord(&event).is_some() {
        return;
    }
    match event.key() {
        Key::ArrowDown => {
            event.prevent_default();
            state.set_active(Some(active.map_or(0, |row| (row + 1) % count)));
        }
        Key::ArrowUp => {
            event.prevent_default();
            state.set_active(Some(match active {
                None | Some(0) => count - 1,
                Some(row) => row - 1,
            }));
        }
        Key::Enter => {
            if let Some(row) = active {
                event.prevent_default();
                run_row(row);
            }
        }
        _ => {}
    }
}
