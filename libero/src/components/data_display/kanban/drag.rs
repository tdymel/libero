//! A card's pointer drag across a [`Kanban`](super::Kanban): one drag for the
//! whole board, so a card can leave its column. The keyboard and the move
//! buttons stay on each column's sortable.

use std::{collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;

use super::{
    lanes::{Lane, Lanes, Rect, edge_step},
    moves::KanbanMove,
};
use crate::{
    components::accessibility::Announcer,
    hooks::{
        DragMove, DragOptions, DragStart, ElementHandle, current_localization, use_distance_drag,
        use_element, use_interval,
    },
    localization::fill,
    platform::{self, Dimensions, ElementApi, Read},
    theme::{SORTABLE_SETTLE, SORTABLE_SETTLE_FROM, TRANSITION_DURATION, TRANSITION_EASING},
};

type Mounted = Option<Rc<MountedData>>;

/// The auto-scroll tick near a side edge. Natively each costs a thread, only while there.
const AUTO_SCROLL_MS: u64 = 40;

/// A card's place and nodes. Not its `ElementHandle`s: the board reading those is a cross-scope read.
#[derive(Clone)]
struct Placed {
    column: usize,
    index: usize,
    element: Mounted,
    handle: Mounted,
    label: String,
}

/// A pointer drag measured and under way.
#[derive(Clone, PartialEq)]
struct Lifted {
    lanes: Lanes,
    label: String,
    /// The board's rect when it scrolls sideways, else `None`.
    board: Option<Rect>,
    /// The board's scroll offset at the lift.
    scroll: (f64, f64),
    /// The least and most `scrollLeft`, measured before the lifted card's
    /// transform widened the board's overflow.
    scroll_range: (f64, f64),
}

/// The card just dropped at `column`, `index`: it slides in from `offset` px (1129).
#[derive(Clone, Copy, PartialEq)]
struct Settle {
    column: usize,
    index: usize,
    offset: (f64, f64),
}

/// Which of a card's controls takes the focus where it lands in another column.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Control {
    Handle,
    MoveTo,
}

/// Where a card moved by the board lands, for its new self to take the focus.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Landing {
    pub(super) column: usize,
    pub(super) index: usize,
    pub(super) control: Control,
}

/// What [`use_board_drag`] takes from its board.
pub(super) struct BoardDragOptions {
    pub(super) onmove: Callback<KanbanMove>,
    pub(super) announcer: Announcer,
    pub(super) labels: Signal<Vec<Option<String>>>,
    pub(super) landing: CopyValue<Option<Landing>>,
}

/// The board's side of the drag: the board root takes the capture and the move, up and cancel handlers.
#[derive(Clone, Copy)]
pub(super) struct BoardDrag {
    pub(super) element: ElementHandle,
    /// A card is being dragged by pointer.
    pub(super) sorting: Memo<bool>,
    /// The column and slot the dragged card would land in.
    pub(super) target: Memo<Option<(usize, usize)>>,
    pub(super) onpointermove: Callback<Event<PointerData>>,
    pub(super) onpointerup: Callback<Event<PointerData>>,
    pub(super) onpointercancel: Callback<Event<PointerData>>,
    cards: CopyValue<BTreeMap<usize, Placed>>,
    lists: CopyValue<Vec<Mounted>>,
    next_id: CopyValue<usize>,
    pressed: CopyValue<Option<usize>>,
    lifted: Signal<Option<Lifted>>,
    travel: Signal<(f64, f64)>,
    /// Px the board scrolled sideways since the lift: the cards moved the other way.
    scrolled: Signal<f64>,
    settle: Signal<Option<Settle>>,
    onpointerdown: Callback<Event<PointerData>>,
    cancel: Callback<()>,
}

impl BoardDrag {
    /// Ends a dropped card's slide: another move is under way.
    pub(super) fn unsettle(mut self) {
        if self.settle.peek().is_some() {
            self.settle.set(None);
        }
    }
}

type Reading = (Read<(f64, f64)>, Read<Dimensions>);

fn read(node: &Rc<MountedData>) -> Reading {
    let node = platform::element(node);
    (node.client_offset(), node.dimensions())
}

async fn rect((offset, size): Reading) -> Option<Rect> {
    let ((x, y), size) = (offset.await.ok()?, size.await.ok()?);
    Some(Rect {
        x,
        y,
        width: size.width,
        height: size.height,
    })
}

/// `template` with the card's `label` and position `index` of `count`.
fn say(template: &str, label: &str, index: usize, count: usize) -> String {
    fill(
        template,
        &[("label", &label), ("n", &(index + 1)), ("m", &count)],
    )
}

