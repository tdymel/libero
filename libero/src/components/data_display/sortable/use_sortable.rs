use std::rc::Rc;

use dioxus::prelude::*;

use super::reorder::{SortableMove, Span, clamp_offset, shift, slot_offset, target_index};
use crate::{
    components::common::Orientation,
    hooks::{
        DragMove, DragOptions, DragStart, ElementHandle, current_localization, edge_scroll_step,
        use_distance_drag, use_early_measure, use_element, use_interval, use_subscription_slot,
    },
    localization::fill,
    platform::{self, Dimensions, ElementApi, Read, ScrollSubscription, scroll},
    theme::{SORTABLE_SETTLE, SORTABLE_SETTLE_FROM, TRANSITION_DURATION, TRANSITION_EASING},
    utils::bump,
};

/// What [`use_sortable`] takes.
pub struct SortableOptions {
    /// `Vertical` stacks the items, `Horizontal` lines them up in a row.
    pub orientation: Orientation,
    /// Called once on drop, only when the item changed place. Apply it to your
    /// data: the list shows its old order until you do.
    pub onreorder: Callback<SortableMove>,
}

/// The list's side of a sortable: spread these onto the element holding the items.
#[derive(Clone, Copy)]
pub struct SortableHandle {
    /// The list: takes the pointer capture while an item drags.
    pub element: ElementHandle,
    /// An item is being dragged, by pointer or keyboard.
    pub sorting: Memo<bool>,
    /// The last lift, move, drop or cancel, in words. Show it in a `role="status"` region.
    pub announcement: Signal<String>,
    pub onpointermove: Callback<Event<PointerData>>,
    pub onpointerup: Callback<Event<PointerData>>,
    pub onpointercancel: Callback<Event<PointerData>>,
}

/// One item's side of a sortable, from [`use_sortable_item`].
#[derive(Clone, Copy)]
pub struct SortableItemHandle {
    /// The item: measured when a drag starts, and moved by [`style`](Self::style).
    pub element: ElementHandle,
    /// The grab handle. Give it `drag_handle_sx()`, or a touch scrolls instead.
    pub handle: ElementHandle,
    /// The grab handle's.
    pub onpointerdown: Callback<Event<PointerData>>,
    /// The grab handle's: Space or Enter lifts and drops, the arrows move, Escape cancels.
    pub onkeydown: Callback<Event<KeyboardData>>,
    /// The grab handle's: focus leaving it cancels a keyboard move.
    pub onblur: Callback<Event<FocusData>>,
    /// A button moving the item one slot up (back, in a row), without a drag.
    pub earlier: ElementHandle,
    pub onearlier: Callback<Event<MouseData>>,
    /// A button moving the item one slot down (forward, in a row).
    pub later: ElementHandle,
    pub onlater: Callback<Event<MouseData>>,
    /// The item is first: there is no slot before it.
    pub first: Memo<bool>,
    /// The item is last: there is no slot after it.
    pub last: Memo<bool>,
    /// This item is the one being dragged.
    pub dragging: Memo<bool>,
    /// Some item of the list is being dragged.
    pub sorting: Memo<bool>,
    offset: Memo<f64>,
    settle: Memo<Option<f64>>,
    horizontal: Memo<bool>,
}

impl SortableItemHandle {
    /// How far the item sits from its slot, in px along the list's axis.
    pub fn offset(&self) -> f64 {
        (self.offset)()
    }

    /// The item's inline `style`: a `transform` moving it by [`offset`](Self::offset)
    /// while it is off its slot (empty at rest), and just after a drop the slide
    /// from where it was let go into its slot.
    pub fn style(&self) -> String {
        self.style_by(0.0)
    }

    /// [`style`](Self::style) with `extra` px more along the flow: a windowed row
    /// laid off its slot (todo 1408).
    pub(crate) fn style_by(&self, extra: f64) -> String {
        let offset = self.offset() + extra;
        let translate = |px: f64| match (self.horizontal)() {
            true => format!("translate({px}px, 0px)"),
            false => format!("translate(0px, {px}px)"),
        };
        // Every declaration closed: the dropped settle var's reset is appended after them (1438).
        // None at rest: an idle transform makes a stacking context and a fixed-position containing block.
        let mut style = match offset == 0.0 {
            true => String::new(),
            false => format!("transform: {};", translate(offset)),
        };
        if let Some(from) = (self.settle)() {
            style.push_str(&format!(
                " {SORTABLE_SETTLE_FROM}: {}; animation: {SORTABLE_SETTLE} {} {};",
                translate(from),
                TRANSITION_DURATION.value(),
                TRANSITION_EASING.value()
            ));
        }
        style
    }
}

type Mounted = Option<Rc<MountedData>>;

