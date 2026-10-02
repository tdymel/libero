use chrono::NaiveDate;
use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::FilterKind,
    column_filter::{ColumnFilter, FilterOperator, filter_of, with_filter, without_column},
    use_table::StateSlice,
};
use crate::{
    components::{
        buttons::Button,
        common::{FOCUSABLE_SELECTOR, Glyph, HtmlTag, Options, attr},
        form::{DateField, NativeSelect, TextField},
        layout::{paper_sx, use_box},
    },
    context::IconSlot,
    hooks::{
        Align, ElementHandle, PopoverOptions, Side, use_debounced_callback, use_id, use_popover,
        use_theme,
    },
    localization::TableLabels,
    platform::{ElementApi, reads_dom_synchronously, when_laid_out},
    sx::StaticSx,
    theme::{Size, SizeCss, Z_INDEX_POPOVER},
};

pub(super) static FILTER_POPOVER_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Md))
        .display("flex")
        .flex_direction("column")
        .gap(SizeCss::SPACING.value(Size::Sm))
        .min_width("14rem")
});

/// A pick in a filter's `NativeSelect`: its value, its label and a unique key.
#[derive(Clone, PartialEq)]
pub(super) struct Choice<V: Clone + PartialEq + 'static> {
    pub value: V,
    pub key: &'static str,
    pub label: &'static str,
}

impl<V: Clone + PartialEq + 'static> Options for Choice<V> {
    fn label(&self) -> String {
        self.label.to_string()
    }

    fn value(&self) -> String {
        self.key.to_string()
    }
}

/// `kind`'s operators to pick from, and `current` among them.
pub(super) fn operator_choices(
    kind: FilterKind,
    current: FilterOperator,
    labels: TableLabels,
) -> (Vec<Choice<FilterOperator>>, Option<Choice<FilterOperator>>) {
    let choices: Vec<Choice<FilterOperator>> = FilterOperator::of(kind)
        .iter()
        .map(|&operator| Choice {
            value: operator,
            key: operator_key(operator),
            label: (labels.operator_name)(operator),
        })
        .collect();
    let picked = choices
        .iter()
        .find(|choice| choice.value == current)
        .cloned();
    (choices, picked)
}

/// The operator `column` filters with now: its item's, else `kind`'s default.
fn operator_of(filters: &[ColumnFilter], column: &str, kind: FilterKind) -> FilterOperator {
    filter_of(filters, column).map_or(FilterOperator::of(kind)[0], |filter| filter.operator)
}

/// Writes `column`'s filter value, or with `to` its `value_to`, keeping the
/// rest. Emptied with the default operator, the item goes: it holds nothing worth keeping.
fn commit(
    slice: StateSlice<Vec<ColumnFilter>>,
    column: &str,
    kind: FilterKind,
    to: bool,
    value: String,
) {
    let filters = slice.peek();
    let operator = operator_of(&filters, column, kind);
    let mut next = filter_of(&filters, column)
        .cloned()
        .unwrap_or_else(|| ColumnFilter::new(column, operator, ""));
    match to {
        true => next.value_to = value,
        false => next.value = value,
    }
    let keeps = !next.value.is_empty()
        || !next.value_to.is_empty()
        || operator != FilterOperator::of(kind)[0];
    let next = keeps.then_some(next);
    if filter_of(&filters, column) != next.as_ref() {
        slice.set(with_filter(&filters, column, next));
    }
}

/// What a column filter edits, shared by the popover and the header filters row.
#[derive(Clone, PartialEq)]
pub(super) struct FilterTarget {
    pub column: String,
    pub kind: FilterKind,
    pub slice: StateSlice<Vec<ColumnFilter>>,
    pub labels: TableLabels,
    pub size: Size,
    /// Called after each change, to announce the rows left.
    pub onfilter: Callback<()>,
}

/// A column filter's edits, owned in the table's tree: the popover renders in
/// a portal, where reading the table's state would outlive its owner.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct FilterEditor {
    /// Typed text not applied yet.
    pub draft: Signal<Option<String>>,
    /// Typing: shown at once, applied once it settles.
    pub ontext: Callback<String>,
    /// A boolean pick, applied at once; `None` is any.
    pub onpick: Callback<Option<bool>>,
    /// A date field's change, applied at once: `true` for `Between`'s last day.
    pub onday: Callback<(bool, String)>,
}

