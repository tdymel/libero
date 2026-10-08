use dioxus::prelude::*;

use crate::{
    components::{
        accessibility::{VisuallyHidden, visually_hidden_sx},
        common::{
            ComboboxState, Input, Part, Parts, inset_focus_ring_sx, navigation_chord, parts_enum,
            recast_parts, use_combobox, use_name_warning,
        },
        feedback::Loader,
        layout::{Box, ScrollArea},
        overlay::{
            Dialog,
            use_modal::{ModalHandle, ModalScope, use_modal},
        },
        typography::Kbd,
    },
    hooks::{Hotkey, use_dismiss_layer, use_hotkeys, use_localization, use_theme},
    platform,
    sx::{StaticSx, Sx, sx},
    theme::{
        SPOTLIGHT_DESCRIPTION_COLOR, SPOTLIGHT_GROUP_COLOR, SPOTLIGHT_MAX_LIST_HEIGHT,
        SPOTLIGHT_PADDING, SPOTLIGHT_SEARCH_FONT_SIZE, SPOTLIGHT_TOP_OFFSET, SPOTLIGHT_WIDTH, Size,
        SizeCss,
    },
    utils::warn,
};

use super::action::{SpotlightAction, group_and_limit};

parts_enum! {
    /// The palette's inner parts, for [`SpotlightOptions::parts`]. The rows sit
    /// inside the list's scroll viewport, so they are descendants.
    pub enum SpotlightPart {
        /// Holds the search box, the list and the status line.
        Body = "body" => "& > [data-slot='body']",
        /// The search `input`.
        Search = "search" => "& > [data-slot='body'] > [data-slot='search']",
        /// The `listbox`, a `ScrollArea`.
        List = "list" => "& > [data-slot='body'] > [data-slot='list']",
        /// A named group of rows.
        Group = "group" => "& > [data-slot='body'] > [data-slot='list'] [data-slot='group']",
        GroupLabel = "group-label" => "& > [data-slot='body'] > [data-slot='list'] [data-slot='group'] > [data-slot='group-label']",
        /// One action row; the highlighted one has `data-active`.
        Option = "option" => "& > [data-slot='body'] > [data-slot='list'] [data-slot='option']",
        Icon = "icon" => "& > [data-slot='body'] > [data-slot='list'] [data-slot='option'] > [data-slot='icon']",
        /// The label and description column.
        Text = "text" => "& > [data-slot='body'] > [data-slot='list'] [data-slot='option'] > [data-slot='text']",
        Label = "label" => "& > [data-slot='body'] > [data-slot='list'] [data-slot='option'] > [data-slot='text'] > [data-slot='label']",
        Description = "description" => "& > [data-slot='body'] > [data-slot='list'] [data-slot='option'] > [data-slot='text'] > [data-slot='description']",
        /// The key hint, one `Kbd` per key.
        Shortcut = "shortcut" => "& > [data-slot='body'] > [data-slot='list'] [data-slot='option'] > [data-slot='shortcut']",
        /// "Nothing found" and the loader.
        Status = "status" => "& > [data-slot='body'] > [data-slot='status']",
    }
}

// Rows are styled from here, the `Menu` shape. Hover tints like the keyboard's row
// (muted.1 on white is invisible, todo 2628); only the keyboard's row takes a ring.
static SPOTLIGHT_BODY_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(SPOTLIGHT_PADDING.value())
        .selector(
            "& > [data-slot='search']",
            sx().width("100%")
                .padding("12px")
                .font("inherit")
                .letter_spacing("inherit")
                .font_size(SPOTLIGHT_SEARCH_FONT_SIZE.value())
                .color("inherit")
                .background("transparent")
                .border("0")
                .border_bottom("2px solid var(--lsx-muted-3)")
                .outline("none"),
        )
        // The input always holds focus: its indicator is the underline, not a ring.
        .selector(
            "& > [data-slot='search']:focus",
            sx().border_bottom_color("primary.6"),
        )
        .selector(
            "& [data-slot='group'] > [data-slot='group-label']",
            sx().padding("8px 12px 4px")
                .font_size(Size::Xs)
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
        .selector("& [role=\"option\"]:hover", sx().background("muted.2"))
        .selector(
            "& [role=\"option\"][data-active]",
            sx().background("muted.2"),
        )
        .selector(
            "& [role=\"option\"][data-active]",
            inset_focus_ring_sx("-2px"),
        )
        .selector(
            "& [data-slot='option'] > [data-slot='icon']",
            sx().display("inline-flex").align_items("center"),
        )
        .selector(
            "& [data-slot='option'] > [data-slot='text']",
            sx().display("flex")
                .flex_direction("column")
                .flex("1")
                .min_width("0")
                .with("overflow-wrap", "anywhere"),
        )
        .selector(
            "& [data-slot='option'] > [data-slot='shortcut']",
            sx().display("inline-flex")
                .align_items("center")
                .gap("4px")
                .white_space("nowrap"),
        )
        .selector(
            "& [data-slot='text'] > [data-slot='description']",
            sx().font_size("0.8125rem")
                .color(SPOTLIGHT_DESCRIPTION_COLOR.value()),
        )
        .selector(
            "& [role=\"status\"]",
            sx().padding("12px").text_align("center"),
        )
        // Not `display: none`: a region that appears with its text may go unheard
        // (todo 2379). A spoken count alone takes no room.
        .selector(
            "& [role=\"status\"]:not([data-shown])",
            visually_hidden_sx(),
        )
});