pub(super) fn use_board_drag(options: BoardDragOptions) -> BoardDrag {
    let BoardDragOptions {
        onmove,
        announcer,
        labels,
        mut landing,
    } = options;
    let element = use_element();
    let words = current_localization().sortable;
    let moved = current_localization().kanban.moved;

    let cards = use_hook(|| CopyValue::new(BTreeMap::<usize, Placed>::new()));
    let lists = use_hook(|| CopyValue::new(Vec::<Mounted>::new()));
    let next_id = use_hook(|| CopyValue::new(0_usize));
    let mut pressed = use_hook(|| CopyValue::new(None::<usize>));
    let mut lifted = use_signal(|| None::<Lifted>);
    let mut travel = use_signal(|| (0.0_f64, 0.0_f64));
    let mut scrolled = use_signal(|| 0.0_f64);
    let mut settle = use_signal(|| None::<Settle>);
    // A release can land before the measure: (measuring, released).
    let mut starting = use_hook(|| CopyValue::new((false, false)));

    let target = use_memo(move || {
        let (dx, dy) = travel();
        lifted.read().as_ref()?.lanes.target(dx + scrolled(), dy)
    });

    // Near a side edge the board scrolls by `edge` px per tick, its cards' geometry with it.
    let mut edge = use_hook(|| CopyValue::new(0.0_f64));
    let auto_scroll = use_interval(
        move || {
            let step = *edge.peek();
            let Some((from, (least, most))) = lifted
                .peek()
                .as_ref()
                .map(|up| (up.scroll, up.scroll_range))
            else {
                return;
            };
            let now = from.0 + *scrolled.peek();
            let next = (now + step).clamp(least, most);
            if next == now {
                return;
            }
            let _ = element.scroll_to(next, from.1);
            let at = element.scroll_offset();
            spawn(async move {
                if let Ok((x, _)) = at.await
                    && lifted.peek().is_some()
                {
                    scrolled.set(x - from.0);
                }
            });
        },
        AUTO_SCROLL_MS,
    );
    let sorting = use_memo(move || lifted.read().is_some());

    let mut lift = move |up: Lifted| {
        let count = up.lanes.lanes[up.lanes.from_column].cards.len();
        announcer.say(say(words.lifted, &up.label, up.lanes.from, count));
        lifted.set(Some(up));
    };

    // Measures every column and card, then calls `started`; `None` when it can't.
    let mut begin = move |id: usize, started: Callback<Option<Lifted>>| {
        let placed = cards.peek().clone();
        let Some(card) = placed.get(&id).cloned() else {
            started.call(None);
            return;
        };
        let list_reads: Option<Vec<Reading>> = lists
            .peek()
            .iter()
            .map(|list| list.as_ref().map(read))
            .collect();
        let mut columns = vec![Vec::new(); lists.peek().len()];
        for placed in placed.values() {
            if let Some(column) = columns.get_mut(placed.column) {
                column.push((placed.index, placed.element.clone()));
            }
        }
        // Each column's cards in order, with no slot missing.
        let card_reads: Option<Vec<Vec<Reading>>> = columns
            .into_iter()
            .map(|mut column| {
                column.sort_by_key(|(index, _)| *index);
                column
                    .iter()
                    .enumerate()
                    .map(|(at, (index, node))| node.as_ref().filter(|_| at == *index).map(read))
                    .collect()
            })
            .collect();
        let (Some(list_reads), Some(card_reads)) = (list_reads, card_reads) else {
            started.call(None);
            return;
        };
        settle.set(None);
        let rtl = element.is_rtl();
        let board_reads = (
            (element.client_offset(), element.dimensions()),
            element.scroll_size(),
            element.scroll_offset(),
        );
        spawn(async move {
            let (board, content, scroll) = board_reads;
            let board = rect(board).await;
            let content = content.await.map_or(0.0, |size| size.width);
            let scroll = scroll.await.unwrap_or_default();
            // A board that fits has nothing to scroll: no ticks.
            let board = board.filter(|board| content > board.width + 0.5);
            let most = board.map_or(0.0, |board| content - board.width);
            // Right to left, `scrollLeft` runs from 0 down to minus the overflow.
            let scroll_range = match rtl {
                true => (-most, 0.0),
                false => (0.0, most),
            };
            let mut lanes = Vec::with_capacity(list_reads.len());
            for (list, column) in list_reads.into_iter().zip(card_reads) {
                let mut cards = Vec::with_capacity(column.len());
                for card in column {
                    cards.push(rect(card).await);
                }
                let (Some(list), Some(cards)) = (rect(list).await, cards.into_iter().collect())
                else {
                    started.call(None);
                    return;
                };
                lanes.push(Lane { list, cards });
            }
            let fits = lanes
                .get(card.column)
                .is_some_and(|lane| card.index < lane.cards.len());
            if !fits {
                started.call(None);
                return;
            }
            // `use_drag` refocuses the handle only on the web.
            if let Some(handle) = &card.handle {
                let _ = platform::element(handle).focus();
            }
            started.call(Some(Lifted {
                lanes: Lanes {
                    lanes,
                    from_column: card.column,
                    from: card.index,
                },
                label: card.label,
                board,
                scroll,
                scroll_range,
            }));
        });
    };

    // Ends the drag where the card hovers; `commit: false` puts it back.
    let mut finish = move |commit: bool| {
        let Some(ended) = lifted.peek().clone() else {
            return;
        };
        let (dx, dy) = *travel.peek();
        let dx = dx + *scrolled.peek();
        let lanes = &ended.lanes;
        let from = (lanes.from_column, lanes.from);
        // Let go off the board, it goes back.
        let landed = commit.then(|| lanes.target(dx, dy)).flatten();
        let to = landed.unwrap_or(from);
        edge.set(0.0);
        auto_scroll.stop();
        lifted.set(None);
        travel.set((0.0, 0.0));
        scrolled.set(0.0);
        pressed.set(None);
        let count = lanes.lanes[to.0].cards.len();
        let label = &ended.label;
        announcer.say(match (landed.is_some(), to.0 == from.0) {
            (false, _) => say(words.cancelled, label, from.1, count),
            (true, true) => say(words.dropped, label, to.1, count),
            (true, false) => {
                let column = labels.peek().get(to.0).cloned().flatten();
                fill(
                    moved,
                    &[
                        ("label", label),
                        ("column", &column.unwrap_or_default()),
                        ("n", &(to.1 + 1)),
                        ("m", &(count + 1)),
                    ],
                )
            }
        });
        if to != from {
            landing.set(Some(Landing {
                column: to.0,
                index: to.1,
                control: Control::Handle,
            }));
            onmove.call(KanbanMove {
                from_column: from.0,
                from: from.1,
                to_column: to.0,
                to: to.1,
            });
        }
        let (x, y) = lanes.landing(to);
        let offset = (dx - x, dy - y);
        if offset.0.abs() > 0.5 || offset.1.abs() > 0.5 {
            settle.set(Some(Settle {
                column: to.0,
                index: to.1,
                offset,
            }));
        }
    };

    let drag = use_distance_drag(DragOptions {
        capture: element,
        onstart: use_callback(move |start: DragStart| {
            let cancel = start.cancel;
            let (Some(id), false) = (*pressed.peek(), lifted.peek().is_some()) else {
                cancel.call(());
                return;
            };
            starting.set((true, false));
            travel.set((0.0, 0.0));
            begin(
                id,
                Callback::new(move |measured: Option<Lifted>| {
                    let (_, released) = starting.replace((false, false));
                    match measured {
                        None => cancel.call(()),
                        // A flick let go before the measure still lands where it was let go.
                        Some(measured) if released => {
                            lifted.set(Some(measured));
                            finish(true);
                        }
                        Some(measured) => lift(measured),
                    }
                }),
            );
        }),
        onmove: use_callback(move |step: DragMove| {
            let delta = step.delta();
            travel.set((delta.x, delta.y));
            let board = lifted.peek().as_ref().and_then(|up| up.board);
            let next = board.map_or(0.0, |board| edge_step(step.client.x, board));
            edge.set(next);
            match (next != 0.0, auto_scroll.active()) {
                (true, false) => auto_scroll.start(),
                (false, true) => auto_scroll.stop(),
                _ => {}
            }
        }),
        onend: use_callback(move |()| {
            if starting.peek().0 {
                starting.set((true, true));
                return;
            }
            finish(true);
        }),
    });
    let cancel = use_callback(move |()| finish(false));

    BoardDrag {
        element,
        sorting,
        target,
        onpointermove: drag.onpointermove,
        onpointerup: drag.onpointerup,
        onpointercancel: drag.onpointercancel,
        cards,
        lists,
        next_id,
        pressed,
        lifted,
        travel,
        scrolled,
        settle,
        onpointerdown: drag.onpointerdown,
        cancel,
    }
}

