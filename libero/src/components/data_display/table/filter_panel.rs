use std::{collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::FilterKind,
    column_filter::{CellTest, ColumnFilter, FilterLogic, FilterOperator, SelectOption, filter_of},
    filter_popover::{
        FILTER_POPOVER_SX, FilterTarget, FilterValue, holds_something, operator_choices,
    },
    use_table::StateSlice,
};
use crate::{
    components::{
        buttons::{ActionIcon, Button},
        common::{Choice, FOCUSABLE_SELECTOR, Glyph, HtmlTag, Options, attr},
        data_display::{Badge, Icon},
        form::NativeSelect,
        layout::use_box,
    },
    context::IconSlot,
    hooks::{
        Align, ElementHandle, PopoverOptions, Side, id_selector, listener, use_debounced_callback,
        use_element, use_formats, use_id, use_popover, use_theme,
    },
    localization::TableLabels,
    platform::{ElementApi, PlatformError, focus_selector, is_tab, next_task},
    sx::{StaticSx, sx},
    theme::{ColorCss, ColorShade, DIVIDER_THICKNESS, Size, SizeCss},
};

static FILTER_PANEL_SX: StaticSx = StaticSx::new(|| {
    FILTER_POPOVER_SX
        .clone()
        .max_width("min(40rem, calc(100vw - 16px))")
        // One line a row: column, operator and value share it, remove at the end.
        .selector(
            "& > [data-filter-line]",
            sx().display("flex")
                .flex_wrap("wrap")
                .align_items("center")
                .gap(SizeCss::SPACING.value(Size::Xs)),
        )
        // A rule between lines: wrapped at 375 px, one line's two rows read as one (5152).
        .selector(
            "& > [data-filter-line] + [data-filter-line]",
            sx().border_top(format!(
                "{} solid {}",
                DIVIDER_THICKNESS.value(Size::Xs),
                ColorCss::MUTED.value(ColorShade::S4)
            ))
            .padding_top(SizeCss::SPACING.value(Size::Sm)),
        )
        .selector(
            "& > [data-filter-line] > *",
            sx().flex("1 1 8rem").min_width("0"),
        )
        .selector(
            "& > [data-filter-line] > [data-filter-remove]",
            sx().flex("none"),
        )
});

/// A column the panel's lines can filter.
#[derive(Clone, PartialEq)]
pub(super) struct PanelColumn {
    pub header: String,
    pub kind: FilterKind,
    pub options: Vec<SelectOption>,
}

impl Options for PanelColumn {
    fn label(&self) -> String {
        self.header.clone()
    }

    fn value(&self) -> String {
        self.header.clone()
    }
}

/// A panel-local id per line, in the lines' order: the rsx key and the draft's key (5151).
#[derive(Clone, Copy)]
struct LineIds(CopyValue<(Vec<u64>, u64)>);

impl LineIds {
    /// The ids for `len` lines: new ones for lines added at the end since.
    fn sync(self, len: usize) -> Vec<u64> {
        let mut inner = self.0;
        let mut held = inner.write();
        let (ids, next) = &mut *held;
        ids.truncate(len);
        while ids.len() < len {
            *next += 1;
            ids.push(*next);
        }
        ids.clone()
    }

    fn line(self, id: u64) -> Option<usize> {
        self.0.read().0.iter().position(|&each| each == id)
    }

    fn remove(self, line: usize) {
        let mut inner = self.0;
        let ids = &mut inner.write().0;
        if line < ids.len() {
            ids.remove(line);
        }
    }

    /// Keeps the lines whose `keep` is true.
    fn retain(self, keep: &[bool]) {
        let mut inner = self.0;
        let ids = &mut inner.write().0;
        let mut keep = keep.iter();
        ids.retain(|_| keep.next().copied().unwrap_or(true));
    }
}

/// The line that takes the focus when `line` of `len` goes, by its index now: the next
/// one, which moves into its place, else the one before. `None`: no line remains.
fn taking_place(line: usize, len: usize) -> Option<usize> {
    match len <= 1 {
        true => None,
        false if line + 1 < len => Some(line + 1),
        false => line.checked_sub(1),
    }
}