pub(super) fn use_filter_editor(target: &FilterTarget) -> FilterEditor {
    let FilterTarget {
        column,
        kind,
        slice,
        onfilter,
        ..
    } = target.clone();
    let mut draft = use_signal(|| None::<String>);
    let apply = {
        let column = column.clone();
        use_debounced_callback(
            move |text: String| {
                // A Clear or an operator pick meanwhile already applied.
                if draft.peek().as_ref() != Some(&text) {
                    return;
                }
                commit(slice, &column, kind, false, text);
                draft.set(None);
                onfilter.call(());
            },
            300,
        )
    };
    let ontext = use_callback(move |text: String| {
        draft.set(Some(text.clone()));
        apply.call(text);
    });
    let onday = {
        let column = column.clone();
        use_callback(move |(to, text): (bool, String)| {
            commit(slice, &column, kind, to, text);
            onfilter.call(());
        })
    };
    let onpick = use_callback(move |on: Option<bool>| {
        let next =
            on.map(|on| ColumnFilter::new(column.clone(), FilterOperator::Is, on.to_string()));
        slice.set(with_filter(&slice.peek(), &column, next));
        onfilter.call(());
    });
    FilterEditor {
        draft,
        ontext,
        onpick,
        onday,
    }
}

/// The value half of a column filter: a text field, or a yes/no pick for a
/// boolean column. Reads nothing of the table's: `filter` and `draft` come in.
#[component]
pub(super) fn FilterValue(
    column: String,
    kind: FilterKind,
    labels: TableLabels,
    size: Size,
    /// The column's filter now.
    filter: Option<ColumnFilter>,
    draft: Option<String>,
    /// Typing, to apply once it settles.
    ontext: EventHandler<String>,
    /// A boolean pick; `None` is any.
    onpick: EventHandler<Option<bool>>,
    /// A date field's change: `true` for `Between`'s last day.
    onday: EventHandler<(bool, String)>,
    /// Shown above the field; else it is named `name`, or by `filter_column`.
    #[props(default)]
    label: Option<String>,
    #[props(default)] name: Option<String>,
) -> Element {
    let current = filter.as_ref();
    let operator = current.map_or(FilterOperator::of(kind)[0], |filter| filter.operator);
    let name = name.unwrap_or_else(|| (labels.filter_column)(&column));
    let aria_label = label.is_none().then(|| name.clone());
    if kind == FilterKind::Boolean {
        let choices = vec![
            Choice {
                value: None,
                key: "any",
                label: labels.any,
            },
            Choice {
                value: Some(true),
                key: "true",
                label: labels.yes,
            },
            Choice {
                value: Some(false),
                key: "false",
                label: labels.no,
            },
        ];
        let picked = match current.map(|filter| filter.value.as_str()) {
            Some("true") => 1,
            Some("false") => 2,
            _ => 0,
        };
        let value = choices[picked].clone();
        return rsx! {
            NativeSelect {
                label,
                aria_label,
                size,
                value,
                options: choices,
                onchange: move |choice: Choice<Option<bool>>| onpick.call(choice.value),
            }
        };
    }
    if !operator.takes_value() {
        return rsx! {};
    }
    if kind == FilterKind::Date {
        let (from, to) = current.map_or_else(Default::default, |filter| {
            (filter.value.clone(), filter.value_to.clone())
        });
        let day = |end: bool, value: String, label: Option<String>, aria_label: Option<String>| {
            if reads_dom_synchronously() {
                return rsx! {
                    DateField {
                        label,
                        aria_label,
                        size,
                        value: value.parse::<NaiveDate>().ok(),
                        "data-filter-value": (!end).then_some(true),
                        onchange: move |day: Option<NaiveDate>| {
                            // `Display` is ISO 8601, as the native input posts.
                            onday.call((end, day.map(|day| day.to_string()).unwrap_or_default()));
                        },
                    }
                };
            }
            // A WebView cannot count the portaled calendar inside the popover: its native date input then.
            rsx! {
                TextField {
                    label,
                    aria_label,
                    size,
                    value,
                    r#type: "date",
                    "data-filter-value": (!end).then_some(true),
                    oninput: move |text: String| onday.call((end, text)),
                }
            }
        };
        if operator != FilterOperator::Between {
            return day(false, from, label, aria_label);
        }
        let named = |end: &str| match &label {
            Some(_) => (Some(end.to_string()), None),
            None => (None, Some(format!("{name}, {end}"))),
        };
        let ((from_label, from_aria), (to_label, to_aria)) =
            (named(labels.date_from), named(labels.date_to));
        return rsx! {
            {day(false, from, from_label, from_aria)}
            {day(true, to, to_label, to_aria)}
        };
    }
    let text = draft
        .or_else(|| current.map(|filter| filter.value.clone()))
        .unwrap_or_default();
    rsx! {
        TextField {
            label,
            aria_label,
            size,
            value: text,
            inputmode: (kind == FilterKind::Number).then_some("decimal"),
            "data-filter-value": true,
            oninput: move |text: String| ontext.call(text),
        }
    }
}

