use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    columns::KanbanColumnMove,
    drag::{
        BoardDrag, BoardDragOptions, Control, Landing, use_board_card, use_board_drag,
        use_board_list,
    },
    moves::KanbanMove,
    shown::{Shown, use_shown, use_shown_card},
};
use crate::{
    CssLayer,
    components::{
        accessibility::{Announcer, use_announcer},
        common::{Glyph, HtmlTag, Input, Orientation, Part, States, base_props, parts_enum},
        data_display::sortable::{
            SORTABLE_CONTENT_SX, SORTABLE_HANDLE_SX, SORTABLE_MOVE_SX, SortableMove,
            SortableOptions, item_name, sortable_item_sx, use_handle_name,
            use_labelled_sortable_item, use_sortable,
        },
        layout::use_box,
        overlay::{Menu, MenuItem, use_menu},
    },
    context::IconSlot,
    hooks::{current_localization, use_css, use_element, use_id, use_media_query},
    localization::fill,
    platform::{self, ElementApi},
    sx::{FORCED_COLORS, StaticSx, sx},
    theme::{PAPER_BACKGROUND, PAPER_BORDER_COLOR, PAPER_RADIUS, Size, SizeCss},
};

static KANBAN_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .align_items("flex-start")
        .gap("md")
        .overflow_x("auto")
        // A board with a height scrolls its cards (3069); the stable gutter keeps the
        // width when a card dragged below the columns adds the scrollbar mid-drag (2454).
        .overflow_y("auto")
        .with("scrollbar-gutter", "stable")
});

static KANBAN_COLUMN_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap("xs")
        .flex("1 0 220px")
        .min_width("0")
        .padding("sm")
        .border_radius("md")
        .position("relative")
        // Forced colours repaint the fill as Canvas; a transparent border becomes the edge.
        .border("1px solid transparent")
        .background("muted.1")
        .selector(
            KanbanColumnPart::Header.selector(),
            sx().display("flex")
                .align_items("center")
                .gap("xs")
                .font_weight("600")
                .padding("4px 0")
                .selector("& > [data-name]", sx().flex("1 1 auto").min_width("0")),
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
        .when(
            "sorting",
            sx().user_select("none")
                .transition("transform 150ms ease")
                .media("(prefers-reduced-motion: reduce)", sx().transition("none")),
        )
        // A column lifted by its reorder handle rises over the others, as a lifted card.
        .when(
            "dragging",
            sx().z_index("1")
                .transition("none")
                .box_shadow(SizeCss::SHADOW.value(Size::Md))
                .media(FORCED_COLORS, sx().outline("2px solid Highlight")),
        )
        .when(
            "target",
            sx().border_color("primary")
                .background("color-mix(in srgb, var(--lsx-primary-6) 8%, var(--lsx-muted-1))")
                .media("(forced-colors: active)", sx().border_color("Highlight")),
        )
});

/// The content's basis beside the handle and `buttons` 24px buttons: the rest of the line
/// while 8rem fit, else the whole line, so the buttons wrap below it together.
fn card_content_basis(buttons: u8) -> String {
    let slot = "(24px + var(--lsx-spacing-xs))";
    let line = format!("calc(100% - {slot})");
    let rest = format!("calc(100% - {} * {slot} - 1px)", buttons + 1);
    let room = format!("calc((8rem + {} * {slot} - 100%) * 999)", buttons + 1);
    format!("max({rest}, min({line}, {room}))")
}

/// The content's basis beside Move to alone, or with the move buttons too (no `:has()` under Blitz).
static CARD_CONTENT_ONE_SX: StaticSx =
    StaticSx::new(|| sx().selector("&[data-slot]", sx().flex_basis(card_content_basis(1))));
static CARD_CONTENT_THREE_SX: StaticSx =
    StaticSx::new(|| sx().selector("&[data-slot]", sx().flex_basis(card_content_basis(3))));

// Under 8rem for the content, as in a 220px column, the move buttons wrap below it, end-aligned.
// A paper card on the column's muted fill (todo 1437).
static KANBAN_CARD_SX: StaticSx = StaticSx::new(|| {
    sortable_item_sx()
        .flex_wrap("wrap")
        .padding("xs")
        .background(PAPER_BACKGROUND.value())
        .border(format!("1px solid {}", PAPER_BORDER_COLOR.value()))
        .border_radius(PAPER_RADIUS.value())
        .selector(
            "& > [data-slot='content'] + *",
            sx().margin_inline_start("auto"),
        )
});