/// What takes the focus once the panel's lines render.
#[derive(Clone, Copy, PartialEq)]
enum PanelFocus {
    Line(usize),
    Add,
}

/// The control a column opened the panel from, which takes the focus back on close.
#[derive(Clone, PartialEq)]
struct PanelOpener {
    button: ElementHandle,
    /// Takes it instead once the column's filter is gone: a filtered-header button goes with it.
    fallback: ElementHandle,
    column: String,
    kind: FilterKind,
}

/// The panel's open state and its button, shared with the column menus.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct PanelControl {
    open: Signal<bool>,
    anchor: ElementHandle,
    focus: Signal<Option<PanelFocus>>,
    /// `None`: opened from the toolbar's Filters button.
    opener: Signal<Option<PanelOpener>>,
}

pub(super) fn use_panel_control() -> PanelControl {
    PanelControl {
        open: use_signal(|| false),
        anchor: use_element(),
        focus: use_signal(|| None),
        opener: use_signal(|| None),
    }
}

impl PanelControl {
    /// Opens on `target`'s line from `button`, adding a line when the column has none.
    /// Focus goes back to `button` on close, or to `fallback` once the filter is gone.
    pub fn open_for(
        mut self,
        target: &FilterTarget,
        button: ElementHandle,
        fallback: ElementHandle,
    ) {
        self.opener.set(Some(PanelOpener {
            button,
            fallback,
            column: target.column.clone(),
            kind: target.kind,
        }));
        let mut filters = target.slice.peek();
        let line = match filters.iter().position(|item| item.column == target.column) {
            Some(line) => line,
            None => {
                let operator = FilterOperator::of(target.kind)[0];
                filters.push(ColumnFilter::new(target.column.clone(), operator, ""));
                target.slice.set(filters.clone());
                filters.len() - 1
            }
        };
        self.focus.set(Some(PanelFocus::Line(line)));
        self.open.set(true);
    }

    /// Whether the panel is open from `button`, for its `aria-expanded`. Reactive.
    pub fn expanded_from(&self, button: ElementHandle) -> bool {
        (self.open)()
            && self
                .opener
                .read()
                .as_ref()
                .is_some_and(|opener| opener.button == button)
    }
}

/// The toolbar's Filters button, with the active filters' count.
#[component]
pub(super) fn FilterPanelButton(
    control: PanelControl,
    labels: TableLabels,
    size: Size,
    active: usize,
) -> Element {
    let (mut open, mut opener) = (control.open, control.opener);
    let mut attributes = vec![
        attr("aria-haspopup", "dialog"),
        attr("aria-expanded", open().to_string()),
        attr("aria-label", (labels.active_filters)(active)),
        attr("data-filter-panel-button", true),
    ];
    attributes.push(listener("onmounted", control.anchor.mount()));
    rsx! {
        Button {
            variant: "standard",
            size,
            attributes,
            onclick: move |_| {
                opener.set(None);
                open.toggle();
            },
            icon: rsx! {
                Icon { variant: "standard", size: "sm", color: "inherit",
                    Glyph { slot: IconSlot::Filter, icon: lucide::funnel::outlined }
                }
            },
            "{labels.filters}"
            if active > 0 {
                span { "aria-hidden": "true", "data-filter-count": true,
                    Badge { size: "xs", "{active}" }
                }
            }
        }
    }
}