/// A column's filter popover, anchored to its menu button, and the button a
/// filtered header shows to reopen it.
#[component]
pub(super) fn FilterPopover(
    target: FilterTarget,
    anchor: ElementHandle,
    open: Signal<bool>,
) -> Element {
    let theme = use_theme();
    let mut open = open;
    let box_id = use_id();
    let editor = use_filter_editor(&target);
    let mut draft = editor.draft;
    let popover = use_popover(
        anchor,
        open(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::End)
            .dismiss(true),
    );
    let FilterTarget {
        column,
        kind,
        slice,
        labels,
        size,
        onfilter,
    } = target;
    // Typing still settling applies after the close: the editor lives here.
    let close = move |refocus: bool| {
        open.set(false);
        if refocus {
            let _ = anchor.focus();
        }
    };
    let mut dismiss = close;
    popover.on_dismiss(move || dismiss(true));
    let floating = *popover.floating();
    // The value field takes focus once placed, else the operator picker.
    let mut entered = use_signal(|| false);
    use_effect(move || match (open(), popover.placed()) {
        (true, true) if !*entered.peek() => {
            entered.set(true);
            let target = floating
                .query_selector("input[data-filter-value]")
                .or_else(|_| floating.query_selector(":is(select, [role=combobox])"))
                .ok();
            match target {
                Some(target) => {
                    let _ = target.focus();
                }
                None => {
                    let _ = floating.focus();
                }
            }
        }
        (false, _) => entered.set(false),
        _ => {}
    });

    let filters = slice.read();
    let filter = filter_of(&filters, &column).cloned();
    let operator = operator_of(&filters, &column, kind);
    let shown_draft = draft.read().clone();
    let (operators, picked) = operator_choices(kind, operator, labels);
    let name = (labels.filter_column)(&column);
    let panel = use_box().framework_sx(&FILTER_POPOVER_SX).prepare();
    let content = open().then(|| {
        let column = column.clone();
        let clear_column = column.clone();
        let mut attributes = vec![
            attr("id", box_id()),
            attr("role", "dialog"),
            attr("aria-label", name.clone()),
            attr("tabindex", "-1"),
            attr("data-filter-popover", true),
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
                // Tab past either end leaves through the menu button, as from it.
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
                    if kind != FilterKind::Boolean {
                        NativeSelect {
                            label: labels.operator,
                            size,
                            value: picked,
                            options: operators,
                            onchange: move |choice: Choice<FilterOperator>| {
                                let filters = slice.peek();
                                let value = draft
                                    .peek()
                                    .clone()
                                    .or_else(|| filter_of(&filters, &column).map(|f| f.value.clone()))
                                    .unwrap_or_default();
                                draft.set(None);
                                let value_to = filter_of(&filters, &column)
                                    .map(|f| f.value_to.clone())
                                    .unwrap_or_default();
                                let next = ColumnFilter {
                                    value_to,
                                    ..ColumnFilter::new(column.clone(), choice.value, value)
                                };
                                slice.set(with_filter(&filters, &column, Some(next)));
                                onfilter.call(());
                            },
                        }
                    }
                    FilterValue {
                        column: clear_column.clone(),
                        kind,
                        labels,
                        size,
                        filter: filter.clone(),
                        draft: shown_draft.clone(),
                        ontext: editor.ontext,
                        onpick: editor.onpick,
                        onday: editor.onday,
                        label: labels.value.to_string(),
                    }
                    Button {
                        size,
                        disabled: filter.is_none(),
                        onclick: move |_| {
                            draft.set(None);
                            slice.set(without_column(&slice.peek(), &clear_column));
                            onfilter.call(());
                        },
                        "{labels.clear_filter}"
                    }
                },
            )
    });
    popover.show(content);
    rsx! {}
}

/// The button a filtered header shows: opens the column's filter.
#[component]
pub(super) fn FilteredButton(
    column: String,
    labels: TableLabels,
    open: Signal<bool>,
    /// Set with `filter_panel`: opens the panel instead.
    #[props(default)]
    onopen: Option<EventHandler<()>>,
) -> Element {
    let mut open = open;
    rsx! {
        button {
            r#type: "button",
            "data-filtered": true,
            aria_label: (labels.filtered)(&column),
            aria_haspopup: "dialog",
            aria_expanded: "{open()}",
            onclick: move |_| match onopen {
                Some(onopen) => onopen.call(()),
                // Blitz focuses the button after the click and reports it at the
                // poll's end, which would dismiss a popover opened in it.
                None => when_laid_out(move || {
                    open.toggle();
                }),
            },
            Glyph { slot: IconSlot::Filter, icon: lucide::funnel::outlined }
        }
    }
}

/// A stable key per operator, for `NativeSelect`'s posted value.
fn operator_key(operator: FilterOperator) -> &'static str {
    use FilterOperator::*;
    match operator {
        Contains => "contains",
        DoesNotContain => "does-not-contain",
        Equals => "equals",
        NotEquals => "not-equals",
        StartsWith => "starts-with",
        EndsWith => "ends-with",
        GreaterThan => "greater-than",
        GreaterOrEqual => "greater-or-equal",
        LessThan => "less-than",
        LessOrEqual => "less-or-equal",
        IsEmpty => "is-empty",
        IsNotEmpty => "is-not-empty",
        Is => "is",
        Before => "before",
        After => "after",
        OnOrBefore => "on-or-before",
        OnOrAfter => "on-or-after",
        Between => "between",
    }
}