/// How a [`use_spotlight`] palette behaves. Set `actions`, or it warns and stays empty.
#[derive(Clone)]
pub struct SpotlightOptions {
    /// The live query to what to show; see [`spotlight_filter`](super::spotlight_filter).
    /// The closure is stored: capture a `Signal`, not a `Vec`, if the list changes.
    pub actions: Option<Callback<String, Vec<SpotlightAction>>>,
    /// Also names the search box; unset, both come from the localization.
    pub placeholder: Option<String>,
    /// Drawn and announced when a non-empty query matches nothing.
    pub nothing_found: Option<Element>,
    /// A cap on the rows drawn, counted through the groups. The spoken count stays the matched one.
    pub limit: Option<usize>,
    /// Close after running an action.
    pub close_on_action: bool,
    /// Start every opening with an empty query.
    pub clear_on_close: bool,
    /// Names the dialog and its list.
    pub aria_label: Option<String>,
    /// Ctrl/Cmd plus this key toggles the palette from anywhere. Web only: no
    /// other renderer has a document-level key listener yet.
    pub shortcut: Option<char>,
    /// The actions are still being fetched: a loader replaces the rows.
    pub loading: bool,
    /// Highlight the first row after every keystroke, so `Enter` runs it.
    pub highlight_first_on_query: bool,
    /// Called with the new query on every keystroke, from the input event. A newer
    /// call supersedes the older: cancel a search still running for it.
    pub onquery: Option<Callback<String>>,
    /// Styles the dialog box.
    pub sx: Input<Sx>,
    /// Styles the palette's inner parts, under `sx`.
    pub parts: Input<Parts<SpotlightPart>>,
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
            sx: Input::None,
            parts: Input::None,
        }
    }
}

/// Opens and closes a [`use_spotlight`] palette.
#[derive(Clone, Copy)]
pub struct SpotlightHandle {
    modal: ModalHandle<()>,
    query: Signal<String>,
    state: ComboboxState,
    clear_on_close: bool,
}

