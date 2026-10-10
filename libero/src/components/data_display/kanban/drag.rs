//! A card's pointer drag across a [`Kanban`](super::Kanban): one drag for the
//! whole board, so a card can leave its column. The keyboard and the move
//! buttons stay on each column's sortable.

use std::{collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;

use super::{
    lanes::{Lane, Lanes, Rect},
    moves::KanbanMove,
    shown::drop_index,
};
use crate::{
    components::accessibility::Announcer,
    hooks::{
        DragMove, DragOptions, DragStart, ElementHandle, current_localization, edge_scroll_step,
        use_distance_drag, use_early_measure, use_element, use_interval,
    },
    localization::fill,
    platform::{self, Dimensions, ElementApi, PlatformError, Read},
    theme::{SORTABLE_SETTLE, SORTABLE_SETTLE_FROM, TRANSITION_DURATION, TRANSITION_EASING},
};

type Mounted = Option<Rc<MountedData>>;

/// The auto-scroll tick near an edge. Natively each costs a thread, only while there.
const AUTO_SCROLL_MS: u64 = 40;

/// The least and most scroll offset along each axis, measured before the lifted card's
/// transform widened the board's overflow: an axis with no overflow is `(0, 0)`.
#[derive(Clone, Copy, PartialEq)]
struct ScrollRange {
    x: (f64, f64),
    y: (f64, f64),
}

impl ScrollRange {
    /// Px per tick the board scrolls each way with the pointer at `client` in `board`. Up
    /// and down only toward the edge the pointer travelled to (`moved_y` since the press),
    /// so a card lifted in an edge zone stays put.
    fn step(&self, board: Rect, client: (f64, f64), moved_y: f64) -> (f64, f64) {
        let scrolls = |(least, most): (f64, f64)| most > least;
        let x = match scrolls(self.x) {
            true => edge_scroll_step(client.0, board.x, board.width),
            false => 0.0,
        };
        let y = match scrolls(self.y) {
            true => edge_scroll_step(client.1, board.y, board.height),
            false => 0.0,
        };
        (x, if y * moved_y > 0.0 { y } else { 0.0 })
    }
}

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
    /// The dragged card's board id.
    id: usize,
    lanes: Lanes,
    label: String,
    /// The board's rect, `None` when it could not be read.
    board: Option<Rect>,
    /// The board's scroll offset at the lift.
    scroll: (f64, f64),
    scroll_range: ScrollRange,
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
    /// A column's own sortable has focused the move, so only the announcement waits.
    Keep,
}