/// What the columns and cards read from their [`Kanban`]. Owned here: a card
/// reads every column's, not only its own.
#[derive(Clone, Copy)]
struct Board {
    /// Each column's label by position, for the Move to menu.
    labels: Signal<Vec<Option<String>>>,
    onmove: Callback<KanbanMove>,
    announcer: Announcer,
    /// Where a card moved by the board lands: one of its controls takes the focus there.
    landing: CopyValue<Option<Landing>>,
    move_buttons: Signal<bool>,
    drag: BoardDrag,
    /// `oncolumnmove` is set: each column shows its reorder handle.
    reorder: Signal<bool>,
    /// The column handles' description.
    column_instructions: Signal<String>,
}

/// A card put in column `column`, called `name`, at slot `slot` of `count` shown cards there.
struct Put {
    column: usize,
    name: String,
    slot: usize,
    count: usize,
    to: usize,
    control: Control,
}

/// What a [`KanbanCard`] reads from its [`KanbanColumn`].
#[derive(Clone, Copy)]
struct ColumnView {
    index: Signal<usize>,
    instructions: Signal<String>,
    shown: Shown,
}

base_props! {
    pub struct KanbanProps {
        /// Called when a card changed place, in its column or to another. Apply it to your data.
        onmove: EventHandler<KanbanMove>,
        /// Called when a column changed place on the board. Set, each column header shows a
        /// reorder handle; apply the move to your column order and its cards.
        #[props(default)]
        oncolumnmove: Option<EventHandler<KanbanColumnMove>>,
        /// Each card's two buttons moving it one slot in its column without a drag (WCAG 2.5.7),
        /// and each column's two with `oncolumnmove`.
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
/// Set `oncolumnmove` and the columns reorder too, by a handle in each header (keyboard)
/// and its move buttons; keep the names in a signal and apply the [`KanbanColumnMove`] to both.
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
    let mut reorder = use_signal(|| props.oncolumnmove.is_some());
    if *reorder.peek() != props.oncolumnmove.is_some() {
        reorder.set(props.oncolumnmove.is_some());
    }
    let labels = use_signal(Vec::new);
    let landing = use_hook(|| CopyValue::new(None));
    let drag = use_board_drag(BoardDragOptions {
        onmove: Callback::new(move |step| onmove.call(step)),
        announcer,
        labels,
        landing,
    });
    // The columns are this list's items: each `KanbanColumn` joins it before making its own.
    let oncolumnmove = props.oncolumnmove;
    let columns = use_sortable(SortableOptions {
        orientation: Orientation::Horizontal,
        onreorder: use_callback(move |step: SortableMove| {
            if let Some(oncolumnmove) = oncolumnmove {
                oncolumnmove.call(KanbanColumnMove {
                    from: step.from,
                    to: step.to,
                });
            }
        }),
    });
    use_effect(move || columns.element.follow(Some(&drag.element)));
    let announcement = columns.announcement;
    use_effect(move || {
        let said = announcement();
        if !said.is_empty() {
            announcer.say(said);
        }
    });
    let column_instructions = use_id();
    let words = current_localization();
    let touch = use_media_query("(pointer: coarse)");
    let described = match touch() && props.move_buttons {
        true => words.kanban.column_touch_instructions,
        false => words.kanban.column_instructions,
    };
    use_context_provider(|| Board {
        labels,
        onmove: Callback::new(move |step| {
            drag.unsettle();
            onmove.call(step);
        }),
        announcer,
        landing,
        move_buttons,
        drag,
        reorder,
        column_instructions,
    });

    let board = use_box()
        .framework_sx(&KANBAN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .element(&drag.element)
        .event("onpointermove", drag.onpointermove)
        .event("onpointerup", drag.onpointerup)
        .event("onpointercancel", drag.onpointercancel)
        .render(HtmlTag::Div, props.attributes, props.children);

    rsx! {
        {board}
        if reorder() {
            div { id: "{column_instructions}", hidden: true, {described} }
        }
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
        /// The header's reorder handle, with `oncolumnmove`.
        Handle = "handle" => "& > [data-slot='header'] > [data-slot='handle']",
        /// The header's move back button, with `oncolumnmove` and `move_buttons`.
        MoveEarlier = "move-earlier" => "& > [data-slot='header'] > [data-slot='move-earlier']",
        /// The header's move forward button.
        MoveLater = "move-later" => "& > [data-slot='header'] > [data-slot='move-later']",
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

    // Before this column's own `use_sortable`: the board's list of columns is the nearest yet.
    let words = current_localization();
    let column_name = fill(words.kanban.column, &[("label", &props.label)]);
    let place = use_labelled_sortable_item(column, Some(column_name.clone()));
    let marker = use_id();
    let (reorder, move_buttons) = (board.reorder, board.move_buttons);
    use_effect(move || {
        // Again when the handle or the move buttons come or go.
        let _ = (reorder(), move_buttons());
        let parts = [
            KanbanColumnPart::Handle,
            KanbanColumnPart::MoveEarlier,
            KanbanColumnPart::MoveLater,
        ]
        .map(|part| {
            format!(
                " > [data-slot=\"header\"] > [data-slot=\"{}\"]",
                part.slot()
            )
        });
        let column = format!("[data-kanban-column=\"{}\"]", marker.peek());
        place.find_parts_by(&column, parts.each_ref().map(String::as_str));
    });

    let (onmove, mut landing, drag) = (board.onmove, board.landing, board.drag);
    let shown = use_shown();
    let mut held = use_hook(|| CopyValue::new(false));
    let list = use_sortable(SortableOptions {
        orientation: Orientation::Vertical,
        onreorder: use_callback(move |step: SortableMove| {
            // The sortable refocuses its own move; an earlier, refused landing is void.
            if landing.peek().is_some() {
                landing.set(None);
            }
            let column = *index.peek();
            // The sortable counts the shown cards; the move speaks the data's indices (2579).
            let (from, to) = shown.reorder(step.from, step.to);
            // The sortable has announced the move; it waits for the card to show there (3164).
            if let Some(card) = drag.card_at(column, from) {
                held.set(true);
                landing.set(Some(Landing {
                    column,
                    index: to,
                    control: Control::Keep,
                    card,
                    from: (column, from),
                    left: None,
                    held: false,
                }));
            }
            onmove.call(KanbanMove {
                from_column: column,
                from,
                to_column: column,
                to,
            });
        }),
    });
    // One live region for the board: each column's lift, move and drop go there.
    let announcer = board.announcer;
    let announcement = list.announcement;
    use_effect(move || {
        let said = announcement();
        match (said.is_empty(), held.replace(false)) {
            (true, _) => {}
            (false, true) => drag.announce_on_landing(said),
            (false, false) => announcer.say(said),
        }
    });

    let header_id = use_id();
    let instructions = use_id();
    use_context_provider(|| ColumnView {
        index,
        instructions,
        shown,
    });
    let touch = use_media_query("(pointer: coarse)");
    let room = use_board_list(board.drag, column, list.element);

    let sorting = (list.sorting)() || (board.drag.sorting)() || (place.sorting)();
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("sorting", sorting)
        .with("dragging", (place.dragging)())
        .with(
            "target",
            (board.drag.target)().is_some_and(|(at, _)| at == column),
        )
        .into();

    // A custom header may hold more than the name: `label` then names the list.
    let (name_attr, name) = match props.header {
        Some(_) => ("aria-label", props.label.clone()),
        None => ("aria-labelledby", header_id()),
    };
    // As `Sortable`'s: a touch screen reader's tap on the handle lifts nothing (2452).
    let described = match (touch(), (board.move_buttons)()) {
        (true, true) => words.kanban.touch_instructions,
        (true, false) => words.kanban.touch_move_to,
        (false, _) => words.kanban.instructions,
    };
    // The slot a card from another column opens grows the list, not past its edge.
    let room = room();
    let items = use_box()
        // Reverted at rest: an omitted padding would keep the last gap (3105).
        .style(Some(match room > 0.0 {
            true => format!("padding-bottom: {room}px"),
            false => "padding-bottom: revert-layer".to_string(),
        }))
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

    let move_class = use_css(Some(&SORTABLE_MOVE_SX), CssLayer::Framework);
    // Built every render, shown only with `oncolumnmove`: the builder's styles are hooks.
    let handle = use_box()
        .framework_sx(&SORTABLE_HANDLE_SX)
        .prepare()
        .attr("data-slot", KanbanColumnPart::Handle.slot())
        .attr("type", "button")
        .attr(
            "aria-label",
            fill(words.sortable.handle, &[("label", &column_name)]),
        )
        .attr("aria-describedby", (board.column_instructions)())
        .event("onkeydown", place.onkeydown)
        .event("onblur", place.onblur)
        .render(
            HtmlTag::Button,
            Vec::new(),
            rsx! { Glyph { slot: IconSlot::Grip, icon: lucide::grip_horizontal::outlined } },
        );
    let handle = reorder().then_some(handle);
    let named = |template: &str| fill(template, &[("label", &column_name)]);
    let moves = (reorder() && move_buttons()).then(|| {
        let (onearlier, onlater) = (place.onearlier, place.onlater);
        rsx! {
            button {
                class: move_class.clone(),
                r#type: "button",
                "data-slot": KanbanColumnPart::MoveEarlier.slot(),
                "aria-label": named(words.sortable.move_backward),
                disabled: (place.first)(),
                onclick: move |event| onearlier.call(event),
                Glyph { slot: IconSlot::ChevronLeft, icon: lucide::chevron_left::outlined }
            }
            button {
                class: move_class.clone(),
                r#type: "button",
                "data-slot": KanbanColumnPart::MoveLater.slot(),
                "aria-label": named(words.sortable.move_forward),
                disabled: (place.last)(),
                onclick: move |event| onlater.call(event),
                Glyph { slot: IconSlot::ChevronRight, icon: lucide::chevron_right::outlined }
            }
        }
    });

    use_box()
        .framework_sx(&KANBAN_COLUMN_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .style(Some(place.style()))
        .prepare()
        .attr("data-kanban-column", marker())
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div { "data-slot": KanbanColumnPart::Header.slot(),
                    {handle}
                    div { id: "{header_id}", "data-name": true,
                        match props.header {
                            Some(header) => header,
                            None => rsx! { "{props.label}" },
                        }
                    }
                    {moves}
                }
                {items}
                // Hidden, not visually hidden: read only as the handles' description.
                div { id: "{instructions}", hidden: true, {described} }
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
        /// The card's position in its column's data, from 0. Key it by its data, not this.
        /// A filtered column may skip indices: the drag, keys, buttons and Move to go by the cards it shows.
        index: usize,
        /// Names the card in its controls and the announcements. Unset, the handle reads the
        /// card's content, the rest "Item {n}" by its position.
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
    let index = props.index;
    let position = use_shown_card(column.shown, index);
    let item = use_labelled_sortable_item(position, props.label.clone());
    let words = current_localization();
    let name = item_name(words.sortable.item, props.label.as_deref(), position);
    let carried = use_board_card(
        board.drag,
        column.index,
        index,
        (item.element, item.handle),
        name.clone(),
        item.onkeydown,
    );
    let dragging = (item.dragging)() || (carried.dragging)();

    let trigger = use_element();
    let marker = use_id();
    let move_buttons = board.move_buttons;
    use_effect(move || {
        // Again when the move buttons come or go.
        let _ = move_buttons();
        let card = format!("[data-kanban-card=\"{}\"]", marker.peek());
        item.find_parts(&card);
        trigger.point_at(platform::mounted_by_selector(&format!(
            "{card} > * > [data-slot=\"{}\"]",
            KanbanCardPart::MoveTo.slot()
        )));
    });
    let mut landing = board.landing;
    let column_index = column.index;
    let handle_node = item.handle;
    let (drag, id, announcer) = (board.drag, carried.id, board.announcer);
    use_effect(use_reactive!(|index| {
        let _ = (trigger.mount_token(), handle_node.mount_token());
        let Some(at) = landing
            .peek()
            .filter(|at| (at.column, at.index) == (column_index(), index))
        else {
            return;
        };
        if !drag.landed(at, id) {
            landing.set(None);
            drag.take_pending();
            return;
        }
        let target = match at.control {
            Control::Handle => Some(handle_node),
            Control::MoveTo => Some(trigger),
            Control::Keep => None,
        };
        if target.is_none_or(|target| target.mounted().is_some()) {
            landing.set(None);
            if let Some(target) = target {
                let _ = target.focus();
            }
            if let Some(text) = drag.take_pending() {
                announcer.say(text);
            }
        } else if !at.held {
            landing.set(Some(Landing { held: true, ..at }));
        }
    }));

    let card_name = name.clone();
    let put = use_callback({
        let card = name.clone();
        move |put: Put| {
            let from_column = *column_index.peek();
            landing.set(Some(Landing {
                column: put.column,
                index: put.to,
                control: put.control,
                card: id,
                from: (from_column, index),
                left: None,
                held: false,
            }));
            drag.announce_on_landing(fill(
                words.kanban.moved,
                &[
                    ("label", &card),
                    ("column", &put.name),
                    ("n", &(put.slot + 1)),
                    ("m", &(put.count + 1)),
                ],
            ));
            board.onmove.call(KanbanMove {
                from_column,
                from: index,
                to_column: put.column,
                to: put.to,
            });
        }
    });

    // A lifted card's Left or Right puts it in the next column at its slot, as far as there is one.
    let keys = carried.onkeydown;
    let onkeydown = use_callback(move |event: Event<KeyboardData>| {
        let forward = match event.key() {
            Key::ArrowRight => true,
            Key::ArrowLeft => false,
            _ => return keys.call(event),
        };
        let Some(slot) = item.lifted_slot(false) else {
            return keys.call(event);
        };
        event.prevent_default();
        let next = match forward != drag.element.is_rtl() {
            true => column_index.peek().checked_add(1),
            false => column_index.peek().checked_sub(1),
        };
        let beside = next.and_then(|at| Some((at, board.labels.peek().get(at)?.clone()?)));
        let Some((to_column, name)) = beside else {
            announcer.say(fill(words.kanban.no_column, &[("label", &card_name)]));
            return;
        };
        item.lifted_slot(true);
        let count = drag.end(to_column).0;
        let slot = slot.min(count);
        let to = drag.slot_index(to_column, slot);
        put.call(Put {
            column: to_column,
            name,
            slot,
            count,
            to,
            control: Control::Handle,
        });
    });

    let named = |template: &str| fill(template, &[("label", &name)]);
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
        .map(|(to_column, column)| {
            let name = column.clone();
            MenuItem::new(column)
                .disabled(to_column == here)
                .onselect(move |()| {
                    // After the last card shown, numbered among the shown ones (2517).
                    let (count, to) = drag.end(to_column);
                    put.call(Put {
                        column: to_column,
                        name: name.clone(),
                        slot: count,
                        count,
                        to,
                        control: Control::MoveTo,
                    });
                })
                .into()
        })
        .collect::<Vec<_>>();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("dragging", dragging)
        .with("sorting", (item.sorting)() || (board.drag.sorting)())
        .into();
    // A keyboard move's offsets win over a dropped card's slide.
    let style = match (item.sorting)() {
        true => item.style(),
        false => carried.style().unwrap_or_else(|| item.style()),
    };

    let content_class = use_css(Some(&SORTABLE_CONTENT_SX), CssLayer::Framework);
    let basis_one = use_css(Some(&CARD_CONTENT_ONE_SX), CssLayer::Framework);
    let basis_three = use_css(Some(&CARD_CONTENT_THREE_SX), CssLayer::Framework);
    let basis_class = match (board.move_buttons)() {
        true => basis_three,
        false => basis_one,
    };
    let content_class = [content_class, basis_class]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let move_class = use_css(Some(&SORTABLE_MOVE_SX), CssLayer::Framework);
    let handle_name = use_handle_name(words.sortable.handle, props.label.as_deref());
    let handle = use_box()
        .framework_sx(&SORTABLE_HANDLE_SX)
        .prepare()
        .attr("data-slot", KanbanCardPart::Handle.slot())
        .attr("type", "button")
        .attr("aria-label", handle_name.aria_label)
        .attr("aria-labelledby", handle_name.aria_labelledby)
        .attr("aria-describedby", (column.instructions)())
        .event("onpointerdown", carried.onpointerdown)
        .event("onkeydown", onkeydown)
        .event("onblur", item.onblur)
        .render(
            HtmlTag::Button,
            Vec::new(),
            rsx! {
                Glyph { slot: IconSlot::Grip, icon: lucide::grip_vertical::outlined }
                {handle_name.words}
            },
        );

    let moves = (board.move_buttons)().then(|| {
        let (onearlier, onlater) = (item.onearlier, item.onlater);
        rsx! {
            button {
                class: move_class.clone(),
                r#type: "button",
                "data-slot": KanbanCardPart::MoveEarlier.slot(),
                "aria-label": named(words.sortable.move_up),
                disabled: (item.first)(),
                onclick: move |event| onearlier.call(event),
                Glyph { slot: IconSlot::ChevronUp, icon: lucide::chevron_up::outlined }
            }
            button {
                class: move_class.clone(),
                r#type: "button",
                "data-slot": KanbanCardPart::MoveLater.slot(),
                "aria-label": named(words.sortable.move_down),
                disabled: (item.last)(),
                onclick: move |event| onlater.call(event),
                Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined }
            }
        }
    });

    let move_to = use_box()
        .prepare()
        .attr("class", move_class.clone())
        .attr("data-slot", KanbanCardPart::MoveTo.slot())
        .attr("type", "button")
        .attr("aria-label", named(words.kanban.move_to))
        .render(
            HtmlTag::Button,
            menu.a11y_attributes(),
            rsx! { Glyph { slot: IconSlot::MoveTo, icon: lucide::arrow_right_left::outlined } },
        );

    use_box()
        .framework_sx(&KANBAN_CARD_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .style(Some(style))
        .prepare()
        .attr("data-kanban-card", marker())
        .render(
            HtmlTag::Li,
            props.attributes,
            rsx! {
                {handle}
                div {
                    id: "{handle_name.content_id}",
                    class: content_class,
                    "data-slot": KanbanCardPart::Content.slot(),
                    {props.children}
                }
                {moves}
                Menu { state: menu, items, {move_to} }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Stylesheet, components::common::part_table};

    #[test]
    fn a_narrow_card_wraps_its_buttons_below_the_content() {
        let css = Stylesheet::from(&KANBAN_CARD_SX).as_str().to_string();
        let basis = Stylesheet::from(&CARD_CONTENT_THREE_SX)
            .as_str()
            .to_string();

        assert!(css.contains("flex-wrap:wrap"), "{css}");
        assert!(basis.contains("flex-basis:max(calc(100% - 4 * "), "{basis}");
        assert!(css.contains("margin-inline-start:auto"), "{css}");
    }

    #[test]
    fn a_board_with_a_height_scrolls_both_ways_in_a_stable_gutter() {
        let css = Stylesheet::from(&KANBAN_SX).as_str().to_string();

        assert!(css.contains("overflow-x:auto"), "{css}");
        assert!(css.contains("overflow-y:auto"), "{css}");
        assert!(css.contains("scrollbar-gutter:stable"), "{css}");
    }

    #[test]
    fn the_column_a_dragged_card_would_land_in_is_marked() {
        let css = Stylesheet::from(&KANBAN_COLUMN_SX).as_str().to_string();
        let target = css
            .split("[data-state~=\"target\"]")
            .nth(1)
            .unwrap_or_default();

        assert!(target.contains("var(--lsx-primary-6) 8%"), "{css}");
        assert!(css.contains("Highlight"), "{css}");
    }

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_tables_are_stable() {
        assert_eq!(
            part_table::<KanbanColumnPart>(),
            [
                ("header", "& > [data-slot='header']"),
                ("list", "& > [data-slot='list']"),
                ("handle", "& > [data-slot='header'] > [data-slot='handle']"),
                (
                    "move-earlier",
                    "& > [data-slot='header'] > [data-slot='move-earlier']"
                ),
                (
                    "move-later",
                    "& > [data-slot='header'] > [data-slot='move-later']"
                ),
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