/// An item's mounted nodes. Not its `ElementHandle`s: those belong to the
/// item's scope, and the list reading them is a cross-scope read.
#[derive(Clone)]
struct Registered {
    /// Which item, for as long as it is mounted.
    id: usize,
    element: Mounted,
    handle: Mounted,
    /// A sibling node the item spans to, a table row's open detail row.
    extent: Mounted,
    label: Option<String>,
}

/// A drag measured and under way.
#[derive(Clone, PartialEq)]
struct Session {
    from: usize,
    spans: Vec<Span>,
    /// `-1.0` where the flow runs against the screen axis: a row right to left.
    sign: f64,
    /// A keyboard move's slot; `None` while a pointer drags.
    keyed: Option<usize>,
    /// The item's name, for the announcements.
    label: String,
}

impl Session {
    fn target(&self, travel: f64) -> usize {
        self.keyed
            .unwrap_or_else(|| target_index(&self.spans, self.from, travel))
    }

    /// The dragged item's offset along the flow.
    fn offset(&self, travel: f64) -> f64 {
        match self.keyed {
            Some(to) => slot_offset(&self.spans, self.from, to),
            None => clamp_offset(&self.spans, self.from, travel),
        }
    }
}

/// The item just dropped at `to`: it slides in from `offset` px, on screen (1129).
#[derive(Clone, Copy, PartialEq)]
struct Settle {
    id: usize,
    to: usize,
    offset: f64,
}

/// Which of an item's controls takes the focus back after its move.
#[derive(Clone, Copy, PartialEq)]
enum Refocus {
    Handle,
    Earlier,
    Later,
}

/// The item just moved: its move took the focus off its `control`.
#[derive(Clone, Copy, PartialEq)]
struct Moved {
    id: usize,
    control: Refocus,
    step: SortableMove,
}

/// A windowed list's slots, all one pitch, most of them unmounted (Table, todo 1408).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct FixedSlots {
    pub count: usize,
    pub pitch: f64,
    /// A keyboard lift's `(from, to)`, for the list to scroll its target into view.
    pub lift: Signal<Option<(usize, usize)>>,
    /// The box the slots scroll in, vertically: a pointer drag near its edge scrolls it.
    pub scroller: ElementHandle,
    /// The sticky header's height over the scroller's top.
    pub head: f64,
}

/// A fixed list's scroller at a drag's start, read in client px.
#[derive(Clone, Copy)]
struct EdgeScroll {
    /// The edge zones' span: below the header to the bottom.
    start: f64,
    size: f64,
    /// `scrollLeft`, `scrollTop` at the start, and the most `scrollTop`.
    from: (f64, f64),
    most: f64,
}

impl EdgeScroll {
    /// Px per tick the scroller moves with the pointer at `at` along the flow,
    /// `moved` px since the press: only toward the edge it went, so a row lifted there stays put.
    fn step(&self, at: f64, moved: f64) -> f64 {
        let step = edge_scroll_step(at, self.start, self.size);
        if step * moved > 0.0 { step } else { 0.0 }
    }
}

/// The list's client position and scroll offset when a pointer drag measured, so a
/// scroll after it can be read as how far the items moved under the pointer (2540).
#[derive(Clone, Copy)]
struct ListAnchor {
    at: (f64, f64),
    scroll: (f64, f64),
}

/// The edge scroll's tick, as a table column drag's.
const AUTO_SCROLL_MS: u64 = 40;

/// What [`use_sortable_item`] reads from its list.
#[derive(Clone, Copy)]
struct SortableContext {
    fixed: CopyValue<Option<FixedSlots>>,
    registry: CopyValue<Vec<Option<Registered>>>,
    next_id: CopyValue<usize>,
    /// How many items are mounted, to tell a repeated `index`.
    live: CopyValue<usize>,
    refocus: CopyValue<Option<Moved>>,
    pressed: CopyValue<Option<usize>>,
    session: Signal<Option<Session>>,
    /// The pointer's travel along the flow, since the press.
    travel: Signal<f64>,
    target: Memo<Option<usize>>,
    sorting: Memo<bool>,
    horizontal: Memo<bool>,
    count: Signal<usize>,
    settle: Signal<Option<Settle>>,
    /// Starts an item's measure at its press.
    onpress: Callback<usize>,
    onpointerdown: Callback<Event<PointerData>>,
    onkeydown: Callback<(usize, Event<KeyboardData>)>,
    onblur: Callback<usize>,
    onstep: Callback<(usize, bool)>,
}

/// One node's position read, started at once: see `ElementApi::dimensions`.
type NodeRead = Read<((f64, f64), Dimensions)>;

/// Every item's read, and its extent's.
type Reads = Vec<(NodeRead, Option<NodeRead>)>;

/// An item's mounted node, handle and extent.
type MountedItem = (Rc<MountedData>, Mounted, Mounted);