/// Every column filter as a line to edit, joined by the filter logic: the
/// `filter_panel` dialog, anchored to its button. Lives in the table's tree.
#[component]
pub(super) fn FilterPanel(
    control: PanelControl,
    columns: Rc<[PanelColumn]>,
    slice: StateSlice<Vec<ColumnFilter>>,
    logic: StateSlice<FilterLogic>,
    labels: TableLabels,
    size: Size,
    onfilter: Callback<()>,
) -> Element {
    let theme = use_theme();
    let PanelControl {
        mut open,
        anchor,
        mut focus,
        opener,
    } = control;
    let box_id = use_id();
    let ids = LineIds(use_hook(|| CopyValue::new((Vec::new(), 0))));
    // Typed values by line id, applied together once typing settles.
    let mut drafts = use_signal(BTreeMap::<u64, String>::new);
    let mut flush = move || {
        let pending = std::mem::take(&mut *drafts.write());
        if pending.is_empty() {
            return;
        }
        let mut filters = slice.peek();
        for (id, text) in pending {
            if let Some(item) = ids.line(id).and_then(|line| filters.get_mut(line)) {
                item.value = text;
            }
        }
        slice.set(filters);
        onfilter.call(());
    };
    let apply = use_debounced_callback(move |()| flush(), 300);
    let edit = move |id: u64, change: Box<dyn FnOnce(&mut ColumnFilter)>| {
        let mut filters = slice.peek();
        if let Some(item) = ids.line(id).and_then(|line| filters.get_mut(line)) {
            change(item);
            slice.set(filters);
            onfilter.call(());
        }
    };
    let formats = use_formats();
    // The column's control it opened from, else the Filters button (5148).
    let back = use_memo(move || match &*opener.read() {
        None => anchor,
        Some(PanelOpener {
            button,
            fallback,
            column,
            kind,
        }) => {
            let kept = filter_of(&slice.read(), column)
                .is_some_and(|item| CellTest::new(item, *kind, formats).is_some());
            if kept { *button } else { *fallback }
        }
    });
    let popover = use_popover(
        anchor,
        open(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::Start)
            .remeasure(slice.read().len() as u64)
            .dismiss(true),
    );
    use_effect(move || {
        if open() && popover.placed() {
            popover.return_focus_to(back());
        }
    });
    let close = move |refocus: bool| {
        open.set(false);
        if refocus {
            let _ = back.peek().focus();
        }
    };
    let mut dismiss = close;
    // The shared focus return decides: a focus moved elsewhere stays (3551).
    popover.on_dismiss(move || dismiss(false));
    let floating = *popover.floating();
    // Opened from the button: the first line, else Add. Once per opening: a
    // re-placement (natively a picker's listbox opening) must not move focus.
    let mut entered = use_hook(|| CopyValue::new(false));
    let prune_columns = columns.clone();
    use_effect(move || {
        let (open, placed) = (open(), popover.placed());
        if !open {
            // Lines a column's Filter added and nobody filled go with the close (5149).
            if entered.replace(false) {
                flush();
                let filters = slice.peek();
                let keep: Vec<bool> = filters
                    .iter()
                    .map(|item| {
                        let kind = prune_columns
                            .iter()
                            .find(|column| column.header == item.column)
                            .map_or(FilterKind::Text, |column| column.kind);
                        holds_something(item, kind)
                    })
                    .collect();
                if keep.contains(&false) {
                    ids.retain(&keep);
                    let mut keep = keep.into_iter();
                    let kept = filters
                        .into_iter()
                        .filter(|_| keep.next() == Some(true))
                        .collect();
                    slice.set(kept);
                }
            }
            return;
        }
        if !placed || entered.replace(true) {
            return;
        }
        if focus.peek().is_none() {
            let first = match slice.peek().is_empty() {
                true => PanelFocus::Add,
                false => PanelFocus::Line(0),
            };
            focus.set(Some(first));
        }
    });
    use_effect(move || {
        let Some(target) = focus() else {
            return;
        };
        if !(open() && popover.placed()) {
            return;
        }
        let selector = match target {
            // Natively `NativeSelect` draws a listbox combobox, no `<select>`.
            PanelFocus::Line(line) => {
                format!("[data-filter-line=\"{line}\"] :is(select, [role=combobox])")
            }
            PanelFocus::Add => "[data-filter-add]".to_string(),
        };
        // The lines render in a portal after this pass.
        spawn(async move {
            for _ in 0..5 {
                next_task().await;
                match floating.query_selector(&selector) {
                    Ok(element) => {
                        let _ = element.focus();
                        break;
                    }
                    // A WebView queries nothing: the page focuses it inside the box (958).
                    Err(PlatformError::Unsupported) => {
                        let inside = format!("{} {selector}", id_selector(&box_id.peek()));
                        let _ = focus_selector(&inside);
                        break;
                    }
                    Err(_) => {}
                }
            }
            focus.set(None);
        });
    });

    let kind_of = |column: &str| {
        columns
            .iter()
            .find(|candidate| candidate.header == column)
            .map_or(FilterKind::Text, |column| column.kind)
    };
    let filters = slice.read();
    let line_ids = ids.sync(filters.len());
    let shown_drafts = drafts.read().clone();
    let lines: Vec<Element> = filters
        .iter()
        .zip(line_ids)
        .enumerate()
        .map(|(line, (item, id))| {
            let kind = kind_of(&item.column);
            let header = item.column.clone();
            let picked = columns.iter().find(|column| column.header == header).cloned();
            let options = picked.as_ref().map(|column| column.options.clone()).unwrap_or_default();
            let (operators, operator) = operator_choices(kind, item.operator, labels);
            rsx! {
                div {
                    key: "{id}",
                    role: "group",
                    aria_label: header.clone(),
                    "data-filter-line": "{line}",
                    NativeSelect {
                        // Not the caller's form data: a surrounding Fieldset's `required` skips it (4018).
                        required: false,
                        aria_label: (labels.filter_line_column)(&header),
                        size,
                        value: picked,
                        options: columns.to_vec(),
                        onchange: move |column: PanelColumn| {
                            // The new column starts empty: text typed for the old one is dropped.
                            drafts.write().remove(&id);
                            edit(id, Box::new(move |item| {
                                *item = ColumnFilter::new(column.header, FilterOperator::of(column.kind)[0], "");
                            }));
                        },
                    }
                    if kind != FilterKind::Boolean {
                        NativeSelect {
                            required: false,
                            aria_label: (labels.filter_line_operator)(&header),
                            size,
                            value: operator,
                            options: operators,
                            onchange: move |choice: Choice<FilterOperator>| {
                                flush();
                                edit(id, Box::new(move |item| item.operator = choice.value));
                            },
                        }
                    }
                    FilterValue {
                        column: header.clone(),
                        kind,
                        labels,
                        size,
                        filter: Some(item.clone()),
                        draft: shown_drafts.get(&id).cloned(),
                        name: (labels.filter_line_value)(&header),
                        ontext: move |text: String| {
                            drafts.write().insert(id, text);
                            apply.call(());
                        },
                        onpick: move |on: Option<bool>| {
                            edit(id, Box::new(move |item| {
                                item.operator = FilterOperator::Is;
                                item.value = on.map(|on| on.to_string()).unwrap_or_default();
                            }));
                        },
                        onday: move |(to, text): (bool, String)| {
                            edit(id, Box::new(move |item| match to {
                                true => item.value_to = text,
                                false => item.value = text,
                            }));
                        },
                        options,
                        onoption: move |option: Option<String>| {
                            edit(id, Box::new(move |item| item.value = option.unwrap_or_default()));
                        },
                        onany: move |values: Vec<String>| {
                            edit(id, Box::new(move |item| item.values = values));
                        },
                    }
                    ActionIcon {
                        aria_label: (labels.remove_filter)(&header),
                        variant: "standard",
                        size,
                        attributes: vec![attr("data-filter-remove", true)],
                        onclick: move |_| {
                            flush();
                            let Some(line) = ids.line(id) else {
                                return;
                            };
                            let len = slice.peek().len();
                            // Focus moves first: a focused node going would dismiss the panel. The
                            // line taking this one's place, else the one before, else Add (5150).
                            let next = match taking_place(line, len) {
                                None => "[data-filter-add]".to_string(),
                                Some(next) => format!(
                                    "[data-filter-line=\"{next}\"] :is(select, [role=combobox])"
                                ),
                            };
                            match floating.query_selector(&next) {
                                Ok(next) => {
                                    let _ = next.focus();
                                }
                                // A WebView queries nothing (958).
                                Err(PlatformError::Unsupported) => {
                                    let at = id_selector(&box_id.peek());
                                    let _ = focus_selector(&format!("{at} {next}"));
                                }
                                Err(_) => {}
                            }
                            let mut filters = slice.peek();
                            if line < filters.len() {
                                filters.remove(line);
                                ids.remove(line);
                                slice.set(filters);
                                onfilter.call(());
                            }
                        },
                        Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
                    }
                }
            }
        })
        .collect();
    let logic_choices: Vec<Choice<FilterLogic>> =
        [(FilterLogic::And, "and"), (FilterLogic::Or, "or")]
            .into_iter()
            .map(|(value, key)| Choice::text(value, key, (labels.logic_name)(value)))
            .collect();
    let current_logic = logic.read();
    let logic_picked = logic_choices
        .iter()
        .find(|choice| choice.value == current_logic)
        .cloned();
    let joins = filters.len() >= 2;
    let add_columns = columns.clone();
    let panel = use_box().framework_sx(&FILTER_PANEL_SX).prepare();
    let content = open().then(|| {
        let mut attributes = vec![
            attr("id", box_id()),
            attr("role", "dialog"),
            attr("aria-label", labels.filters),
            attr("tabindex", "-1"),
            attr("data-filter-panel", true),
        ];
        attributes.push(attr("style", popover.vars()));
        attributes.extend(popover.floating_events());
        let mut close = close;
        panel
            .element(&floating)
            .event("onkeydown", move |event: KeyboardEvent| {
                if !is_tab(&event) {
                    return;
                }
                // Tab past either end leaves through the Filters button, as from it.
                let Ok(items) = floating.query_selector_all(FOCUSABLE_SELECTOR) else {
                    return;
                };
                let at = items.iter().position(|item| item.is_focused());
                let backwards = event.modifiers().shift();
                let leaves = match (at, backwards) {
                    (None, _) => true,
                    (Some(at), false) => at + 1 >= items.len(),
                    (Some(at), true) => at == 0,
                };
                if leaves {
                    close(true);
                    if backwards {
                        event.prevent_default();
                    }
                }
            })
            .render(
                HtmlTag::Div,
                attributes,
                rsx! {
                    if joins {
                        NativeSelect {
                            required: false,
                            label: labels.logic,
                            size,
                            value: logic_picked,
                            options: logic_choices,
                            onchange: move |choice: Choice<FilterLogic>| {
                                logic.set(choice.value);
                                onfilter.call(());
                            },
                        }
                    }
                    {lines.into_iter()}
                    Button {
                        size,
                        variant: "standard",
                        "data-filter-add": true,
                        disabled: add_columns.is_empty(),
                        onclick: move |_| {
                            flush();
                            let mut filters = slice.peek();
                            let Some(column) = add_columns
                                .iter()
                                .find(|column| filters.iter().all(|item| item.column != column.header))
                                .or(add_columns.first())
                            else {
                                return;
                            };
                            let operator = FilterOperator::of(column.kind)[0];
                            filters.push(ColumnFilter::new(column.header.clone(), operator, ""));
                            focus.set(Some(PanelFocus::Line(filters.len() - 1)));
                            slice.set(filters);
                        },
                        "{labels.add_filter}"
                    }
                },
            )
    });
    popover.show(content);
    rsx! {}
}

#[cfg(test)]
mod tests {
    use super::taking_place;

    /// 5150: the next line moves into a removed one's place; the last falls back to the one before.
    #[test]
    fn a_removed_line_hands_the_focus_to_the_line_taking_its_place() {
        assert_eq!(taking_place(0, 1), None);
        assert_eq!(taking_place(0, 2), Some(1));
        assert_eq!(taking_place(1, 3), Some(2));
        assert_eq!(taking_place(2, 3), Some(1));
    }
}
