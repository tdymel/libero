use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::moves::KanbanMove;
use crate::{
    CssLayer,
    components::{
        accessibility::{Announcer, use_announcer},
        common::{Glyph, HtmlTag, Input, Orientation, Part, States, base_props, parts_enum},
        data_display::sortable::{
            SORTABLE_CONTENT_SX, SORTABLE_HANDLE_SX, SORTABLE_ITEM_SX, SORTABLE_MOVE_SX,
            SortableMove, SortableOptions, use_labelled_sortable_item, use_sortable,
        },
        layout::use_box,
        overlay::{Menu, MenuItem, use_menu},
    },
    context::IconSlot,
    hooks::{current_localization, use_css, use_element, use_id},
    localization::fill,
    platform::ElementApi,
    sx::{StaticSx, sx},
};

static KANBAN_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("flex-start")
        .gap("md")
        .overflow_x("auto")
});

static KANBAN_COLUMN_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap("xs")
        .flex("1 0 220px")
        .min_width("0")
        .padding("sm")
        .border_radius("md")
        // Forced colours repaint the fill as Canvas; a transparent border becomes the edge.
        .border("1px solid transparent")
        .background("muted.1")
        .selector(
            KanbanColumnPart::Header.selector(),
            sx().font_weight("600").padding("4px 0"),
        )
        .selector(
            KanbanColumnPart::List.selector(),
            sx().display("flex")
                .flex_direction("column")
                .gap("xs")
                .list_style("none")
                .margin("0")
                .padding("0")
                // An empty column still shows where its cards go.
                .min_height("48px"),
        )
        .when("sorting", sx().user_select("none"))
});

/// What the columns and cards read from their [`Kanban`]. Owned here: a card
/// reads every column's, not only its own.
#[derive(Clone, Copy)]
struct Board {
    /// Each column's label by position, for the Move to menu.
    labels: Signal<Vec<Option<String>>>,
    /// Each column's card count by position, for where a move lands.
    counts: CopyValue<Vec<usize>>,
    onmove: Callback<KanbanMove>,
    announcer: Announcer,
    /// Where a card moved across lands: its Move to trigger takes the focus there.
    landing: CopyValue<Option<(usize, usize)>>,
    move_buttons: Signal<bool>,
}

/// What a [`KanbanCard`] reads from its [`KanbanColumn`].
#[derive(Clone, Copy)]
struct ColumnView {
    index: Signal<usize>,
    instructions: Signal<String>,
}

base_props! {
    pub struct KanbanProps {
        /// Called when a card changed place, in its column or to another. Apply it to your data.
        onmove: EventHandler<KanbanMove>,
        /// Each card's two buttons moving it one slot in its column without a drag (WCAG 2.5.7).
        #[props(default = true)]
        move_buttons: bool,
        /// [`KanbanColumn`]s, keyed by their data.
        children: Element,
    }
}

/// A board of columns whose cards move by drag, keyboard or a Move to menu.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Kanban, KanbanCard, KanbanColumn, KanbanMove};
/// # fn app() -> Element {
/// let names = ["To do", "Doing", "Done"];
/// let mut cards = use_signal(|| vec![vec!["Write"], vec!["Test"], vec![]]);
/// rsx! {
///     Kanban { onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
///         for (column, label) in names.into_iter().enumerate() {
///             KanbanColumn { key: "{label}", index: column, label,
///                 for (index, card) in cards()[column].clone().into_iter().enumerate() {
///                     KanbanCard { key: "{card}", index, label: card, "{card}" }
///                 }
///             }
///         }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/kanban>
#[component]
pub fn Kanban(props: KanbanProps) -> Element {
    let onmove = props.onmove;
    let announcer = use_announcer();
    let mut move_buttons = use_signal(|| props.move_buttons);
    if *move_buttons.peek() != props.move_buttons {
        move_buttons.set(props.move_buttons);
    }
    use_context_provider(|| Board {
        labels: Signal::new(Vec::new()),
        counts: CopyValue::new(Vec::new()),
        onmove: Callback::new(move |step| onmove.call(step)),
        announcer,
        landing: CopyValue::new(None),
        move_buttons,
    });

    let board = use_box()
        .framework_sx(&KANBAN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Div, props.attributes, props.children);

    rsx! {
        {board}
        {announcer.render()}
    }
}

parts_enum! {
    /// [`KanbanColumn`]'s inner parts, for its `parts` prop.
    pub enum KanbanColumnPart {
        /// The header naming the column.
        Header = "header" => "& > [data-slot='header']",
        /// The list holding the cards.
        List = "list" => "& > [data-slot='list']",
    }
}

base_props! {
    parts(KanbanColumnPart);
    pub struct KanbanColumnProps {
        /// The column's position on the board, from 0. Key it by its data, not this.
        index: usize,
        /// The column's name: the header's text, the list's name, a Move to entry.
        #[props(into)]
        label: String,
        /// The header's content instead of `label`, which stays the list's name.
        #[props(default)]
        header: Option<Element>,
        /// [`KanbanCard`]s, keyed by their data.
        children: Element,
    }
}