/// Each item's mounted nodes, in order; `None` while a slot is empty.
fn mounted(registry: &[Option<Registered>]) -> Option<Vec<MountedItem>> {
    registry
        .iter()
        .map(|item| {
            let item = item.as_ref()?;
            Some((
                item.element.clone()?,
                item.handle.clone(),
                item.extent.clone(),
            ))
        })
        .collect()
}

/// What is wrong with the items' `index`es when they are not exactly `0..live` (2512).
fn index_fault(registry: &[Option<Registered>], live: usize) -> Option<String> {
    if let Some(gap) = registry.iter().position(Option::is_none) {
        return Some(format!(
            "Sortable: no item has index {gap}, so no drag or lift starts; number the items 0..n."
        ));
    }
    (registry.len() != live).then(|| {
        format!(
            "Sortable: {live} items share {} indices, so some never move; number the items 0..n.",
            registry.len()
        )
    })
}

fn read_node(node: &Rc<MountedData>) -> NodeRead {
    platform::client_rect(node)
}

fn start_reads(items: &[MountedItem]) -> Reads {
    items
        .iter()
        .map(|(item, _, extent)| (read_node(item), extent.as_ref().map(read_node)))
        .collect()
}

/// A node's span along the flow, `None` when a read failed.
async fn node_span(read: NodeRead, vertical: bool, flipped: bool) -> Option<Span> {
    let Ok(((x, y), size)) = read.await else {
        return None;
    };
    Some(match (vertical, flipped) {
        (true, _) => Span {
            start: y,
            size: size.height,
        },
        (false, false) => Span {
            start: x,
            size: size.width,
        },
        (false, true) => Span {
            start: -(x + size.width),
            size: size.width,
        },
    })
}

/// The spans along the flow, each item's up to its extent's far edge; `None` when a read failed.
async fn spans(reads: Reads, vertical: bool, flipped: bool) -> Option<Vec<Span>> {
    let spans = reads.into_iter().map(|(item, extent)| async move {
        let item = node_span(item, vertical, flipped);
        let Some(extent) = extent else {
            return item.await;
        };
        let (span, extent) = platform::join(item, node_span(extent, vertical, flipped)).await;
        let (span, extent) = (span?, extent?);
        let start = span.start.min(extent.start);
        Some(Span {
            start,
            size: span.end().max(extent.end()) - start,
        })
    });
    platform::join_all(spans).await.into_iter().collect()
}

/// Where a drag's spans come from.
enum Measure {
    Read(Reads),
    Fixed(FixedSlots),
}

/// `count` slots of `pitch` px, end to end.
fn fixed_spans(count: usize, pitch: f64) -> Vec<Span> {
    (0..count)
        .map(|slot| Span {
            start: slot as f64 * pitch,
            size: pitch,
        })
        .collect()
}

const ZERO_WIDTH: char = '\u{200B}';

/// `template` with the item's `label` and position `index` of `count`.
fn say(template: &str, label: &str, index: usize, count: usize) -> String {
    fill(
        template,
        &[("label", &label), ("n", &(index + 1)), ("m", &count)],
    )
}

fn lifts(key: &Key) -> bool {
    match key {
        Key::Character(c) => c == " ",
        Key::Enter => true,
        _ => false,
    }
}

/// A list whose items reorder by dragging their handle. Call
/// [`use_sortable_item`] in each item's component below it.
///
/// A press on a handle drags once it moved a few px, so a click stays a click.
/// The items step aside while one drags; on drop `onreorder` gets the move.
/// On the keyboard, Space lifts the focused handle's item, the arrows move it,
/// Space drops it and Escape puts it back.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Box, Orientation, VisuallyHidden};
/// # use libero::hooks::{
/// #     SortableMove, SortableOptions, drag_handle_sx, use_sortable, use_sortable_item,
/// # };
/// # fn app() -> Element {
/// let mut fruit = use_signal(|| vec!["apple", "pear", "plum"]);
/// let list = use_sortable(SortableOptions {
///     orientation: Orientation::Vertical,
///     onreorder: Callback::new(move |step: SortableMove| step.apply(&mut fruit.write())),
/// });
///
/// rsx! {
///     Box {
///         onmounted: list.element.mount(),
///         onpointermove: move |event| list.onpointermove.call(event),
///         onpointerup: move |event| list.onpointerup.call(event),
///         onpointercancel: move |event| list.onpointercancel.call(event),
///         for (index, name) in fruit().into_iter().enumerate() {
///             Fruit { key: "{name}", index, name }
///         }
///     }
///     VisuallyHidden { role: "status", {list.announcement} }
/// }
/// # }
///
/// #[component]
/// fn Fruit(index: usize, name: &'static str) -> Element {
///     let item = use_sortable_item(index);
///     rsx! {
///         Box { onmounted: item.element.mount(), style: item.style(),
///             button {
///                 onmounted: item.handle.mount(),
///                 onpointerdown: move |event| item.onpointerdown.call(event),
///                 onkeydown: move |event| item.onkeydown.call(event),
///                 onblur: move |event| item.onblur.call(event),
///                 style: "touch-action: none",
///                 "Drag {name}"
///             }
///         }
///     }
/// }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/sortable>
pub fn use_sortable(options: SortableOptions) -> SortableHandle {
    use_fixed_sortable(options, None)
}