/// Where a card moved by the board lands, for its new self to take the focus.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Landing {
    pub(super) column: usize,
    pub(super) index: usize,
    pub(super) control: Control,
    /// The moved card's board id and the slot it left.
    pub(super) card: usize,
    pub(super) from: (usize, usize),
    /// The board render in which the moved card left its slot.
    pub(super) left: Option<u64>,
    /// A card at the slot claimed it and waits for its control to mount.
    pub(super) held: bool,
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
    landing: CopyValue<Option<Landing>>,
    /// What to announce once the moved card shows at its new slot (2901).
    pending: CopyValue<Option<String>>,
    /// The render cycles so far, see [`tick`](Self::tick).
    renders: CopyValue<u64>,
    /// A render or drop has opened this cycle and the effects have not run yet.
    fresh: CopyValue<bool>,
    /// Written when a cycle opens: its effect reopens `fresh`.
    flush: Signal<u64>,
    next_id: CopyValue<usize>,
    pressed: CopyValue<Option<usize>>,
    lifted: Signal<Option<Lifted>>,
    travel: Signal<(f64, f64)>,
    /// Px the board scrolled since the lift: the cards moved the other way.
    scrolled: Signal<(f64, f64)>,
    settle: Signal<Option<Settle>>,
    /// Starts a card's measure at its press.
    press: Callback<usize>,
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

    /// Called by every board, column and card render and card drop: the first one after the
    /// effects ran opens the next render cycle. An app's data change may reach only a column
    /// or a card, not the board (2580).
    pub(super) fn tick(self) {
        let (mut fresh, mut renders, mut flush) = (self.fresh, self.renders, self.flush);
        let (Ok(mut fresh), Ok(mut renders)) = (fresh.try_write(), renders.try_write()) else {
            return;
        };
        if !*fresh {
            *fresh = true;
            *renders += 1;
            if let Ok(mut flush) = flush.try_write() {
                *flush += 1;
            }
        }
    }

    /// Card `claimant` may take `landing`'s focus: it is the moved card, or that one left its
    /// slot in this render cycle. A move the app refused leaves it in place (2451); a later data
    /// change moving it is no landing (2519).
    pub(super) fn landed(self, landing: Landing, claimant: usize) -> bool {
        claimant == landing.card || landing.held || landing.left == Some(*self.renders.peek())
    }

    /// Holds `text` for [`take_pending`](Self::take_pending): a move the app refuses is never announced.
    pub(super) fn announce_on_landing(self, text: String) {
        let mut pending = self.pending;
        pending.set(Some(text));
    }

    /// The board id of the card at `index` of `column`.
    pub(super) fn card_at(self, column: usize, index: usize) -> Option<usize> {
        let cards = self.cards.peek();
        cards
            .iter()
            .find(|(_, placed)| (placed.column, placed.index) == (column, index))
            .map(|(&id, _)| id)
    }

    /// The held announcement, once the moved card shows at its slot.
    pub(super) fn take_pending(self) -> Option<String> {
        let mut pending = self.pending;
        pending.write().take()
    }

    /// The number of cards mounted in `column`, and the `index` just past its last one:
    /// where Move to lands, the data's end when every card is shown.
    pub(super) fn end(self, column: usize) -> (usize, usize) {
        let cards = self.cards.peek();
        let shown = cards.values().filter(|placed| placed.column == column);
        shown.fold((0, 0), |(count, end), placed| {
            (count + 1, end.max(placed.index + 1))
        })
    }

    /// The `index` a card put in slot `slot` of `column` takes: just before the card
    /// there, after the last one past it.
    pub(super) fn slot_index(self, column: usize, slot: usize) -> usize {
        let cards = self.cards.peek();
        let mut indices: Vec<usize> = cards
            .values()
            .filter(|placed| placed.column == column)
            .map(|placed| placed.index)
            .collect();
        indices.sort_unstable();
        drop_index(&indices, None, slot)
    }

    /// Card `id` left its slot or the board: stamps the landing it armed with this render.
    fn left(self, id: usize) {
        let mut landing = self.landing;
        let render = self.renders.try_peek().map(|renders| *renders);
        if let (Ok(mut armed), Ok(render)) = (landing.try_write(), render)
            && let Some(at) = armed.as_mut().filter(|at| at.card == id)
        {
            at.left = Some(render);
        }
    }
}

type Reading = Read<((f64, f64), Dimensions)>;

fn read(node: &Rc<MountedData>) -> Reading {
    platform::client_rect(node)
}

