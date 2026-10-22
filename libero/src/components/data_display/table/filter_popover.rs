#[cfg(feature = "date-time")]
use crate::{components::form::DateField, platform::reads_dom_synchronously};
#[cfg(feature = "date-time")]
use chrono::NaiveDate;
use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    cell_value::FilterKind,
    column_filter::{
        ColumnFilter, FilterOperator, SelectOption, filter_of, with_filter, without_column,
    },
    use_table::StateSlice,
};
use crate::{
    components::{
        buttons::Button,
        common::{FOCUSABLE_SELECTOR, Glyph, HtmlTag, Options, attr},
        form::{Checkbox, MultiSelect, NativeSelect, SelectionArgs, TextField},
        layout::{paper_sx, use_box},
    },
    context::IconSlot,
    hooks::{
        Align, ElementHandle, POPOVER_AVAILABLE_HEIGHT, PopoverOptions, PressMarker, Side,
        id_selector, placement_sx, use_debounced_callback, use_id, use_marked_popover, use_theme,
    },
    localization::TableLabels,
    platform::{
        ElementApi, PlatformError, focus_first_of, focus_lands_in, focus_tab_from, is_tab,
        page_task, when_laid_out,
    },
    sx::{StaticSx, sx},
    theme::{Size, SizeCss, Z_INDEX_POPOVER},
};

pub(super) static FILTER_POPOVER_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .and(placement_sx(true))
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Md))
        .display("flex")
        .flex_direction("column")
        .gap(SizeCss::SPACING.value(Size::Sm))
        // Beats the placement's `auto`, which the inline style used to win with (3913).
        .min_width("14rem")
        // Never past the room on its side: it scrolls instead (WCAG 1.4.10, todo 1964).
        .max_height(POPOVER_AVAILABLE_HEIGHT.value_or("none"))
        .overflow_y("auto")
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

impl Options for SelectOption {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn value(&self) -> String {
        self.value.clone()
    }
}

/// A select column's option in a filter's `NativeSelect`; `None` is any.
#[derive(Clone)]
struct OptionPick {
    option: Option<String>,
    label: String,
}

impl PartialEq for OptionPick {
    fn eq(&self, other: &Self) -> bool {
        self.option == other.option
    }
}