impl SpotlightHandle {
    /// Opens the palette. Call it from the trigger's handler, so focus returns there.
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

/// A command palette: a modal search box over a list of actions.
///
/// Call it in a component that outlives every trigger: the palette portals from there.
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
///
/// Docs: <https://libero-ui.dev/overlay/spotlight>
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
    // The last render's row count, and the key generation it drew with.
    let layout_generation = use_hook(|| CopyValue::new((0usize, 0u64)));
    let pressed_input = use_hook(|| CopyValue::new(false));
    let modal = use_modal(move |scope: ModalScope<()>| {
        let options = &options_for_render;
        let id = state.id();
        let listbox = format!("{id}-listbox");
        let actions = options
            .actions
            .map(|actions| actions.call(query()))
            .unwrap_or_default();
        let loading = options.loading;
        // While fetching, the rows are the last query's: neither drawn nor reachable.
        let actions = match loading {
            true => Vec::new(),
            false => actions,
        };
        // Spoken: what the query matched, not what `limit` let through (todo 2383).
        let matched = actions.len();
        let groups = group_and_limit(actions, options.limit);
        let count = groups
            .iter()
            .map(|(_, members)| members.len())
            .sum::<usize>();
        state.set_rows(count);
        // A narrowed list gets fresh rows: Blitz keeps a surviving row's
        // min-content text layout and wraps its label (todo 627).
        let mut layout_generation = layout_generation;
        let (last_count, mut generation) = *layout_generation.peek();
        if count < last_count {
            generation = generation.wrapping_add(1);
        }
        layout_generation.set((count, generation));
        let active = state
            .active()
            .filter(|_| count > 0)
            .map(|row| row.min(count - 1));

        let run = move |callback: Option<Callback<()>>| {
            if let Some(callback) = callback {
                callback.call(());
            }
            if close_on_action {
                state.close();
                // Out of this dispatch: the event still bubbles through the modal.
                spawn(async move { scope.close() });
            }
        };

        let (source, limit) = (options.actions, options.limit);
        let drawn_query = query.peek().clone();
        // A key can land before the render of the input event before it (todo 2388):
        // the rows come from the live query, as `ComboboxCore` reads `active_now()`.
        let onkeydown = move |event: KeyboardEvent| {
            if !matches!(event.key(), Key::ArrowDown | Key::ArrowUp | Key::Enter) {
                return;
            }
            // An `onquery` caller has not answered a newer query's `loading` yet (todo 2408).
            let unanswered = onquery.is_some() && *query.peek() != drawn_query;
            let actions = match (loading || unanswered, source) {
                (false, Some(source)) => source.call(query.peek().clone()),
                _ => Vec::new(),
            };
            let callbacks: Vec<Option<Callback<()>>> = group_and_limit(actions, limit)
                .into_iter()
                .flat_map(|(_, members)| members.into_iter().map(|action| action.onclick))
                .collect();
            let count = callbacks.len();
            let active = state
                .active_now()
                .filter(|_| count > 0)
                .map(|row| row.min(count - 1));
            spotlight_key(event, state, active, count, move |row| run(callbacks[row]));
        };

        let rows = spotlight_rows(groups, &id, generation, active, run);

        let filtered = !loading && !query().trim().is_empty();
        let empty = filtered && count == 0;
        // What a query left, said but not shown (WCAG 4.1.3, todo 1574).
        let results = (filtered && count > 0).then(|| (labels.results)(matched));
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
        // A given placeholder says what this palette searches, so it names the box too (todo 2380).
        let search_label = options
            .placeholder
            .clone()
            .unwrap_or_else(|| labels.search.to_string());
        let placeholder = options
            .placeholder
            .clone()
            .unwrap_or_else(|| labels.placeholder.to_string());
        // `ComboboxCore`'s linear map: the active row lands in view, nothing measured.
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
                    .padding(SPOTLIGHT_PADDING.value())
                    .and(options.sx.as_ref().cloned().unwrap_or_default()),
                parts: recast_parts(options.parts.clone()),
                // A press off the input would focus the list or the dialog (WCAG 2.4.3,
                // todo 2378); the input's own press still places the caret.
                onmousedown: move |event: MouseEvent| {
                    let mut pressed_input = pressed_input;
                    if !pressed_input.replace(false) {
                        event.prevent_default();
                    }
                },
                Box { framework_sx: &SPOTLIGHT_BODY_SX, "data-slot": SpotlightPart::Body.slot(),
                    input {
                        onmousedown: move |_| {
                            let mut pressed_input = pressed_input;
                            pressed_input.set(true);
                        },
                        "data-slot": SpotlightPart::Search.slot(),
                        r#type: "text",
                        autocomplete: "off",
                        spellcheck: "false",
                        "data-autofocus": "true",
                        // A WebView's trap cannot find it by `data-autofocus`, and the
                        // soft keyboard comes up only for a focused box (todo 1014).
                        onmounted: move |event: MountedEvent| {
                            let _ = platform::element(&event.data()).focus();
                        },
                        "aria-autocomplete": "list",
                        // Its own name: the dialog and the listbox carry `aria_label`.
                        "aria-label": "{search_label}",
                        placeholder: "{placeholder}",
                        value: "{query}",
                        oninput: move |event: FormEvent| {
                            let mut query = query;
                            query.set(event.value());
                            if let Some(onquery) = onquery {
                                onquery.call(event.value());
                            }
                            // Clamped against the new count, so `Some(0)` on
                            // an empty result is no highlight.
                            state.set_active(highlight_first.then_some(0));
                        },
                        onkeydown,
                        // The listbox is drawn while open, rows or none.
                        ..state.aria(true),
                    }
                    ScrollArea {
                        id: "{listbox}",
                        "data-slot": SpotlightPart::List.slot(),
                        "role": "listbox",
                        "aria-label": "{aria_label}",
                        sx: sx().max_height(SPOTLIGHT_MAX_LIST_HEIGHT.value()),
                        scroll_position_y: scroll_y,
                        "aria-busy": loading.then_some("true"),
                        {rows.into_iter()}
                    }
                    // Always mounted, and outside the busy listbox some screen
                    // readers hold back, so "nothing found" and "loading" are heard.
                    div {
                        "role": "status",
                        "data-slot": SpotlightPart::Status.slot(),
                        "data-shown": (loading || empty).then_some("true"),
                        if loading {
                            Loader { size: Size::Sm }
                            VisuallyHidden { "{labels.loading}" }
                        }
                        {nothing_found}
                        if let Some(results) = results {
                            VisuallyHidden { "{results}" }
                        }
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
    use_shortcut(handle, options.shortcut);
    handle
}

/// Ctrl/Cmd + `shortcut` toggles the palette. Closed, it listens filtered, so a
/// text field keeps the chord; open, unfiltered, as focus sits in the search box.
/// Two bindings and a guard on `open` per press, so no render reads `open`.
fn use_shortcut(handle: SpotlightHandle, shortcut: Option<char>) {
    let layer = use_dismiss_layer();
    // Both Ctrl and Cmd on every platform, not `mod`: a Linux keyboard may have a Meta key.
    let bindings = shortcut.into_iter().flat_map(|key| {
        ["ctrl", "meta"].into_iter().flat_map(move |modifier| {
            let chord = format!("{modifier}+{key}");
            [
                // Something else is open above the page: not ours to cover.
                Hotkey::new(chord.clone(), move || handle.toggle())
                    .when(move || !handle.modal.is_open_untracked() && !layer.any_open()),
                Hotkey::new(chord, move || handle.toggle())
                    .include_editable(true)
                    .when(move || handle.modal.is_open_untracked()),
            ]
        })
    });
    use_hotkeys(bindings);
}

/// The action rows, grouped. A group label is presentational: a `listbox` holds
/// only options and groups. A new `generation` remounts the rows.
fn spotlight_rows(
    groups: Vec<(Option<String>, Vec<SpotlightAction>)>,
    id: &str,
    generation: u64,
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
                        key: "{generation}-{row}",
                        id: "{option_id}",
                        "data-slot": SpotlightPart::Option.slot(),
                        "role": "option",
                        "aria-labelledby": "{option_id}-label",
                        "aria-describedby": (!described.is_empty()).then_some(described),
                        "data-active": (active == Some(row)).then_some("true"),
                        // Or the click takes focus out of the search box.
                        onmousedown: move |event: MouseEvent| event.prevent_default(),
                        onclick: move |_| run(onclick),
                        if let Some(icon) = action.icon {
                            span { "data-slot": SpotlightPart::Icon.slot(), {icon} }
                        }
                        span { "data-slot": SpotlightPart::Text.slot(),
                            span { id: "{option_id}-label", "data-slot": SpotlightPart::Label.slot(), "{action.label}" }
                            if let Some(description) = action.description {
                                span {
                                    id: "{option_id}-description",
                                    "data-slot": SpotlightPart::Description.slot(),
                                    "{description}"
                                }
                            }
                        }
                        if let Some(shortcut) = action.shortcut {
                            span { id: "{option_id}-shortcut", "data-slot": SpotlightPart::Shortcut.slot(),
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
                        key: "group-{generation}-{group_index}",
                        "role": "group",
                        "data-slot": SpotlightPart::Group.slot(),
                        "aria-labelledby": "{label_id}",
                        div {
                            id: "{label_id}",
                            "role": "presentation",
                            "data-slot": SpotlightPart::GroupLabel.slot(),
                            "{group}"
                        }
                        {options_rows.into_iter()}
                    }
                }
            }
            // Bare rows, as in `Combobox`: an unnamed group adds nothing.
            None => rsx! {
                Fragment { key: "group-{generation}-{group_index}", {options_rows.into_iter()} }
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

/// The search box's keys: the arrows wrap through the rows, Enter runs the highlight.
fn spotlight_key(
    event: KeyboardEvent,
    state: ComboboxState,
    active: Option<usize>,
    count: usize,
    run_row: impl Fn(usize),
) {
    // A chord is the caret's or the browser's, a composing key the IME's (todo 2377).
    if count == 0 || event.is_composing() || navigation_chord(&event).is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let body = "& > [data-slot='body']";
        let list = format!("{body} > [data-slot='list']");
        let option = format!("{list} [data-slot='option']");
        let expected: Vec<(&str, String)> = vec![
            ("body", body.into()),
            ("search", format!("{body} > [data-slot='search']")),
            ("list", list.clone()),
            ("group", format!("{list} [data-slot='group']")),
            (
                "group-label",
                format!("{list} [data-slot='group'] > [data-slot='group-label']"),
            ),
            ("option", option.clone()),
            ("icon", format!("{option} > [data-slot='icon']")),
            ("text", format!("{option} > [data-slot='text']")),
            (
                "label",
                format!("{option} > [data-slot='text'] > [data-slot='label']"),
            ),
            (
                "description",
                format!("{option} > [data-slot='text'] > [data-slot='description']"),
            ),
            ("shortcut", format!("{option} > [data-slot='shortcut']")),
            ("status", format!("{body} > [data-slot='status']")),
        ];
        let table: Vec<(&str, String)> = part_table::<SpotlightPart>()
            .into_iter()
            .map(|(slot, selector)| (slot, selector.to_string()))
            .collect();

        assert_eq!(table, expected);
    }
}