/// [`use_sortable`] over `fixed` slots, when set: the spans come from the pitch,
/// not from measuring every item, which a window leaves unmounted.
pub(crate) fn use_fixed_sortable(
    options: SortableOptions,
    fixed: Option<FixedSlots>,
) -> SortableHandle {
    let SortableOptions {
        orientation,
        onreorder,
    } = options;
    let mut fixed_slots = use_hook(|| CopyValue::new(fixed));
    fixed_slots.set(fixed);
    let element = use_element();
    let horizontal = orientation == Orientation::Horizontal;
    let horizontal = use_memo(use_reactive!(|horizontal| horizontal));
    let words = current_localization().sortable;

    let registry = use_hook(|| CopyValue::new(Vec::<Option<Registered>>::new()));
    let next_id = use_hook(|| CopyValue::new(0_usize));
    let live = use_hook(|| CopyValue::new(0_usize));
    let mut refocus = use_hook(|| CopyValue::new(None::<Moved>));
    let mut pressed = use_hook(|| CopyValue::new(None::<usize>));
    let mut session = use_signal(|| None::<Session>);
    let mut travel = use_signal(|| 0.0_f64);
    let mut announcement = use_signal(String::new);
    let count = use_signal(|| 0_usize);
    let mut settle = use_signal(|| None::<Settle>);
    // A release can land before the measure: (measuring, released).
    let mut starting = use_hook(|| CopyValue::new((false, false)));
    // The drag's direction sign, read at its start so moves before the measure use it.
    let mut drag_sign = use_hook(|| CopyValue::new(1.0_f64));
    // Fixed slots: the scroller once read, the pointer's (client position, travel)
    // along the flow, and how far the edge scroll moved the slots since the start (todo 1872).
    let mut edge = use_hook(|| CopyValue::new(None::<EdgeScroll>));
    let mut pointer = use_hook(|| CopyValue::new((0.0_f64, 0.0_f64)));
    let mut scrolled = use_hook(|| CopyValue::new(0.0_f64));
    // Plain lists: the list's place at the measure, and a tick per scroll during a pointer drag.
    let mut anchor = use_hook(|| CopyValue::new(None::<ListAnchor>));
    let scroll_tick = use_signal(|| 0_u64);
    let listening = use_subscription_slot::<dyn ScrollSubscription>();
    let auto_scroll = use_interval(
        move || {
            let (Some(scroll), Some(slots)) = (*edge.peek(), *fixed_slots.peek()) else {
                return;
            };
            if session.peek().is_none() {
                return;
            }
            let now = scroll.from.1 + *scrolled.peek();
            let (at, moved) = *pointer.peek();
            let next = (now + scroll.step(at, moved)).clamp(0.0, scroll.most);
            if next == now {
                return;
            }
            let _ = slots.scroller.scroll_to(scroll.from.0, next);
            let at = slots.scroller.scroll_offset();
            spawn(async move {
                let Ok((_, top)) = at.await else {
                    return;
                };
                // Dropped meanwhile: the travel is spent.
                if edge.peek().is_none() {
                    return;
                }
                scrolled.set(top - scroll.from.1);
                travel.set(pointer.peek().1 + top - scroll.from.1);
            });
        },
        AUTO_SCROLL_MS,
    );

    let target = use_memo(move || {
        let session = session.read();
        Some(session.as_ref()?.target(travel()))
    });
    let sorting = use_memo(move || session.read().is_some());

    let label_of = move |index: usize| -> (usize, String) {
        let registry = registry.peek();
        let item = registry.get(index).cloned().flatten();
        let id = item.as_ref().map_or(usize::MAX, |item| item.id);
        let label = item
            .and_then(|item| item.label)
            .unwrap_or_else(|| fill(words.item, &[("n", &(index + 1))]));
        (id, label)
    };

    // A status region rereads only a changed text: a repeat flips a trailing zero-width space.
    let mut announce = move |text: String| {
        let flip = {
            let now = announcement.peek();
            now.trim_end_matches(ZERO_WIDTH) == text && !now.ends_with(ZERO_WIDTH)
        };
        announcement.set(if flip {
            format!("{text}{ZERO_WIDTH}")
        } else {
            text
        });
    };

    let mut lift = move |lifted: Session| {
        refocus.set(None);
        announce(say(
            words.lifted,
            &lifted.label,
            lifted.from,
            lifted.spans.len(),
        ));
        session.set(Some(lifted));
    };

    // Measures the list, then calls `started` with the session; `None` when it can't.
    let mut begin = move |from: usize, keyed: bool, started: Callback<Option<Session>>| {
        let flipped = *horizontal.peek() && element.is_rtl();
        let vertical = !*horizontal.peek();
        let (reads, handle) = match *fixed_slots.peek() {
            Some(slots) if from < slots.count => {
                let handle = registry.peek().get(from).cloned().flatten();
                (Measure::Fixed(slots), handle.and_then(|item| item.handle))
            }
            Some(_) => {
                started.call(None);
                return;
            }
            None => {
                if let Some(fault) = index_fault(&registry.peek(), *live.peek()) {
                    crate::utils::warn(&fault);
                }
                let items = mounted(&registry.peek());
                let Some(items) = items.filter(|items| from < items.len()) else {
                    started.call(None);
                    return;
                };
                (Measure::Read(start_reads(&items)), items[from].1.clone())
            }
        };
        let label = label_of(from).1;
        settle.set(None);
        let list_reads = (!keyed && fixed_slots.peek().is_none())
            .then(|| (element.client_offset(), element.scroll_offset()));
        spawn(async move {
            let spans = match reads {
                Measure::Read(reads) => spans(reads, vertical, flipped).await,
                Measure::Fixed(slots) => Some(fixed_spans(slots.count, slots.pitch)),
            };
            let Some(spans) = spans else {
                started.call(None);
                return;
            };
            if let Some((at, scroll)) = list_reads
                && let (Ok(at), Ok(scroll)) = (at.await, scroll.await)
            {
                anchor.set(Some(ListAnchor { at, scroll }));
            }
            // `use_drag` refocuses the handle only on the web.
            if !keyed && let Some(handle) = &handle {
                let _ = platform::element(handle).focus();
            }
            started.call(Some(Session {
                from,
                spans,
                sign: if flipped { -1.0 } else { 1.0 },
                keyed: keyed.then_some(from),
                label,
            }));
        });
    };

    // Ends the session at `to`; `commit: false` puts the item back.
    let mut finish = move |commit: bool| {
        let Some(ended) = session.peek().clone() else {
            return;
        };
        let travel_now = *travel.peek();
        let to = if commit {
            ended.target(travel_now)
        } else {
            ended.from
        };
        session.set(None);
        travel.set(0.0);
        pressed.set(None);
        auto_scroll.stop();
        edge.set(None);
        anchor.set(None);
        scrolled.set(0.0);
        let count = ended.spans.len();
        let words_for = if commit {
            words.dropped
        } else {
            words.cancelled
        };
        announce(say(words_for, &ended.label, to, count));
        let (id, _) = label_of(ended.from);
        let from_slot = ended.offset(travel_now) - slot_offset(&ended.spans, ended.from, to);
        if from_slot.abs() > 0.5 {
            settle.set(Some(Settle {
                id,
                to,
                offset: from_slot * ended.sign,
            }));
        }
        if to != ended.from {
            let step = SortableMove {
                from: ended.from,
                to,
            };
            refocus.set(Some(Moved {
                id,
                control: Refocus::Handle,
                step,
            }));
            onreorder.call(step);
        }
    };

    // A settling item's transform would skew a measure taken now.
    let early = use_early_measure::<Session>();
    let onpress = use_callback(move |index: usize| {
        let idle = session.peek().is_none() && settle.peek().is_none() && !starting.peek().0;
        early.press(index, idle, move |started| begin(index, false, started));
    });

    let drag = use_distance_drag(DragOptions {
        capture: element,
        onstart: use_callback(move |start: DragStart| {
            let cancel = start.cancel;
            let from = *pressed.peek();
            let (Some(from), false) = (from, session.peek().is_some()) else {
                cancel.call(());
                return;
            };
            starting.set((true, false));
            travel.set(0.0);
            let flipped = *horizontal.peek() && element.is_rtl();
            drag_sign.set(if flipped { -1.0 } else { 1.0 });
            scrolled.set(0.0);
            edge.set(None);
            pointer.set((start.client.y, 0.0));
            if let Some(slots) = *fixed_slots.peek()
                && !*horizontal.peek()
            {
                // Started in the handler, as Blitz needs.
                let scroller = slots.scroller;
                let (offset, size) = (scroller.client_offset(), scroller.dimensions());
                let (at, content) = (scroller.scroll_offset(), scroller.scroll_size());
                spawn(async move {
                    let (Ok((_, y)), Ok(size), Ok(from), Ok(content)) =
                        (offset.await, size.await, at.await, content.await)
                    else {
                        return;
                    };
                    let scroll = EdgeScroll {
                        start: y + slots.head,
                        size: size.height - slots.head,
                        from,
                        most: (content.height - size.height).max(0.0),
                    };
                    edge.set(Some(scroll));
                    // The pointer may already wait at the edge.
                    let (at, moved) = *pointer.peek();
                    if scroll.step(at, moved) != 0.0 && !auto_scroll.active() {
                        auto_scroll.start();
                    }
                });
            }
            let started = Callback::new(move |measured: Option<Session>| {
                let (_, released) = starting.replace((false, false));
                match measured {
                    None => cancel.call(()),
                    // A flick let go before the measure still lands where it was let go.
                    Some(measured) if released => {
                        session.set(Some(measured));
                        finish(true);
                    }
                    Some(measured) => lift(measured),
                }
            });
            if !early.claim(from, started) {
                begin(from, false, started);
            }
        }),
        onmove: use_callback(move |step: DragMove| {
            let delta = step.delta();
            let along = if *horizontal.peek() { delta.x } else { delta.y };
            let moved = along * *drag_sign.peek();
            // Kept before the measure lands too, so the first frame is not a jump.
            travel.set(moved + *scrolled.peek());
            pointer.set((step.client.y, moved));
            let near =
                (*edge.peek()).is_some_and(|scroll| scroll.step(step.client.y, moved) != 0.0);
            match (near, auto_scroll.active()) {
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
            if session.peek().as_ref().is_some_and(|s| s.keyed.is_none()) {
                finish(true);
            }
        }),
    });

    let onkeydown = use_callback(move |(index, event): (usize, Event<KeyboardData>)| {
        let key = event.key();
        // A held Space or Enter must not toggle lift and drop at the repeat rate (2509).
        if lifts(&key) && event.is_auto_repeating() {
            event.prevent_default();
            return;
        }
        let keyed = session
            .peek()
            .as_ref()
            .and_then(|s| s.keyed.map(|to| (s.from, to, s.spans.len(), s.sign)));
        let Some((from, to, count, sign)) = keyed else {
            // Escape puts a pointer drag's item back too.
            if key == Key::Escape && session.peek().is_some() {
                event.prevent_default();
                finish(false);
            } else if lifts(&key) && session.peek().is_none() && !starting.peek().0 {
                event.prevent_default();
                starting.set((true, false));
                begin(
                    index,
                    true,
                    Callback::new(move |measured: Option<Session>| {
                        starting.set((false, false));
                        if let Some(measured) = measured {
                            lift(measured);
                        }
                    }),
                );
            }
            return;
        };
        if index != from {
            return;
        }
        let last = count.saturating_sub(1);
        // In a right to left row the next slot sits to the left.
        let step = |forward: bool| if forward == (sign > 0.0) { 1 } else { -1 };
        let next = match key {
            _ if lifts(&key) => {
                event.prevent_default();
                finish(true);
                return;
            }
            Key::Escape => {
                event.prevent_default();
                finish(false);
                return;
            }
            Key::ArrowDown if !*horizontal.peek() => to as isize + 1,
            Key::ArrowUp if !*horizontal.peek() => to as isize - 1,
            Key::ArrowRight if *horizontal.peek() => to as isize + step(true),
            Key::ArrowLeft if *horizontal.peek() => to as isize + step(false),
            Key::Home => 0,
            Key::End => last as isize,
            _ => return,
        };
        event.prevent_default();
        let next = next.clamp(0, last as isize) as usize;
        if next == to {
            let label = session.peek().as_ref().map(|s| s.label.clone());
            announce(say(words.unmoved, &label.unwrap_or_default(), to, count));
            return;
        }
        if let Some(lifted) = session.write().as_mut() {
            lifted.keyed = Some(next);
            announce(say(words.moved, &lifted.label, next, count));
        }
    });

    let onblur = use_callback(move |index: usize| {
        let lifted = session
            .peek()
            .as_ref()
            .is_some_and(|s| s.keyed.is_some() && s.from == index);
        if lifted {
            finish(false);
        }
    });

    // A move button's: one slot, no drag.
    let onstep = use_callback(move |(index, later): (usize, bool)| {
        let total = *count.peek();
        if session.peek().is_some() || total == 0 {
            return;
        }
        let to = match later {
            true if index + 1 < total => index + 1,
            false if index > 0 => index - 1,
            _ => return,
        };
        let (id, label) = label_of(index);
        // A button that turns disabled drops the focus: the other one takes it.
        let keep = match (later, to == 0, to + 1 == total) {
            (true, _, true) => Refocus::Earlier,
            (false, true, _) => Refocus::Later,
            (true, _, _) => Refocus::Later,
            (false, _, _) => Refocus::Earlier,
        };
        let step = SortableMove { from: index, to };
        refocus.set(Some(Moved {
            id,
            control: keep,
            step,
        }));
        announce(say(words.moved, &label, to, total));
        onreorder.call(step);
    });

    // Fixed slots count every row, mounted or not; a keyboard lift is told up.
    let slot_count = fixed.map(|slots| slots.count);
    use_effect(use_reactive!(|slot_count| {
        let mut count = count;
        if let Some(slots) = slot_count
            && *count.peek() != slots
        {
            count.set(slots);
        }
    }));
    use_effect(move || {
        let keyed = session
            .read()
            .as_ref()
            .and_then(|lifted| Some((lifted.from, lifted.keyed?)));
        match fixed_slots.peek().as_ref().map(|slots| slots.lift) {
            Some(mut lift) => {
                if *lift.peek() != keyed {
                    lift.set(keyed);
                }
            }
            // The lifted item moves by `transform` only: keep it in view per step (2510).
            None => {
                let node = keyed
                    .and_then(|(from, _)| registry.peek().get(from).cloned().flatten()?.element);
                if let Some(node) = node {
                    let _ = platform::scroll_chain_into_view(&node, false);
                }
            }
        }
    });

    // A wheel scroll mid-drag moves the items under the pointer: the travel follows it (2540).
    use_effect(move || {
        let pointer_drag = session.read().as_ref().is_some_and(|s| s.keyed.is_none());
        if !pointer_drag || fixed_slots.peek().is_some() {
            listening.clear();
            return;
        }
        if !listening.is_some()
            && let Some(api) = scroll()
        {
            listening.set(Some(api.on_scroll(Box::new(move || bump(scroll_tick)))));
        }
    });
    use_effect(move || {
        if scroll_tick() == 0 {
            return;
        }
        let Some(start) = *anchor.peek() else {
            return;
        };
        if !session.peek().as_ref().is_some_and(|s| s.keyed.is_none()) {
            return;
        }
        let (at, now) = (element.client_offset(), element.scroll_offset());
        let (vertical, sign) = (!*horizontal.peek(), *drag_sign.peek());
        spawn(async move {
            let (Ok(at), Ok(now)) = (at.await, now.await) else {
                return;
            };
            if session.peek().is_none() {
                return;
            }
            let (moved, inner) = match vertical {
                true => (at.1 - start.at.1, now.1 - start.scroll.1),
                false => (at.0 - start.at.0, now.0 - start.scroll.0),
            };
            let extra = (inner - moved) * sign;
            scrolled.set(extra);
            travel.set(pointer.peek().1 + extra);
        });
    });

    use_context_provider(|| SortableContext {
        fixed: fixed_slots,
        registry,
        next_id,
        live,
        refocus,
        pressed,
        session,
        travel,
        target,
        sorting,
        horizontal,
        count,
        settle,
        onpress,
        onpointerdown: drag.onpointerdown,
        onkeydown,
        onblur,
        onstep,
    });

    SortableHandle {
        element,
        sorting,
        announcement,
        onpointermove: drag.onpointermove,
        onpointerup: drag.onpointerup,
        onpointercancel: drag.onpointercancel,
    }
}

fn owns(item: &Option<Registered>, id: usize) -> bool {
    item.as_ref().is_some_and(|item| item.id == id)
}

/// Drops trailing empty slots, so the length is the item count.
fn trim(items: &mut Vec<Option<Registered>>) {
    while items.last().is_some_and(Option::is_none) {
        items.pop();
    }
}

/// One item of the nearest [`use_sortable`] list, at `index` in its order.
/// Panics outside one. The indices run exactly `0..n`: a gap or a repeat stops every drag.
///
/// Key the item by its data, not its index: a reorder then moves the item,
/// focus included, instead of rebuilding it.
pub fn use_sortable_item(index: usize) -> SortableItemHandle {
    use_labelled_sortable_item(index, None)
}

/// [`use_sortable_item`], named `label` in the announcements.
pub(crate) fn use_labelled_sortable_item(
    index: usize,
    label: Option<String>,
) -> SortableItemHandle {
    use_spanning_sortable_item(index, label, None)
}

/// [`use_labelled_sortable_item`] whose item runs on to `extent`, a sibling
/// node after it that moves with it: measured as one span.
pub(crate) fn use_spanning_sortable_item(
    index: usize,
    label: Option<String>,
    extent: Option<ElementHandle>,
) -> SortableItemHandle {
    let context = try_use_context::<SortableContext>()
        .expect("use_sortable_item: no `use_sortable` list above this component.");
    let element = use_element();
    let handle = use_element();
    let earlier = use_element();
    let later = use_element();
    let mut slot = use_hook(|| CopyValue::new(None::<usize>));
    let id = use_hook(|| {
        let mut next_id = context.next_id;
        let id = *next_id.peek();
        next_id.set(id + 1);
        let mut live = context.live;
        *live.write() += 1;
        id
    });

    let mut registry = context.registry;
    let mut refocus = context.refocus;
    let mut count = context.count;
    // After the mount and every reorder: the DOM is in its new order by then.
    use_effect(use_reactive!(|index, label, extent| {
        let _ = (
            element.mount_token(),
            handle.mount_token(),
            extent.map(|extent| extent.mount_token()),
        );
        let mut items = registry.write();
        let before = slot.replace(Some(index));
        if let Some(old) = before
            && items.get(old).is_some_and(|item| owns(item, id))
        {
            items[old] = None;
        }
        if items.len() <= index {
            items.resize(index + 1, None);
        }
        items[index] = Some(Registered {
            id,
            element: element.mounted(),
            handle: handle.mounted(),
            extent: extent.and_then(|extent| extent.mounted()),
            label: label.clone(),
        });
        trim(&mut items);
        let len = context
            .fixed
            .peek()
            .map_or(items.len(), |slots| slots.count);
        drop(items);
        if *count.peek() != len {
            count.set(len);
        }
        // Moving a node in the DOM blurs it. Only for that move: after an ignored one,
        // a later unrelated change of index must not pull the focus here.
        let moved = *refocus.peek();
        if let Some(Moved { control, step, .. }) = moved.filter(|moved| moved.id == id) {
            refocus.set(None);
            if (before, index) != (Some(step.from), step.to) {
                return;
            }
            let _ = match control {
                Refocus::Handle => handle.focus(),
                Refocus::Earlier => earlier.focus(),
                Refocus::Later => later.focus(),
            };
        }
    }));
    use_drop(move || {
        let mut live = context.live;
        if let Ok(mut live) = live.try_write() {
            *live = live.saturating_sub(1);
        }
        let mut items = registry.write();
        if let Some(index) = *slot.peek()
            && items.get(index).is_some_and(|item| owns(item, id))
        {
            items[index] = None;
            trim(&mut items);
            let len = match context.fixed.try_peek().ok().and_then(|fixed| *fixed) {
                Some(slots) => slots.count,
                None => items.len(),
            };
            drop(items);
            // The list may be going too.
            if let Ok(mut count) = count.try_write() {
                *count = len;
            }
        }
    });

    let SortableContext {
        session,
        travel,
        target,
        settle,
        ..
    } = context;
    let offset = use_memo(use_reactive!(|index| {
        let session = session.read();
        let Some(session) = session.as_ref() else {
            return 0.0;
        };
        let flow = if index == session.from {
            session.offset(travel())
        } else {
            target().map_or(0.0, |to| shift(&session.spans, session.from, to, index))
        };
        flow * session.sign
    }));
    let dragging = use_memo(use_reactive!(|index| {
        session
            .read()
            .as_ref()
            .is_some_and(|session| session.from == index)
    }));
    let settle = use_memo(use_reactive!(|index| {
        settle()
            .filter(|settle| settle.id == id && settle.to == index)
            .map(|settle| settle.offset)
    }));
    let first = use_memo(use_reactive!(|index| index == 0));
    // 0 is no item registered yet, the first render's: unknown, not last (2511).
    let last = use_memo(use_reactive!(|index| {
        let count = count();
        count > 0 && index + 1 >= count
    }));

    let mut pressed = context.pressed;
    let onpointerdown = use_callback(move |event: Event<PointerData>| {
        pressed.set(Some(index));
        context.onpress.call(index);
        context.onpointerdown.call(event);
    });
    let onkeydown = use_callback(move |event| context.onkeydown.call((index, event)));
    let onblur = use_callback(move |_| context.onblur.call(index));
    let onearlier = use_callback(move |_| context.onstep.call((index, false)));
    let onlater = use_callback(move |_| context.onstep.call((index, true)));

    SortableItemHandle {
        element,
        handle,
        onpointerdown,
        onkeydown,
        onblur,
        earlier,
        onearlier,
        later,
        onlater,
        first,
        last,
        dragging,
        sorting: context.sorting,
        offset,
        settle,
        horizontal: context.horizontal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_slots_lie_end_to_end_at_their_pitch() {
        let spans = fixed_spans(3, 40.0);

        assert_eq!(
            spans[2],
            Span {
                start: 80.0,
                size: 40.0
            }
        );
        assert_eq!(slot_offset(&spans, 0, 2), 80.0);
    }

    fn slot(id: usize) -> Option<Registered> {
        Some(Registered {
            id,
            element: None,
            handle: None,
            extent: None,
            label: None,
        })
    }

    #[test]
    fn an_index_gap_or_repeat_is_a_fault_and_0_to_n_is_not() {
        assert_eq!(index_fault(&[slot(0), slot(1)], 2), None);

        let gap = index_fault(&[slot(0), None, slot(2)], 2).unwrap();
        assert!(gap.contains("no item has index 1"), "{gap}");
        let repeat = index_fault(&[slot(0), slot(1)], 3).unwrap();
        assert!(repeat.contains("3 items share 2 indices"), "{repeat}");
    }
}