/// A column's side of the board drag: registers its card list, and returns the
/// room it grows by below its cards while a card from another hovers there.
pub(super) fn use_board_list(drag: BoardDrag, column: usize, list: ElementHandle) -> Memo<f64> {
    let mut lists = drag.lists;
    let mut at = use_hook(|| CopyValue::new(None::<usize>));
    use_effect(use_reactive!(|column| {
        let _ = list.mount_token();
        let mut lists = lists.write();
        if let Some(old) = at.replace(Some(column))
            && let Some(entry) = lists.get_mut(old)
        {
            *entry = None;
        }
        if lists.len() <= column {
            lists.resize(column + 1, None);
        }
        lists[column] = list.mounted();
    }));
    use_drop(move || {
        if let Ok(mut lists) = lists.try_write()
            && let Some(old) = *at.peek()
        {
            if let Some(entry) = lists.get_mut(old) {
                *entry = None;
            }
            while lists.last().is_some_and(Option::is_none) {
                lists.pop();
            }
        }
    });
    let (lifted, target) = (drag.lifted, drag.target);
    use_memo(use_reactive!(|column| {
        let lifted = lifted.read();
        match (lifted.as_ref(), target()) {
            (Some(lifted), Some(target)) => lifted.lanes.room(column, target),
            _ => 0.0,
        }
    }))
}

