use std::rc::Rc;

use dioxus::prelude::*;

use super::reorder::{SortableMove, Span, clamp_offset, shift, target_index};
use crate::{
    components::common::Orientation,
    hooks::{DragMove, DragOptions, DragStart, ElementHandle, use_distance_drag, use_element},
    platform::{self, ElementApi},
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
    /// An item is being dragged.
    pub sorting: Memo<bool>,
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
    /// This item is the one being dragged.
    pub dragging: Memo<bool>,
    /// Some item of the list is being dragged.
    pub sorting: Memo<bool>,
    offset: Memo<f64>,
    horizontal: Memo<bool>,
}

impl SortableItemHandle {
    /// How far the item sits from its slot, in px along the list's axis.
    pub fn offset(&self) -> f64 {
        (self.offset)()
    }

    /// The item's inline `style`: a `transform` moving it by [`offset`](Self::offset).
    pub fn style(&self) -> String {
        let offset = self.offset();
        match (self.horizontal)() {
            true => format!("transform: translate({offset}px, 0px)"),
            false => format!("transform: translate(0px, {offset}px)"),
        }
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
}

/// A drag measured and under way.
#[derive(Clone, PartialEq)]
struct Session {
    from: usize,
    spans: Vec<Span>,
    /// `-1.0` where the flow runs against the screen axis: a row right to left.
    sign: f64,
}

/// What [`use_sortable_item`] reads from its list.
#[derive(Clone, Copy)]
struct SortableContext {
    registry: CopyValue<Vec<Option<Registered>>>,
    next_id: CopyValue<usize>,
    /// The item just dropped elsewhere: its move took the focus off its handle.
    refocus: CopyValue<Option<usize>>,
    pressed: CopyValue<Option<usize>>,
    session: Signal<Option<Session>>,
    /// The pointer's travel along the flow, since the press.
    travel: Signal<f64>,
    target: Memo<Option<usize>>,
    sorting: Memo<bool>,
    horizontal: Memo<bool>,
    onpointerdown: Callback<Event<PointerData>>,
}

/// A list whose items reorder by dragging their handle. Call
/// [`use_sortable_item`] in each item's component below it.
///
/// A press on a handle drags once it moved a few px, so a click stays a click.
/// The items step aside while one drags; on drop `onreorder` gets the move.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Box, Orientation};
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
/// }
/// # }
///
/// #[component]
/// fn Fruit(index: usize, name: &'static str) -> Element {
///     let item = use_sortable_item(index);
///     rsx! {
///         Box { onmounted: item.element.mount(), style: item.style(),
///             Box {
///                 onmounted: item.handle.mount(),
///                 onpointerdown: move |event| item.onpointerdown.call(event),
///                 sx: drag_handle_sx(),
///                 "Drag"
///             }
///             "{name}"
///         }
///     }
/// }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/sortable>
pub fn use_sortable(options: SortableOptions) -> SortableHandle {
    let SortableOptions {
        orientation,
        onreorder,
    } = options;
    let element = use_element();
    let horizontal = orientation == Orientation::Horizontal;
    let horizontal = use_memo(use_reactive!(|horizontal| horizontal));

    let registry = use_hook(|| CopyValue::new(Vec::<Option<Registered>>::new()));
    let next_id = use_hook(|| CopyValue::new(0_usize));
    let mut refocus = use_hook(|| CopyValue::new(None::<usize>));
    let mut pressed = use_hook(|| CopyValue::new(None::<usize>));
    let mut session = use_signal(|| None::<Session>);
    let mut travel = use_signal(|| 0.0_f64);
    // A release can land before the measure: (measuring, released).
    let mut starting = use_hook(|| CopyValue::new((false, false)));

    let target = use_memo(move || {
        let session = session.read();
        let session = session.as_ref()?;
        Some(target_index(&session.spans, session.from, travel()))
    });
    let sorting = use_memo(move || session.read().is_some());

    let drag = use_distance_drag(DragOptions {
        capture: element,
        onstart: use_callback(move |start: DragStart| {
            let cancel = start.cancel;
            // Every slot filled and mounted, or there is no order to measure.
            let items: Option<Vec<(Rc<MountedData>, Mounted)>> = registry
                .peek()
                .iter()
                .map(|item| {
                    let item = item.as_ref()?;
                    Some((item.element.clone()?, item.handle.clone()))
                })
                .collect();
            let (Some(from), Some(items)) = (*pressed.peek(), items) else {
                cancel.call(());
                return;
            };
            if from >= items.len() {
                cancel.call(());
                return;
            }
            let flipped = *horizontal.peek() && element.is_rtl();
            let vertical = !*horizontal.peek();
            starting.set((true, false));
            travel.set(0.0);
            // Started here, awaited in the task: see `ElementApi::dimensions`.
            let reads: Vec<_> = items
                .iter()
                .map(|(item, _)| {
                    let item = platform::element(item);
                    (item.client_offset(), item.dimensions())
                })
                .collect();
            let handle = items[from].1.clone();
            spawn(async move {
                let mut spans = Vec::with_capacity(reads.len());
                for (offset, size) in reads {
                    let (Ok((x, y)), Ok(size)) = (offset.await, size.await) else {
                        starting.set((false, false));
                        cancel.call(());
                        return;
                    };
                    spans.push(match (vertical, flipped) {
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
                    });
                }
                let (_, released) = starting.replace((false, false));
                if released {
                    pressed.set(None);
                    return;
                }
                // `use_drag` refocuses the handle only on the web.
                if let Some(handle) = &handle {
                    let _ = platform::element(handle).focus();
                }
                session.set(Some(Session {
                    from,
                    spans,
                    sign: if flipped { -1.0 } else { 1.0 },
                }));
            });
        }),
        onmove: use_callback(move |step: DragMove| {
            let delta = step.delta();
            let along = if *horizontal.peek() { delta.x } else { delta.y };
            // Kept before the measure lands too, so the first frame is not a jump.
            let sign = session.peek().as_ref().map_or(1.0, |session| session.sign);
            travel.set(along * sign);
        }),
        onend: use_callback(move |()| {
            if starting.peek().0 {
                starting.set((true, true));
                return;
            }
            let landed = session.peek().as_ref().map(|session| {
                let to = target_index(&session.spans, session.from, *travel.peek());
                SortableMove {
                    from: session.from,
                    to,
                }
            });
            session.set(None);
            travel.set(0.0);
            pressed.set(None);
            if let Some(step) = landed
                && step.from != step.to
            {
                let id = registry
                    .peek()
                    .get(step.from)
                    .cloned()
                    .flatten()
                    .map(|item| item.id);
                refocus.set(id);
                onreorder.call(step);
            }
        }),
    });

    use_context_provider(|| SortableContext {
        registry,
        next_id,
        refocus,
        pressed,
        session,
        travel,
        target,
        sorting,
        horizontal,
        onpointerdown: drag.onpointerdown,
    });

    SortableHandle {
        element,
        sorting,
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
/// Panics outside one.
///
/// Key the item by its data, not its index: a reorder then moves the item,
/// focus included, instead of rebuilding it.
pub fn use_sortable_item(index: usize) -> SortableItemHandle {
    let context = try_use_context::<SortableContext>()
        .expect("use_sortable_item: no `use_sortable` list above this component.");
    let element = use_element();
    let handle = use_element();
    let mut slot = use_hook(|| CopyValue::new(None::<usize>));
    let id = use_hook(|| {
        let mut next_id = context.next_id;
        let id = *next_id.peek();
        next_id.set(id + 1);
        id
    });

    let mut registry = context.registry;
    let mut refocus = context.refocus;
    // After the mount and every reorder: the DOM is in its new order by then.
    use_effect(use_reactive!(|index| {
        let _ = (element.mount_token(), handle.mount_token());
        let mut items = registry.write();
        if let Some(old) = slot.replace(Some(index))
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
        });
        trim(&mut items);
        drop(items);
        // Moving a node in the DOM blurs it.
        if *refocus.peek() == Some(id) {
            refocus.set(None);
            let _ = handle.focus();
        }
    }));
    use_drop(move || {
        let mut items = registry.write();
        if let Some(index) = *slot.peek()
            && items.get(index).is_some_and(|item| owns(item, id))
        {
            items[index] = None;
            trim(&mut items);
        }
    });

    let SortableContext {
        session,
        travel,
        target,
        ..
    } = context;
    let offset = use_memo(use_reactive!(|index| {
        let session = session.read();
        let Some(session) = session.as_ref() else {
            return 0.0;
        };
        let flow = if index == session.from {
            clamp_offset(&session.spans, session.from, travel())
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

    let mut pressed = context.pressed;
    let onpointerdown = use_callback(move |event: Event<PointerData>| {
        pressed.set(Some(index));
        context.onpointerdown.call(event);
    });

    SortableItemHandle {
        element,
        handle,
        onpointerdown,
        dragging,
        sorting: context.sorting,
        offset,
        horizontal: context.horizontal,
    }
}