async fn rect(reading: Reading) -> Option<Rect> {
    let ((x, y), size) = reading.await.ok()?;
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
    let mut pending = use_hook(|| CopyValue::new(None::<String>));
    let next_id = use_hook(|| CopyValue::new(0_usize));
    let renders = use_hook(|| CopyValue::new(0_u64));
    let mut fresh = use_hook(|| CopyValue::new(false));
    let flush = use_signal(|| 0_u64);
    use_effect(move || {
        flush();
        fresh.set(false);
    });
    let mut pressed = use_hook(|| CopyValue::new(None::<usize>));
    let mut lifted = use_signal(|| None::<Lifted>);
    let mut travel = use_signal(|| (0.0_f64, 0.0_f64));
    let mut scrolled = use_signal(|| (0.0_f64, 0.0_f64));
    let mut settle = use_signal(|| None::<Settle>);
    // A release can land before the measure: (measuring, released).
    let mut starting = use_hook(|| CopyValue::new((false, false)));

    let target = use_memo(move || {
        let (dx, dy) = travel();
        let (sx, sy) = scrolled();
        lifted.read().as_ref()?.lanes.target(dx + sx, dy + sy)
    });

    // Near an edge the board scrolls by `edge` px per tick, its cards' geometry with it.
    let mut edge = use_hook(|| CopyValue::new((0.0_f64, 0.0_f64)));
    let auto_scroll = use_interval(
        move || {
            let step = *edge.peek();
            let Some((from, range)) = lifted
                .peek()
                .as_ref()
                .map(|up| (up.scroll, up.scroll_range))
            else {
                return;
            };
            let (sx, sy) = *scrolled.peek();
            let now = (from.0 + sx, from.1 + sy);
            let next = (
                (now.0 + step.0).clamp(range.x.0, range.x.1),
                (now.1 + step.1).clamp(range.y.0, range.y.1),
            );
            if next == now {
                return;
            }
            let _ = element.scroll_to(next.0, next.1);
            let at = element.scroll_offset();
            spawn(async move {
                if let Ok((x, y)) = at.await
                    && lifted.peek().is_some()
                {
                    scrolled.set((x - from.0, y - from.1));
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
        // Each column's mounted cards in order; a filtered column's indices may skip (2517).
        for column in &mut columns {
            column.sort_by_key(|(index, _)| *index);
        }
        let indices: Vec<Vec<usize>> = columns
            .iter()
            .map(|column| column.iter().map(|(index, _)| *index).collect())
            .collect();
        let card_reads: Option<Vec<Vec<Reading>>> = columns
            .iter()
            .map(|column| {
                column
                    .iter()
                    .map(|(_, node)| node.as_ref().map(read))
                    .collect()
            })
            .collect();
        let (Some(list_reads), Some(card_reads)) = (list_reads, card_reads) else {
            started.call(None);
            return;
        };
        settle.set(None);
        let rtl = element.is_rtl();
        let board_rect: Reading = match element.mounted() {
            Some(board) => read(&board),
            None => Box::pin(std::future::ready(Err(PlatformError::Unsupported))),
        };
        let board_reads = (
            board_rect,
            element.scroll_size(),
            element.client_size(),
            element.scroll_offset(),
        );
        spawn(async move {
            let (board, content, client, scroll) = board_reads;
            // Every read at once: one round-trip on a WebView, not one per card (todo 2018).
            let lanes_read = platform::join_all(list_reads.into_iter().zip(card_reads).map(
                |(list, column)| {
                    platform::join(rect(list), platform::join_all(column.into_iter().map(rect)))
                },
            ));
            let ((board, (content, (client, scroll))), lanes_read) = platform::join(
                platform::join(
                    rect(board),
                    platform::join(content, platform::join(client, scroll)),
                ),
                lanes_read,
            )
            .await;
            let content = content.map_or((0.0, 0.0), |size| (size.width, size.height));
            let scroll = scroll.unwrap_or_default();
            // An axis that fits has nothing to scroll: no ticks.
            let overflow = |content: f64, extent: f64| match content > extent + 0.5 {
                true => content - extent,
                false => 0.0,
            };
            // The client box leaves out the border and a classic scrollbar the offset never covers.
            let client = client
                .ok()
                .filter(|_| !platform::scroll_size_spans_border_box())
                .map(|size| (size.width, size.height));
            let (most_x, most_y) = board.map_or((0.0, 0.0), |board| {
                let extent = client.unwrap_or((board.width, board.height));
                (overflow(content.0, extent.0), overflow(content.1, extent.1))
            });
            // Right to left, `scrollLeft` runs from 0 down to minus the overflow.
            let scroll_range = ScrollRange {
                x: match rtl {
                    true => (-most_x, 0.0),
                    false => (0.0, most_x),
                },
                y: (0.0, most_y),
            };
            let mut lanes = Vec::with_capacity(lanes_read.len());
            for ((list, cards), indices) in lanes_read.into_iter().zip(indices) {
                let (Some(list), Some(cards)) = (list, cards.into_iter().collect()) else {
                    started.call(None);
                    return;
                };
                lanes.push(Lane {
                    list,
                    cards,
                    indices,
                });
            }
            let Some(from) = lanes
                .get(card.column)
                .and_then(|lane| lane.indices.iter().position(|&at| at == card.index))
            else {
                started.call(None);
                return;
            };
            // `use_drag` refocuses the handle only on the web.
            if let Some(handle) = &card.handle {
                let _ = platform::element(handle).focus();
            }
            started.call(Some(Lifted {
                id,
                lanes: Lanes {
                    lanes,
                    from_column: card.column,
                    from,
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
        let (sx, sy) = *scrolled.peek();
        let (dx, dy) = (dx + sx, dy + sy);
        let lanes = &ended.lanes;
        let from = (lanes.from_column, lanes.from);
        // Let go off the board, it goes back.
        let landed = commit.then(|| lanes.target(dx, dy)).flatten();
        let to = landed.unwrap_or(from);
        edge.set((0.0, 0.0));
        auto_scroll.stop();
        lifted.set(None);
        travel.set((0.0, 0.0));
        scrolled.set((0.0, 0.0));
        pressed.set(None);
        let count = lanes.lanes[to.0].cards.len();
        let label = &ended.label;
        let said = match (landed.is_some(), to.0 == from.0) {
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
        };
        // A move waits for the render that shows it: the app may refuse it.
        match to == from {
            true => announcer.say(said),
            false => pending.set(Some(said)),
        }
        // Slots count the mounted cards; the move and the landing speak `index`.
        let from_index = lanes.lifted_index();
        let to_index = match to == from {
            true => from_index,
            false => lanes.index_at(to),
        };
        if to != from {
            landing.set(Some(Landing {
                column: to.0,
                index: to_index,
                control: Control::Handle,
                card: ended.id,
                from: (from.0, from_index),
                left: None,
                held: false,
            }));
            onmove.call(KanbanMove {
                from_column: from.0,
                from: from_index,
                to_column: to.0,
                to: to_index,
            });
        }
        let (x, y) = lanes.landing(to);
        let offset = (dx - x, dy - y);
        if offset.0.abs() > 0.5 || offset.1.abs() > 0.5 {
            settle.set(Some(Settle {
                column: to.0,
                index: to_index,
                offset,
            }));
        }
    };

    // A settling card's transform would skew a measure taken now.
    let early = use_early_measure::<Lifted>();
    let press = use_callback(move |id: usize| {
        let idle = lifted.peek().is_none() && settle.peek().is_none() && !starting.peek().0;
        early.press(id, idle, move |started| begin(id, started));
    });

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
            let started = Callback::new(move |measured: Option<Lifted>| {
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
            });
            if !early.claim(id, started) {
                begin(id, started);
            }
        }),
        onmove: use_callback(move |step: DragMove| {
            let delta = step.delta();
            travel.set((delta.x, delta.y));
            let next = lifted.peek().as_ref().map_or((0.0, 0.0), |up| {
                let client = (step.client.x, step.client.y);
                up.board.map_or((0.0, 0.0), |board| {
                    up.scroll_range.step(board, client, delta.y)
                })
            });
            edge.set(next);
            match (next != (0.0, 0.0), auto_scroll.active()) {
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
    let board = BoardDrag {
        element,
        sorting,
        target,
        onpointermove: drag.onpointermove,
        onpointerup: drag.onpointerup,
        onpointercancel: drag.onpointercancel,
        cards,
        lists,
        landing,
        pending,
        renders,
        fresh,
        flush,
        next_id,
        pressed,
        lifted,
        travel,
        scrolled,
        settle,
        press,
        onpointerdown: drag.onpointerdown,
        cancel,
    };
    board.tick();
    board
}

/// A column's side of the board drag: registers its card list, and returns the
/// room it grows by below its cards while a card from another hovers there.
pub(super) fn use_board_list(drag: BoardDrag, column: usize, list: ElementHandle) -> Memo<f64> {
    drag.tick();
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
    /// The card's board id, for its [`Landing`].
    pub(super) id: usize,
    /// The handle's.
    pub(super) onpointerdown: Callback<Event<PointerData>>,
    /// The handle's: Escape puts a dragged card back; otherwise the column's keys.
    pub(super) onkeydown: Callback<Event<KeyboardData>>,
    /// This card is the one being dragged.
    pub(super) dragging: Memo<bool>,
    offset: Memo<Option<(f64, f64)>>,
    settle: Memo<Option<(f64, f64)>>,
    /// The card has been offset or has settled since mount.
    settled: CopyValue<bool>,
}

impl BoardCard {
    /// The card's inline style while the board drags or it settles; `None` otherwise.
    pub(super) fn style(&self) -> Option<String> {
        let translate = |(x, y): (f64, f64)| format!("translate({x}px, {y}px)");
        let mut settled = self.settled;
        if let Some(offset) = (self.offset)() {
            settled.set(true);
            // Closed by `;`: a dropped settle var's reset is appended after it (1438).
            return Some(format!(
                "transform: {}; animation: none;",
                translate(offset)
            ));
        }
        let Some(from) = (self.settle)() else {
            // An omitted transform or animation keeps its last value (3039, 3105).
            return settled
                .peek()
                .then(|| "transform: none; animation: none;".to_string());
        };
        settled.set(true);
        Some(format!(
            "transform: translate(0px, 0px); {SORTABLE_SETTLE_FROM}: {}; animation: {SORTABLE_SETTLE} {} {};",
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
    drag.tick();
    let id = use_hook(|| {
        let mut next_id = drag.next_id;
        let id = *next_id.peek();
        next_id.set(id + 1);
        id
    });
    let mut cards = drag.cards;
    use_effect(use_reactive!(|index, label| {
        let _ = (element.mount_token(), handle.mount_token());
        let at = column();
        cards.write().insert(
            id,
            Placed {
                column: at,
                index,
                element: element.mounted(),
                handle: handle.mounted(),
                label: label.clone(),
            },
        );
        if drag
            .landing
            .peek()
            .is_some_and(|armed| armed.card == id && armed.from != (at, index))
        {
            drag.left(id);
        }
    }));
    use_drop(move || {
        if let Ok(mut cards) = cards.try_write() {
            cards.remove(&id);
        }
        drag.tick();
        drag.left(id);
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
        let up = lifted.as_ref()?;
        if up.id == id {
            let (dx, dy) = travel();
            let (sx, sy) = scrolled();
            return Some((dx + sx, dy + sy));
        }
        let at = column();
        let slot = up.lanes.slot(at, index)?;
        Some((0.0, target().map_or(0.0, |to| up.lanes.step(at, slot, to))))
    }));
    let dragging = use_memo(move || lifted.read().as_ref().is_some_and(|up| up.id == id));
    let settle = use_memo(use_reactive!(|index| {
        settle()
            .filter(|settle| (settle.column, settle.index) == (column(), index))
            .map(|settle| settle.offset)
    }));

    let mut pressed = drag.pressed;
    let onpointerdown = use_callback(move |event: Event<PointerData>| {
        pressed.set(Some(id));
        drag.press.call(id);
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
        id,
        onpointerdown,
        onkeydown,
        dragging,
        offset,
        settle,
        settled: use_hook(|| CopyValue::new(false)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOARD: Rect = Rect {
        x: 0.0,
        y: 100.0,
        width: 600.0,
        height: 200.0,
    };

    fn range(x: f64, y: f64) -> ScrollRange {
        ScrollRange {
            x: (0.0, x),
            y: (0.0, y),
        }
    }

    #[test]
    fn a_board_scrolls_only_along_the_axis_that_overflows() {
        let at_bottom = (300.0, 296.0);
        assert_eq!(range(0.0, 0.0).step(BOARD, at_bottom, 50.0), (0.0, 0.0));
        assert_eq!(range(0.0, 500.0).step(BOARD, at_bottom, 50.0).0, 0.0);
        assert!(range(0.0, 500.0).step(BOARD, at_bottom, 50.0).1 > 0.0);
        assert!(range(400.0, 0.0).step(BOARD, (596.0, 200.0), 0.0).0 > 0.0);
        assert_eq!(range(400.0, 0.0).step(BOARD, (596.0, 296.0), 50.0).1, 0.0);
    }

    #[test]
    fn a_card_lifted_in_an_edge_zone_scrolls_up_or_down_only_once_it_travels_there() {
        let range = range(0.0, 500.0);
        assert_eq!(range.step(BOARD, (300.0, 296.0), 0.0), (0.0, 0.0));
        assert_eq!(range.step(BOARD, (300.0, 296.0), -20.0), (0.0, 0.0));
        assert!(range.step(BOARD, (300.0, 296.0), 3.0).1 > 0.0);
        assert!(range.step(BOARD, (300.0, 104.0), -3.0).1 < 0.0);
        assert_eq!(range.step(BOARD, (300.0, 104.0), 3.0), (0.0, 0.0));
    }
}