/// A card's side of the board drag, from [`use_board_card`].
#[derive(Clone, Copy)]
pub(super) struct BoardCard {
    /// The handle's.
    pub(super) onpointerdown: Callback<Event<PointerData>>,
    /// The handle's: Escape puts a dragged card back; otherwise the column's keys.
    pub(super) onkeydown: Callback<Event<KeyboardData>>,
    /// This card is the one being dragged.
    pub(super) dragging: Memo<bool>,
    offset: Memo<Option<(f64, f64)>>,
    settle: Memo<Option<(f64, f64)>>,
}

impl BoardCard {
    /// The card's inline style while the board drags or it settles; `None` otherwise.
    pub(super) fn style(&self) -> Option<String> {
        let translate = |(x, y): (f64, f64)| format!("translate({x}px, {y}px)");
        if let Some(offset) = (self.offset)() {
            return Some(format!("transform: {}", translate(offset)));
        }
        let from = (self.settle)()?;
        Some(format!(
            "transform: translate(0px, 0px); {SORTABLE_SETTLE_FROM}: {}; animation: {SORTABLE_SETTLE} {} {}",
            translate(from),
            TRANSITION_DURATION.value(),
            TRANSITION_EASING.value()
        ))
    }
}

/// Card `index` of the column at `column`: registers it with the board drag.
/// `keys` is the column's keyboard handler for the handle.
pub(super) fn use_board_card(
    drag: BoardDrag,
    column: Signal<usize>,
    index: usize,
    nodes: (ElementHandle, ElementHandle),
    label: String,
    keys: Callback<Event<KeyboardData>>,
) -> BoardCard {
    let (element, handle) = nodes;
    let id = use_hook(|| {
        let mut next_id = drag.next_id;
        let id = *next_id.peek();
        next_id.set(id + 1);
        id
    });
    let mut cards = drag.cards;
    use_effect(use_reactive!(|index, label| {
        let _ = (element.mount_token(), handle.mount_token());
        cards.write().insert(
            id,
            Placed {
                column: column(),
                index,
                element: element.mounted(),
                handle: handle.mounted(),
                label: label.clone(),
            },
        );
    }));
    use_drop(move || {
        if let Ok(mut cards) = cards.try_write() {
            cards.remove(&id);
        }
    });

    let BoardDrag {
        lifted,
        travel,
        scrolled,
        target,
        settle,
        ..
    } = drag;
    let offset = use_memo(use_reactive!(|index| {
        let lifted = lifted.read();
        let lanes = &lifted.as_ref()?.lanes;
        let at = column();
        if (lanes.from_column, lanes.from) == (at, index) {
            let (dx, dy) = travel();
            return Some((dx + scrolled(), dy));
        }
        Some((0.0, target().map_or(0.0, |to| lanes.step(at, index, to))))
    }));
    let dragging = use_memo(use_reactive!(|index| {
        lifted.read().as_ref().is_some_and(|lifted| {
            (lifted.lanes.from_column, lifted.lanes.from) == (column(), index)
        })
    }));
    let settle = use_memo(use_reactive!(|index| {
        settle()
            .filter(|settle| (settle.column, settle.index) == (column(), index))
            .map(|settle| settle.offset)
    }));

    let mut pressed = drag.pressed;
    let onpointerdown = use_callback(move |event: Event<PointerData>| {
        pressed.set(Some(id));
        drag.onpointerdown.call(event);
    });
    let onkeydown = use_callback(move |event: Event<KeyboardData>| {
        if !*drag.sorting.peek() {
            keys.call(event);
        } else if event.key() == Key::Escape {
            event.prevent_default();
            drag.cancel.call(());
        }
    });

    BoardCard {
        onpointerdown,
        onkeydown,
        dragging,
        offset,
        settle,
    }
}