/// One column of a [`Kanban`]: a header, then its cards in a list.
///
/// Docs: <https://libero-ui.dev/data-display/kanban>
#[component]
pub fn KanbanColumn(props: KanbanColumnProps) -> Element {
    let board = use_context::<Board>();
    let column = props.index;
    let mut index = use_signal(|| column);
    if *index.peek() != column {
        index.set(column);
    }
    let mut labels = board.labels;
    let label = props.label.clone();
    use_effect(use_reactive!(|column, label| {
        let mut entries = labels.write();
        if entries.len() <= column {
            entries.resize(column + 1, None);
        }
        entries[column] = Some(label.clone());
    }));
    use_drop(move || {
        if let Ok(mut entries) = labels.try_write() {
            if let Some(entry) = entries.get_mut(*index.peek()) {
                *entry = None;
            }
            while entries.last().is_some_and(Option::is_none) {
                entries.pop();
            }
        }
    });

    let onmove = board.onmove;
    let list = use_sortable(SortableOptions {
        orientation: Orientation::Vertical,
        onreorder: use_callback(move |step: SortableMove| {
            let column = *index.peek();
            onmove.call(KanbanMove {
                from_column: column,
                from: step.from,
                to_column: column,
                to: step.to,
            });
        }),
    });
    // One live region for the board: each column's lift, move and drop go there.
    let announcer = board.announcer;
    let announcement = list.announcement;
    use_effect(move || {
        let said = announcement();
        if !said.is_empty() {
            announcer.say(said);
        }
    });

    let header_id = use_id();
    let instructions = use_id();
    use_context_provider(|| ColumnView {
        index,
        instructions,
    });
    let words = current_localization().sortable;

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("sorting", (list.sorting)())
        .into();

    // A custom header may hold more than the name: `label` then names the list.
    let (name_attr, name) = match props.header {
        Some(_) => ("aria-label", props.label.clone()),
        None => ("aria-labelledby", header_id()),
    };
    let items = use_box()
        .prepare()
        .element(&list.element)
        .attr("data-slot", KanbanColumnPart::List.slot())
        // Safari with VoiceOver drops a `list-style: none` list's role.
        .attr("role", "list")
        .attr(name_attr, name)
        .event("onpointermove", list.onpointermove)
        .event("onpointerup", list.onpointerup)
        .event("onpointercancel", list.onpointercancel)
        .render(HtmlTag::Ul, Vec::new(), props.children);

    use_box()
        .framework_sx(&KANBAN_COLUMN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .prepare()
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div { id: "{header_id}", "data-slot": KanbanColumnPart::Header.slot(),
                    match props.header {
                        Some(header) => header,
                        None => rsx! { "{props.label}" },
                    }
                }
                {items}
                // Hidden, not visually hidden: read only as the handles' description.
                div { id: "{instructions}", hidden: true, {words.instructions} }
            },
        )
}

parts_enum! {
    /// [`KanbanCard`]'s inner parts, for its `parts` prop. Each is a direct
    /// child, but `MoveTo` sits in the menu's anchor.
    pub enum KanbanCardPart {
        /// The drag handle.
        Handle = "handle" => "& > [data-slot='handle']",
        /// The wrapper round `children`.
        Content = "content" => "& > [data-slot='content']",
        /// The move up button.
        MoveEarlier = "move-earlier" => "& > [data-slot='move-earlier']",
        /// The move down button.
        MoveLater = "move-later" => "& > [data-slot='move-later']",
        /// The Move to column menu's trigger.
        MoveTo = "move-to" => "& > * > [data-slot='move-to']",
    }
}

base_props! {
    parts(KanbanCardPart);
    pub struct KanbanCardProps {
        /// The card's position in its column, from 0. Key it by its data, not this.
        index: usize,
        /// Names the card in the announcements. Unset, "Item {n}" by its position.
        #[props(default, into)]
        label: Option<String>,
        children: Element,
    }
}

