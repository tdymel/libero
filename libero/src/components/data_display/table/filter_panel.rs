use std::{collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::FilterKind,
    column_filter::{ColumnFilter, FilterLogic, FilterOperator},
    filter_popover::{Choice, FILTER_POPOVER_SX, FilterTarget, FilterValue, operator_choices},
    use_table::StateSlice,
};
use crate::{
    components::{
        buttons::{ActionIcon, Button},
        common::{FOCUSABLE_SELECTOR, Glyph, HtmlTag, Options, attr},
        data_display::{Badge, Icon},
        form::NativeSelect,
        layout::use_box,
    },
    context::IconSlot,
    hooks::{
        Align, ElementHandle, PopoverOptions, Side, id_selector, listener, use_debounced_callback,
        use_element, use_id, use_popover, use_theme,
    },
    localization::TableLabels,
    platform::{ElementApi, PlatformError, focus_selector, next_task},
    sx::{StaticSx, sx},
    theme::{Size, SizeCss},
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
}

impl Options for PanelColumn {
    fn label(&self) -> String {
        self.header.clone()
    }

    fn value(&self) -> String {
        self.header.clone()
    }
}

/// What takes the focus once the panel's lines render.
#[derive(Clone, Copy, PartialEq)]
enum PanelFocus {
    Line(usize),
    Add,
}

/// The panel's open state and its button, shared with the column menus.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct PanelControl {
    open: Signal<bool>,
    anchor: ElementHandle,
    focus: Signal<Option<PanelFocus>>,
}

pub(super) fn use_panel_control() -> PanelControl {
    PanelControl {
        open: use_signal(|| false),
        anchor: use_element(),
        focus: use_signal(|| None),
    }
}

impl PanelControl {
    /// Opens on `target`'s line, adding one when the column has none.
    pub fn open_for(mut self, target: &FilterTarget) {
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
}

/// The toolbar's Filters button, with the active filters' count.
#[component]
pub(super) fn FilterPanelButton(
    control: PanelControl,
    labels: TableLabels,
    size: Size,
    active: usize,
) -> Element {
    let mut open = control.open;
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
            onclick: move |_| open.toggle(),
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
    } = control;
    let box_id = use_id();
    // Typed values by line, applied together once typing settles.
    let mut drafts = use_signal(BTreeMap::<usize, String>::new);
    let mut flush = move || {
        let pending = std::mem::take(&mut *drafts.write());
        if pending.is_empty() {
            return;
        }
        let mut filters = slice.peek();
        for (line, text) in pending {
            if let Some(item) = filters.get_mut(line) {
                item.value = text;
            }
        }
        slice.set(filters);
        onfilter.call(());
    };
    let apply = use_debounced_callback(move |()| flush(), 300);
    let edit = move |line: usize, change: Box<dyn FnOnce(&mut ColumnFilter)>| {
        let mut filters = slice.peek();
        if let Some(item) = filters.get_mut(line) {
            change(item);
            slice.set(filters);
            onfilter.call(());
        }
    };
    let popover = use_popover(
        anchor,
        open(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::Start)
            .dismiss(true),
    );
    let close = move |refocus: bool| {
        open.set(false);
        if refocus {
            let _ = anchor.focus();
        }
    };
    let mut dismiss = close;
    popover.on_dismiss(move || dismiss(true));
    let floating = *popover.floating();
    let filters = slice.read();
    // Opened from the button: the first line, else Add. Once per opening: a
    // re-placement (natively a picker's listbox opening) must not move focus.
    let mut entered = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        let (open, placed) = (open(), popover.placed());
        if !open {
            entered.set(false);
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
    let shown_drafts = drafts.read().clone();
    let lines: Vec<Element> = filters
        .iter()
        .enumerate()
        .map(|(line, item)| {
            let kind = kind_of(&item.column);
            let header = item.column.clone();
            let picked = columns.iter().find(|column| column.header == header).cloned();
            let (operators, operator) = operator_choices(kind, item.operator, labels);
            rsx! {
                div { key: "{line}", "data-filter-line": "{line}",
                    NativeSelect {
                        aria_label: (labels.filter_line_column)(&header),
                        size,
                        value: picked,
                        options: columns.to_vec(),
                        onchange: move |column: PanelColumn| {
                            edit(line, Box::new(move |item| {
                                *item = ColumnFilter::new(column.header, FilterOperator::of(column.kind)[0], "");
                            }));
                        },
                    }
                    if kind != FilterKind::Boolean {
                        NativeSelect {
                            aria_label: (labels.filter_line_operator)(&header),
                            size,
                            value: operator,
                            options: operators,
                            onchange: move |choice: Choice<FilterOperator>| {
                                flush();
                                edit(line, Box::new(move |item| item.operator = choice.value));
                            },
                        }
                    }
                    FilterValue {
                        column: header.clone(),
                        kind,
                        labels,
                        size,
                        filter: Some(item.clone()),
                        draft: shown_drafts.get(&line).cloned(),
                        name: (labels.filter_line_value)(&header),
                        ontext: move |text: String| {
                            drafts.write().insert(line, text);
                            apply.call(());
                        },
                        onpick: move |on: Option<bool>| {
                            edit(line, Box::new(move |item| {
                                item.operator = FilterOperator::Is;
                                item.value = on.map(|on| on.to_string()).unwrap_or_default();
                            }));
                        },
                        onday: move |(to, text): (bool, String)| {
                            edit(line, Box::new(move |item| match to {
                                true => item.value_to = text,
                                false => item.value = text,
                            }));
                        },
                    }
                    ActionIcon {
                        aria_label: (labels.remove_filter)(&header),
                        variant: "standard",
                        size,
                        attributes: vec![attr("data-filter-remove", true)],
                        onclick: move |_| {
                            flush();
                            // Focus moves first: a focused node going would dismiss the panel.
                            let next = match line {
                                0 => "[data-filter-add]".to_string(),
                                _ => format!(
                                    "[data-filter-line=\"{}\"] :is(select, [role=combobox])",
                                    line - 1
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
            .map(|(value, key)| Choice {
                value,
                key,
                label: (labels.logic_name)(value),
            })
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
        attributes.push(attr("style", popover.style()));
        attributes.extend(popover.floating_events());
        let mut close = close;
        panel
            .element(&floating)
            .event("onkeydown", move |event: KeyboardEvent| {
                if event.key() != Key::Tab {
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