impl Options for OptionPick {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn value(&self) -> String {
        self.option.clone().unwrap_or_default()
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

/// Lets `edit` change `column`'s filter, keeping the rest. Emptied with the default
/// operator, the item goes: it holds nothing worth keeping.
fn commit(
    slice: StateSlice<Vec<ColumnFilter>>,
    column: &str,
    kind: FilterKind,
    edit: impl FnOnce(&mut ColumnFilter),
) {
    let filters = slice.peek();
    let operator = operator_of(&filters, column, kind);
    let mut next = filter_of(&filters, column)
        .cloned()
        .unwrap_or_else(|| ColumnFilter::new(column, operator, ""));
    edit(&mut next);
    let keeps = !next.value.is_empty()
        || !next.value_to.is_empty()
        || !next.values.is_empty()
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
    /// A select column's options.
    pub options: Vec<SelectOption>,
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
    /// An operator pick, applied at once with the value and draft kept.
    pub onoperator: Callback<FilterOperator>,
    /// A boolean pick, applied at once; `None` is any.
    pub onpick: Callback<Option<bool>>,
    /// A date field's change, applied at once: `true` for `Between`'s last day.
    pub onday: Callback<(bool, String)>,
    /// A select column's single option, applied at once; `None` is any.
    pub onoption: Callback<Option<String>>,
    /// A select column's `IsAnyOf` options, applied at once.
    pub onany: Callback<Vec<String>>,
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
                commit(slice, &column, kind, |filter| filter.value = text);
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
            commit(slice, &column, kind, |filter| match to {
                true => filter.value_to = text,
                false => filter.value = text,
            });
            onfilter.call(());
        })
    };
    let onoption = {
        let column = column.clone();
        use_callback(move |option: Option<String>| {
            commit(slice, &column, kind, |filter| {
                filter.value = option.unwrap_or_default();
            });
            onfilter.call(());
        })
    };
    let onany = {
        let column = column.clone();
        use_callback(move |values: Vec<String>| {
            commit(slice, &column, kind, |filter| filter.values = values);
            onfilter.call(());
        })
    };
    let onoperator = {
        let column = column.clone();
        use_callback(move |operator: FilterOperator| {
            let filters = slice.peek();
            let held = filter_of(&filters, &column);
            // Typed text not applied yet moves to the new operator.
            let value = draft
                .peek()
                .clone()
                .or_else(|| held.map(|filter| filter.value.clone()))
                .unwrap_or_default();
            draft.set(None);
            let next = ColumnFilter {
                value_to: held.map(|f| f.value_to.clone()).unwrap_or_default(),
                values: held.map(|f| f.values.clone()).unwrap_or_default(),
                ..ColumnFilter::new(column.clone(), operator, value)
            };
            slice.set(with_filter(&filters, &column, Some(next)));
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
        onoperator,
        onpick,
        onday,
        onoption,
        onany,
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
    /// A select column's options.
    #[props(default)]
    options: Vec<SelectOption>,
    /// A select column's single option's value; `None` is any.
    onoption: EventHandler<Option<String>>,
    /// A select column's `IsAnyOf` options.
    onany: EventHandler<Vec<String>>,
    /// Shown above the field; else it is named `name`, or by `filter_column`.
    #[props(default)]
    label: Option<String>,
    #[props(default)] name: Option<String>,
    /// A one-line `IsAnyOf` pick for the header filters row, in place of the checkbox column.
    #[props(default)]
    compact: bool,
) -> Element {
    let current = filter.as_ref();
    let operator = current.map_or(FilterOperator::of(kind)[0], |filter| filter.operator);
    let name = name.unwrap_or_else(|| (labels.filter_column)(&column));
    let aria_label = label.is_none().then(|| name.clone());
    if kind == FilterKind::Select {
        if !operator.takes_value() {
            return rsx! {};
        }
        if operator == FilterOperator::IsAnyOf {
            let picked = current
                .map(|filter| filter.values.clone())
                .unwrap_or_default();
            if compact {
                return rsx! {
                    CompactAnyOf {
                        name,
                        size,
                        any: labels.any,
                        options,
                        picked,
                        onany,
                    }
                };
            }
            return rsx! {
                div {
                    role: "group",
                    aria_label: name,
                    style: "display: flex; flex-direction: column; gap: 0.25rem;",
                    for option in options.iter().cloned() {
                        Checkbox {
                            key: "{option.value}",
                            label: option.label.clone(),
                            size,
                            checked: picked.contains(&option.value),
                            onchange: {
                                let (options, picked) = (options.clone(), picked.clone());
                                move |on: bool| {
                                    let next = options
                                        .iter()
                                        .filter(|each| match **each == option {
                                            true => on,
                                            false => picked.contains(&each.value),
                                        })
                                        .map(|each| each.value.clone())
                                        .collect();
                                    onany.call(next);
                                }
                            },
                        }
                    }
                }
            };
        }
        let mut choices = vec![OptionPick {
            option: None,
            label: labels.any.to_string(),
        }];
        choices.extend(options.iter().map(|option| OptionPick {
            option: Some(option.value.clone()),
            label: option.label.clone(),
        }));
        let value = current.map(|filter| &filter.value);
        let picked = choices
            .iter()
            .find(|choice| choice.option.as_ref() == value.filter(|value| !value.is_empty()))
            .cloned();
        return rsx! {
            NativeSelect {
                label,
                aria_label,
                size,
                value: picked,
                options: choices,
                onchange: move |choice: OptionPick| onoption.call(choice.option),
            }
        };
    }
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
            #[cfg(feature = "date-time")]
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
            // A WebView cannot count the portaled calendar inside the popover, and `date-time` may be
            // off: the native date input then.
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

/// `IsAnyOf` as one line: a `MultiSelect` whose picks collapse to the first label and a count.
#[component]
fn CompactAnyOf(
    name: String,
    size: Size,
    any: &'static str,
    options: Vec<SelectOption>,
    picked: Vec<String>,
    onany: EventHandler<Vec<String>>,
) -> Element {
    let held: Vec<SelectOption> = options
        .iter()
        .filter(|option| picked.contains(&option.value))
        .cloned()
        .collect();
    let summary = match held.split_first() {
        Some((first, [])) => Some((first.clone(), first.label.clone())),
        Some((first, rest)) => Some((first.clone(), format!("{} +{}", first.label, rest.len()))),
        None => None,
    };
    let selection = Callback::new(move |args: SelectionArgs<SelectOption>| match &summary {
        Some((first, text)) if args.value == *first => {
            rsx! {
                span {
                    "data-filter-summary": true,
                    style: "white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                    "{text}"
                }
            }
        }
        _ => rsx! {},
    });
    let ordered = options.clone();
    rsx! {
        MultiSelect::<SelectOption> {
            aria_label: name,
            size,
            sx: sx().min_width("9rem"),
            placeholder: any,
            options: options.clone(),
            value: held,
            selection,
            onchange: move |next: Vec<SelectOption>| {
                let values = ordered
                    .iter()
                    .filter(|option| next.contains(option))
                    .map(|option| option.value.clone())
                    .collect();
                onany.call(values);
            },
        }
    }
}

/// A column's filter popover, anchored to its menu button, and the button a
/// filtered header shows to reopen it.
#[component]
pub(super) fn FilterPopover(
    target: FilterTarget,
    anchor: ElementHandle,
    /// The button that opened it, which takes the focus back on close.
    opener: Signal<ElementHandle>,
    open: Signal<bool>,
    /// Spread on the menu button while open: focus moved there stays inside (3591).
    marker: PressMarker,
) -> Element {
    let theme = use_theme();
    let mut open = open;
    let box_id = use_id();
    let editor = use_filter_editor(&target);
    let mut draft = editor.draft;
    let popover = use_marked_popover(
        anchor,
        open(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::End)
            .dismiss(true),
        Some(marker),
    );
    let FilterTarget {
        column,
        kind,
        options,
        slice,
        labels,
        size,
        onfilter,
    } = target;
    // Typing still settling applies after the close: the editor lives here.
    let held = column.clone();
    // Clearing the filter removes its indicator: the menu button then.
    let back = use_memo(move || match filter_of(&slice.read(), &held) {
        Some(_) => *opener.read(),
        None => anchor,
    });
    // After the popover's own remember of the anchor, which placing outlasts.
    use_effect(move || {
        if open() && popover.placed() {
            popover.return_focus_to(back());
        }
    });
    let close = use_callback(move |refocus: bool| {
        open.set(false);
        if refocus {
            let _ = back.peek().focus();
        }
    });
    // The shared focus return decides: a focus moved elsewhere stays (3551).
    popover.on_dismiss(move || close.call(false));
    let floating = *popover.floating();
    // The value field takes focus once placed, else the operator picker.
    let mut entered = use_signal(|| false);
    use_effect(move || match (open(), popover.placed()) {
        (true, true) if !*entered.peek() => {
            entered.set(true);
            const VALUE: &str = "input[data-filter-value]";
            const PICKER: &str = ":is(select, [role=combobox])";
            let target = floating
                .query_selector(VALUE)
                .or_else(|_| floating.query_selector(PICKER));
            match target {
                Ok(target) => {
                    let _ = target.focus();
                }
                // A WebView queries nothing: the page focuses the first in the box (958).
                Err(PlatformError::Unsupported) => {
                    let at = id_selector(&box_id.peek());
                    let _ =
                        focus_first_of(&[format!("{at} {VALUE}"), format!("{at} {PICKER}"), at]);
                }
                Err(_) => {
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
    // A WebView's Tab past the end: on from the menu button, once the close has handed it focus.
    let mut tab_on = use_signal(|| false);
    use_effect(move || {
        if !open() && tab_on() {
            tab_on.set(false);
            spawn(async {
                page_task().await;
                let _ = focus_tab_from("", false);
            });
        }
    });
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
        attributes.push(attr("style", popover.vars()));
        attributes.extend(popover.floating_events());
        panel
            .element(&floating)
            .event("onkeydown", move |event: KeyboardEvent| {
                if !is_tab(&event) {
                    return;
                }
                // Tab past either end leaves through the menu button, as from it.
                let backwards = event.modifiers().shift();
                let items = match floating.query_selector_all(FOCUSABLE_SELECTOR) {
                    Ok(items) => items,
                    // A WebView queries nothing and has moved focus already: close if it left the box (1403).
                    Err(PlatformError::Unsupported) => {
                        let at = id_selector(&box_id.peek());
                        spawn(async move {
                            page_task().await;
                            if matches!(focus_lands_in(&at).await, Ok(false)) {
                                close.call(true);
                                tab_on.set(!backwards);
                            }
                        });
                        return;
                    }
                    Err(_) => return,
                };
                let at = items.iter().position(|item| item.is_focused());
                let leaves = match (at, backwards) {
                    (None, _) => true,
                    (Some(at), false) => at + 1 >= items.len(),
                    (Some(at), true) => at == 0,
                };
                if leaves {
                    close.call(true);
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
                                editor.onoperator.call(choice.value);
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
                        options: options.clone(),
                        onoption: editor.onoption,
                        onany: editor.onany,
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
    /// This button, and the popover's focus target while it opens from here.
    handle: ElementHandle,
    opener: Signal<ElementHandle>,
    /// Set with `filter_panel`: opens the panel instead.
    #[props(default)]
    onopen: Option<EventHandler<()>>,
    /// In a grid the column menu's Filter is the keyboard route: out of the Tab order.
    grid: bool,
) -> Element {
    let (mut open, mut opener) = (open, opener);
    rsx! {
        button {
            r#type: "button",
            "data-filtered": true,
            tabindex: grid.then_some("-1"),
            aria_label: (labels.filtered)(&column),
            aria_haspopup: "dialog",
            aria_expanded: "{open()}",
            onmounted: handle.mount(),
            onclick: move |_| match onopen {
                Some(onopen) => onopen.call(()),
                // Blitz focuses the button after the click and reports it at the
                // poll's end, which would dismiss a popover opened in it.
                None => {
                    opener.set(handle);
                    when_laid_out(move || {
                        open.toggle();
                    })
                }
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
        IsNot => "is-not",
        IsAnyOf => "is-any-of",
        Before => "before",
        After => "after",
        OnOrBefore => "on-or-before",
        OnOrAfter => "on-or-after",
        Between => "between",
    }
}