/// One card of a [`KanbanColumn`]: a drag handle, `children`, the move buttons,
/// then a menu moving it to another column.
///
/// Docs: <https://libero-ui.dev/data-display/kanban>
#[component]
pub fn KanbanCard(props: KanbanCardProps) -> Element {
    let board = use_context::<Board>();
    let column = use_context::<ColumnView>();
    let item = use_labelled_sortable_item(props.index, props.label.clone());
    let words = current_localization();
    let index = props.index;
    let dragging = (item.dragging)();

    // Counted under the column the card mounted in; a board keeps its columns in place.
    let mut counts = board.counts;
    let counted = use_hook(|| {
        let at = *column.index.peek();
        let mut counts = counts.write();
        if counts.len() <= at {
            counts.resize(at + 1, 0);
        }
        counts[at] += 1;
        at
    });
    use_drop(move || {
        if let Ok(mut counts) = counts.try_write()
            && let Some(count) = counts.get_mut(counted)
        {
            *count = count.saturating_sub(1);
        }
    });

    let trigger = use_element();
    let mut landing = board.landing;
    let column_index = column.index;
    use_effect(use_reactive!(|index| {
        let _ = trigger.mount_token();
        if *landing.peek() == Some((column_index(), index)) && trigger.mounted().is_some() {
            landing.set(None);
            let _ = trigger.focus();
        }
    }));

    let label = props.label.clone();
    let menu = use_menu();
    let here = column_index();
    let names: Vec<(usize, String)> = board
        .labels
        .read()
        .iter()
        .enumerate()
        .filter_map(|(at, name)| Some((at, name.clone()?)))
        .collect();
    let items = names
        .into_iter()
        .map(|(to_column, name)| {
            let label = label.clone();
            MenuItem::new(name.clone())
                .disabled(to_column == here)
                .onselect(move |()| {
                    let to = counts.peek().get(to_column).copied().unwrap_or(0);
                    let from_column = *column_index.peek();
                    let card = label
                        .clone()
                        .unwrap_or_else(|| fill(words.sortable.item, &[("n", &(index + 1))]));
                    landing.set(Some((to_column, to)));
                    board.announcer.say(fill(
                        words.kanban.moved,
                        &[
                            ("label", &card),
                            ("column", &name),
                            ("n", &(to + 1)),
                            ("m", &(to + 1)),
                        ],
                    ));
                    board.onmove.call(KanbanMove {
                        from_column,
                        from: index,
                        to_column,
                        to,
                    });
                })
                .into()
        })
        .collect::<Vec<_>>();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("dragging", dragging)
        .with("sorting", (item.sorting)())
        .into();

    let content_class = use_css(Some(&SORTABLE_CONTENT_SX), CssLayer::Framework);
    let move_class = use_css(Some(&SORTABLE_MOVE_SX), CssLayer::Framework);
    let handle = use_box()
        .framework_sx(&SORTABLE_HANDLE_SX)
        .prepare()
        .element(&item.handle)
        .attr("data-slot", KanbanCardPart::Handle.slot())
        .attr("type", "button")
        .attr("aria-label", words.sortable.handle)
        .attr("aria-describedby", (column.instructions)())
        .event("onpointerdown", item.onpointerdown)
        .event("onkeydown", item.onkeydown)
        .event("onblur", item.onblur)
        .render(
            HtmlTag::Button,
            Vec::new(),
            rsx! { Glyph { slot: IconSlot::Grip, icon: lucide::grip_vertical::outlined } },
        );

    let moves = (board.move_buttons)().then(|| {
        let (onearlier, onlater) = (item.onearlier, item.onlater);
        rsx! {
            button {
                class: move_class.clone(),
                r#type: "button",
                "data-slot": KanbanCardPart::MoveEarlier.slot(),
                "aria-label": words.sortable.move_up,
                disabled: (item.first)(),
                onmounted: item.earlier.mount(),
                onclick: move |event| onearlier.call(event),
                Glyph { slot: IconSlot::ChevronUp, icon: lucide::chevron_up::outlined }
            }
            button {
                class: move_class.clone(),
                r#type: "button",
                "data-slot": KanbanCardPart::MoveLater.slot(),
                "aria-label": words.sortable.move_down,
                disabled: (item.last)(),
                onmounted: item.later.mount(),
                onclick: move |event| onlater.call(event),
                Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
            }
        }
    });

    let move_to = use_box()
        .prepare()
        .element(&trigger)
        .attr("class", move_class.clone())
        .attr("data-slot", KanbanCardPart::MoveTo.slot())
        .attr("type", "button")
        .attr("aria-label", words.kanban.move_to)
        .render(
            HtmlTag::Button,
            menu.a11y_attributes(),
            rsx! { Glyph { slot: IconSlot::MoveTo, icon: lucide::arrow_right_left::outlined } },
        );

    use_box()
        .framework_sx(&SORTABLE_ITEM_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .style(Some(item.style()))
        .prepare()
        .element(&item.element)
        .render(
            HtmlTag::Li,
            props.attributes,
            rsx! {
                {handle}
                div { class: content_class, "data-slot": KanbanCardPart::Content.slot(), {props.children} }
                {moves}
                Menu { state: menu, items, {move_to} }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_tables_are_stable() {
        assert_eq!(
            part_table::<KanbanColumnPart>(),
            [
                ("header", "& > [data-slot='header']"),
                ("list", "& > [data-slot='list']"),
            ]
        );
        assert_eq!(
            part_table::<KanbanCardPart>(),
            [
                ("handle", "& > [data-slot='handle']"),
                ("content", "& > [data-slot='content']"),
                ("move-earlier", "& > [data-slot='move-earlier']"),
                ("move-later", "& > [data-slot='move-later']"),
                ("move-to", "& > * > [data-slot='move-to']"),
            ]
        );
    }
}
