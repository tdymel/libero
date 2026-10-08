use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use super::{
    TourStep,
    geometry::{has_size, highlight_style, hole_rect, mask_strips, remeasure_key},
};
use crate::{
    components::{
        accessibility::FocusTrap,
        buttons::Button,
        common::{
            FOCUSABLE_SELECTOR, HtmlTag, Input, Part, Parts, attr, has_shortcut_modifier,
            parts_enum, parts_under_sx, use_name_warning,
        },
        layout::{self, use_box},
        overlay::Dialog,
    },
    context::LiberoContext,
    hooks::{
        ElementHandle, ElementRect, FocusReturn, LocalText, POPOVER_AVAILABLE_HEIGHT,
        PopoverOptions, Rect, escape_closes, local_text, use_back, use_dismiss_layer, use_element,
        use_element_rect, use_focus_return, use_id, use_localization, use_popover_on,
        use_portal_slot, use_scheduled, use_theme,
    },
    localization::fill,
    platform::{
        ElementApi, KeyChord, OBSERVE_ATTR, arrow_target, focus_first_of, key_taken, keyboard,
        logical_key, mounted_by_selector, prefers_reduced_motion, scroll_chain_into_view,
        typing_target,
    },
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, Sx, sx},
    theme::{Direction, OVERLAY_OPACITY, Z_INDEX_POPOVER},
    utils::{bump, warn},
};

parts_enum! {
    /// A tour's parts, for [`TourOptions::parts`]. The card's own sit inside it.
    pub enum TourPart {
        /// The transparent layer over the page that takes every press; on an
        /// interactive step, four strips around the hole.
        Mask = "mask" => "& > [data-slot='mask']",
        /// The hole around the target; its outer shadow is the dimming.
        Highlight = "highlight" => "& > [data-slot='highlight']",
        /// Places the card beside the target, or in the middle.
        Positioner = "positioner" => "& > [data-slot='positioner']",
        /// The `dialog`: the default card, or the box around [`TourOptions::card`]'s.
        Card = "card" => "& > [data-slot='positioner'] > [data-slot='card']",
        /// The row holding the title and the close button.
        Header = "header" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='header']",
        Title = "title" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='header'] > [data-slot='title']",
        Close = "close" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='header'] > [data-slot='close']",
        /// The step's description or content.
        Body = "body" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='body']",
        /// The progress text and the buttons.
        Footer = "footer" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='footer']",
        Progress = "progress" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='footer'] > [data-slot='progress']",
        Skip = "skip" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='footer'] > [data-slot='skip']",
        Previous = "previous" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='footer'] > [data-slot='previous']",
        Next = "next" => "& > [data-slot='positioner'] > [data-slot='card'] > [data-slot='footer'] > [data-slot='next']",
    }
}

// The popover layer, so a target inside a modal is covered too.
static TOUR_SX: StaticSx = StaticSx::new(|| {
    let dim = format!("0 0 0 100vmax rgba(0, 0, 0, {})", OVERLAY_OPACITY.value());
    // On a `FocusTrap`, so `display` beats its `contents`.
    sx().display("block")
        .position("fixed")
        .inset("0")
        .z_index(Z_INDEX_POPOVER.value())
        .pointer_events("none")
        .selector(
            "& > [data-slot='mask']",
            sx().position("fixed").inset("0").pointer_events("auto"),
        )
        // A 100vmax spread covers the viewport from any hole; Blitz paints and hit-tests it (2210 probe).
        .selector(
            "& > [data-slot='highlight']",
            sx().position("fixed")
                .pointer_events("none")
                .box_shadow(dim.clone()),
        )
        // A sized hole's own edge: on a dark page the dim alone is about 1.3:1 (1.4.11).
        .selector(
            "& > [data-slot='highlight'][data-ringed]",
            sx().box_shadow(format!("0 0 0 2px currentColor, {dim}"))
                .media(FORCED_COLORS, sx().outline("2px solid CanvasText")),
        )
        // Glides to a new step's target only; a scroll or resize remeasure follows at once.
        .selector(
            "& > [data-slot='highlight'][data-moving]",
            sx().transition("left 0.2s ease, top 0.2s ease, width 0.2s ease, height 0.2s ease")
                .media(REDUCED_MOTION, sx().transition("none")),
        )
        .selector("& > [data-slot='positioner']", sx().pointer_events("none"))
        .selector(
            "& > [data-slot='positioner'] > [data-slot='card']",
            sx().pointer_events("auto"),
        )
});

/// How long a step change may take to glide the hole before it stops waiting for `transitionend`.
const HOLE_GLIDE_FALLBACK_MS: u64 = 500;

// No pointer events: presses beside the card reach the mask.
static CENTRED_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .display("flex")
        // `safe`: a card taller than the viewport starts at the top, not clipped at both ends.
        .align_items("safe center")
        .justify_content("center")
        .padding("md")
        // The card's cap: this box's height inside the padding.
        .var(POPOVER_AVAILABLE_HEIGHT, "100%")
});

static PLACED_SX: StaticSx = StaticSx::new(|| sx().display("block"));

// Never past the room on its side: it scrolls instead (WCAG 1.4.10).
static CARD_SX: StaticSx = StaticSx::new(|| {
    sx().margin("0")
        .max_width("100%")
        .max_height(POPOVER_AVAILABLE_HEIGHT.value_or("none"))
        .overflow_y("auto")
        .selector("& > [data-slot='body']", sx().margin_bottom("md"))
        .selector(
            "& > [data-slot='footer']",
            sx().display("flex")
                .align_items("center")
                .flex_wrap("wrap")
                .gap("xs"),
        )
        .selector(
            "& > [data-slot='footer'] > [data-slot='progress']",
            sx().flex("1").font_size("sm").color("muted.7"),
        )
});

/// What a press on the dimmed page does, for [`TourOptions::mask_click`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MaskClick {
    /// Nothing: a stray tap does not lose the tour.
    #[default]
    None,
    Close,
    Next,
}

/// How a [`use_tour`] behaves. Set `steps`; everything else has a default.
#[derive(Clone, PartialEq)]
pub struct TourOptions {
    pub steps: Vec<TourStep>,
    /// The step shown, controlled: `next()` and the buttons only call `onchange`.
    pub current: Option<usize>,
    /// Called with the step a button, key or handle call asks for.
    pub onchange: Option<Callback<usize>>,
    /// Called once when Next is pressed on the last step.
    pub onfinish: Option<Callback<()>>,
    /// Called with the step shown when the tour is closed early: Escape, Back, Skip, close.
    /// Steps going empty while open close it too, with the last step shown.
    pub onclose: Option<Callback<usize>>,
    pub mask_click: MaskClick,
    /// Names every step's card, over the step titles.
    pub aria_label: Option<String>,
    /// Draws the card's inside in place of the default; the tour keeps placing,
    /// naming and focusing it.
    pub card: Option<Callback<TourView, Element>>,
    /// Styles the card.
    pub sx: Input<Sx>,
    /// Styles the mask, the highlight and the card's parts.
    pub parts: Input<Parts<TourPart>>,
    /// Remembers in local storage that the tour was finished or closed, for
    /// [`TourHandle::seen`]. Read at mount.
    pub storage_key: Option<String>,
}

impl Default for TourOptions {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            current: None,
            onchange: None,
            onfinish: None,
            onclose: None,
            mask_click: MaskClick::None,
            aria_label: None,
            card: None,
            sx: Input::None,
            parts: Input::None,
            storage_key: None,
        }
    }
}

/// The options a handle call needs, written on every render of the owner.
#[derive(Clone, Default)]
struct Latest {
    total: usize,
    current: Option<usize>,
    onchange: Option<Callback<usize>>,
    onfinish: Option<Callback<()>>,
    onclose: Option<Callback<usize>>,
}

/// Starts, moves and ends a [`use_tour`]. `Copy`, so it travels into handlers.
#[derive(Clone, Copy)]
pub struct TourHandle {
    open: Signal<bool>,
    /// The uncontrolled step.
    index: Signal<usize>,
    latest: CopyValue<Latest>,
    focus_return: FocusReturn,
    seen: Option<LocalText>,
}

impl PartialEq for TourHandle {
    fn eq(&self, other: &Self) -> bool {
        self.open == other.open && self.index == other.index
    }
}

impl TourHandle {
    /// Opens on the first step, or on `current` when controlled. Call it from the
    /// trigger's handler, so focus returns there. Does nothing without steps.
    pub fn start(&self) {
        let latest = self.latest.peek().clone();
        if *self.open.peek() || latest.total == 0 {
            return;
        }
        self.focus_return.remember_focused();
        if latest.current.is_none() {
            let mut index = self.index;
            index.set(0);
        }
        let mut open = self.open;
        open.set(true);
    }

    pub fn is_open(&self) -> bool {
        (self.open)()
    }

    /// The step shown, clamped to the steps there are.
    pub fn index(&self) -> usize {
        let latest = self.latest.peek();
        let index = latest.current.unwrap_or_else(|| (self.index)());
        index.min(latest.total.saturating_sub(1))
    }

    pub fn total(&self) -> usize {
        self.latest.peek().total
    }

    /// Whether the tour was finished or closed under its [`TourOptions::storage_key`],
    /// in this run or an earlier one. Reactive; `false` without a key.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// # use libero::components::{Button, TourOptions, TourStep, use_tour};
    /// # fn app() -> Element {
    /// let tour = use_tour(TourOptions {
    ///     steps: vec![TourStep::new("welcome").title("Welcome")],
    ///     storage_key: Some("first-run-tour".into()),
    ///     ..Default::default()
    /// });
    /// rsx! {
    ///     if !tour.seen() {
    ///         Button { onclick: move |_| tour.start(), "Take the tour" }
    ///     }
    ///     Button { onclick: move |_| tour.forget(), "Show the tour again" }
    /// }
    /// # }
    /// ```
    pub fn seen(&self) -> bool {
        self.seen.is_some_and(|seen| seen.is_stored())
    }

    /// Drops what [`seen`](Self::seen) remembers.
    pub fn forget(&self) {
        if let Some(seen) = self.seen {
            seen.remove();
        }
    }

    /// Goes to step `index`, clamped. Controlled, it only calls `onchange`.
    pub fn go_to(&self, index: usize) {
        let latest = self.latest.peek().clone();
        if latest.total == 0 {
            return;
        }
        let index = index.min(latest.total - 1);
        if index == self.index_untracked() {
            return;
        }
        if latest.current.is_none() {
            let mut own = self.index;
            own.set(index);
        }
        if let Some(onchange) = latest.onchange {
            onchange.call(index);
        }
    }

    /// The next step, or [`finish`](Self::finish) on the last.
    pub fn next(&self) {
        let index = self.index_untracked();
        match index + 1 < self.total() {
            true => self.go_to(index + 1),
            false => self.finish(),
        }
    }

    pub fn prev(&self) {
        if let Some(index) = self.index_untracked().checked_sub(1) {
            self.go_to(index);
        }
    }

    /// Ends the tour early: `onclose` with the step shown, focus back to the trigger.
    pub fn close(&self) {
        self.close_at(self.index_untracked());
    }

    fn close_at(&self, index: usize) {
        if self.end() {
            let onclose = self.latest.peek().onclose;
            if let Some(onclose) = onclose {
                onclose.call(index);
            }
        }
    }

    /// Ends the tour as done: `onfinish`, focus back to the trigger.
    pub fn finish(&self) {
        if self.end() {
            let onfinish = self.latest.peek().onfinish;
            if let Some(onfinish) = onfinish {
                onfinish.call(());
            }
        }
    }

    /// Closes; `false` when already closed.
    fn end(&self) -> bool {
        if !*self.open.peek() {
            return false;
        }
        let mut open = self.open;
        open.set(false);
        self.focus_return.restore();
        if let Some(seen) = self.seen {
            seen.set("true".to_string());
        }
        true
    }

    fn index_untracked(&self) -> usize {
        let latest = self.latest.peek();
        let index = latest.current.unwrap_or_else(|| *self.index.peek());
        index.min(latest.total.saturating_sub(1))
    }
}

/// A [`TourHandle`] call from the portal outlet, run in the owner's scope.
#[derive(Clone, Copy, PartialEq)]
enum Move {
    Next,
    /// A key's step: stops on the last step, where only Done finishes.
    Forward,
    Prev,
    GoTo(usize),
    Close,
    Finish,
}

/// The step a [`TourOptions::card`] draws, and the moves its buttons make.
#[derive(Clone, PartialEq)]
pub struct TourView {
    pub index: usize,
    pub total: usize,
    pub step: TourStep,
    /// The default card's "{n} of {m}", in the provider's language.
    pub progress: String,
    act: Callback<Move>,
}

impl TourView {
    pub fn is_first(&self) -> bool {
        self.index == 0
    }

    pub fn is_last(&self) -> bool {
        self.index + 1 >= self.total
    }

    pub fn next(&self) {
        self.act.call(Move::Next);
    }

    pub fn prev(&self) {
        self.act.call(Move::Prev);
    }

    pub fn go_to(&self, index: usize) {
        self.act.call(Move::GoTo(index));
    }

    pub fn close(&self) {
        self.act.call(Move::Close);
    }

    pub fn finish(&self) {
        self.act.call(Move::Finish);
    }
}

/// A guided tour: dims the page around one element per step and explains it in
/// a card beside it, with Back, Next and Skip.
///
/// Escape, Android's Back and the close button end it early; ArrowLeft and
/// ArrowRight step, ArrowRight stopping on the last step: only Done finishes.
/// The page is not scrolled away from: each step scrolls its
/// target into view. Call it in a component that outlives every target.
/// An [`interactive`](TourStep::interactive) step's target takes presses, and Tab
/// moves between it and the card.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, TourOptions, TourStep, use_tour};
/// # use libero::hooks::use_element;
/// # fn app() -> Element {
/// let search = use_element();
/// let tour = use_tour(TourOptions {
///     steps: vec![
///         TourStep::new("welcome").title("Welcome").description("A quick look around."),
///         TourStep::new("search").target(search).title("Search").description("Find anything."),
///     ],
///     ..Default::default()
/// });
/// rsx! {
///     input { onmounted: search.mount(), ..search.attributes() }
///     Button { onclick: move |_| tour.start(), "Take the tour" }
/// }
/// # }
/// ```
pub fn use_tour(options: TourOptions) -> TourHandle {
    let open = use_signal(|| false);
    let index = use_signal(|| 0usize);
    let focus_return = use_focus_return();
    let seen = use_hook(|| options.storage_key.as_deref().map(local_text));
    let mut latest = use_hook(|| CopyValue::new(Latest::default()));
    latest.set(Latest {
        total: options.steps.len(),
        current: options.current,
        onchange: options.onchange,
        onfinish: options.onfinish,
        onclose: options.onclose,
    });
    let handle = TourHandle {
        open,
        index,
        latest,
        focus_return,
        seen,
    };

    use_name_warning(
        options.aria_label.is_some() || options.steps.iter().all(|step| step.title.is_some()),
        "use_tour: a step without a `title` and no `aria_label`, falling back to the localization's. \
         A dialog needs a name of its own to be told apart.",
    );

    // The layer is outside this scope: its moves run here, through a callback made here.
    let act = use_callback(move |step: Move| match step {
        Move::Next => handle.next(),
        Move::Forward => handle.go_to(handle.index_untracked() + 1),
        Move::Prev => handle.prev(),
        Move::GoTo(index) => handle.go_to(index),
        Move::Close => handle.close(),
        Move::Finish => handle.finish(),
    });

    // Root-owned copies of the targets, which the layer may read; each follows its target.
    let mirrors = use_hook(|| CopyValue::new(Vec::<ElementHandle>::new()));
    while mirrors.peek().len() < options.steps.len() {
        let mut mirrors = mirrors;
        mirrors
            .write()
            .push(ElementHandle::new_in_scope(ScopeId::ROOT));
    }
    // `None` for a selector's step: the layer points that mirror at the match.
    let targets: Vec<Option<Option<ElementHandle>>> = options
        .steps
        .iter()
        .map(|step| match (step.target, &step.target_selector) {
            (None, Some(_)) => None,
            (target, _) => Some(target),
        })
        .collect();
    use_effect(use_reactive!(|targets| {
        for (mirror, target) in mirrors.peek().iter().zip(&targets) {
            if let Some(target) = target {
                mirror.follow(target.as_ref());
            }
        }
    }));

    // The owner unmounting takes the tour with it: hand focus back.
    use_drop(move || {
        if open.try_peek().is_ok_and(|open| *open) {
            focus_return.restore_detached();
        }
        if let Ok(mirrors) = mirrors.try_peek() {
            mirrors.iter().for_each(ElementHandle::release);
        }
    });

    // Steps gone while open end the tour, or it would stay open with nothing shown.
    // `onclose` gets the last step shown, not the clamped 0 (todo 2317).
    let mut last_shown = use_hook(|| CopyValue::new(0usize));
    let total = options.steps.len();
    use_effect(use_reactive!(|total| {
        if total == 0 && *open.peek() {
            handle.close_at(*last_shown.peek());
        }
    }));

    let slot = use_portal_slot();
    let shown = open() && !options.steps.is_empty();
    slot.show(shown.then(|| {
        let index = handle.index();
        last_shown.set(index);
        let mut options = options;
        for (step, mirror) in options.steps.iter_mut().zip(mirrors.peek().iter()) {
            match step.target {
                Some(target) => {
                    step.target = Some(mirror.retag(target.tag()));
                    step.target_selector = None;
                }
                None if step.target_selector.is_some() => step.target = Some(*mirror),
                None => {}
            }
        }
        rsx! { TourLayer { act, index, options } }
    }));
    handle
}

/// The open tour in the portal outlet: mask, highlight and the step's card.
#[component]
fn TourLayer(act: Callback<Move>, index: usize, options: TourOptions) -> Element {
    let theme = use_theme();
    let defaults = theme.tour;
    let step = options.steps[index].clone();

    let layer = use_dismiss_layer();
    use_hook(move || Rc::new(layer.push()));
    use_back(true, use_callback(move |()| act.call(Move::Close)));
    // Each step's card mounts its own; focus never leaves it but for `<body>`.
    let positioner = use_element();

    // Escape with focus fallen to `<body>`, as `Modal` hears it (todo 1304).
    let stray_escape = use_signal(|| 0u64);
    use_hook(move || {
        Rc::new(keyboard().map(|api| {
            api.on_key_unfiltered(std::boxed::Box::new(move |chord: KeyChord| {
                let outside = positioner
                    .try_mounted()
                    .is_some_and(|mounted| !chord.within(&mounted, positioner.tag()));
                if chord.key != Key::Escape || chord.repeat || !outside || !layer.is_top() {
                    return false;
                }
                bump(stray_escape);
                true
            }))
        }))
    });
    let seen = use_hook(|| Rc::new(Cell::new(0u64)));
    use_effect(move || {
        let tick = stray_escape();
        if tick != seen.replace(tick) {
            act.call(Move::Close);
        }
    });

    // An interactive step's target sits in the Tab order beside the card: Tab off its
    // edge goes back in. `true` only once focus moved, so the press moves no further.
    let interactive = step.interactive && step.target.is_some();
    let mut bridged = use_hook(|| CopyValue::new(None::<ElementHandle>));
    bridged.set(step.target.filter(|_| interactive));
    use_hook(move || {
        Rc::new(keyboard().map(|api| {
            api.on_key_unfiltered(std::boxed::Box::new(move |chord: KeyChord| {
                let target = bridged.try_peek().ok().and_then(|target| *target);
                let Some((target, mounted)) =
                    target.and_then(|target| Some((target, target.try_mounted()?)))
                else {
                    return false;
                };
                if chord.key != Key::Tab || !chord.within(&mounted, target.tag()) {
                    return false;
                }
                let backwards = chord.modifiers.shift();
                let leaving = (backwards && target.is_focused())
                    || edge_focusable(&target, !backwards).is_none_or(|item| item.is_focused());
                leaving
                    && edge_focusable(&positioner, backwards)
                        .is_some_and(|item| item.focus().is_ok() && item.is_focused())
            }))
        }))
    });

    // A selector's target is looked up as its step shows, before the rect effect measures.
    let selector = step.target_selector.clone();
    let mirror = step.target;
    use_effect(use_reactive!(|index, selector| {
        let _ = index;
        if let (Some(selector), Some(mirror)) = (&selector, mirror) {
            mirror.point_at(mounted_by_selector(selector));
        }
    }));

    let rect = use_element_rect(step.target, true);
    let padding = step.padding.unwrap_or(defaults.padding);
    // Still 0x0 after the hook's laid-out tries: mounted but not rendered (`display: none`).
    let measured = match rect {
        ElementRect::At { rect, viewport } if rect.width > 0.0 || rect.height > 0.0 => {
            Some((rect, viewport))
        }
        _ => None,
    };
    let hole = step
        .target
        .and(measured)
        .map(|(rect, viewport)| hole_rect(rect, padding, viewport));
    // While a new target is measured the hole stays put, and glides on from there.
    let last_hole = use_hook(|| Rc::new(Cell::new(None::<Rect>)));
    let hole = match rect {
        ElementRect::Pending if step.target.is_some() => last_hole.get(),
        _ => hole,
    };
    last_hole.set(hole);
    // No target, or one that never mounted or renders nothing: the card waits in the middle.
    let missing = step.target.is_some()
        && match rect {
            ElementRect::Missing => true,
            ElementRect::At { .. } => measured.is_none(),
            ElementRect::Pending => false,
        };
    let centred = step.target.is_none() || missing;
    let key = step.key.clone();
    let looked_up = step.target_selector.clone();
    use_effect(use_reactive!(|missing, key, looked_up| {
        if !missing {
            return;
        }
        match looked_up {
            Some(selector) => warn(&format!(
                "use_tour: step \"{key}\"'s target_selector \"{selector}\" matches nothing rendered \
                 (or the renderer is a WebView, which cannot look it up), so its card shows in the middle."
            )),
            None => warn(&format!(
                "use_tour: step \"{key}\"'s target is not mounted or not rendered, so its card shows in the middle."
            )),
        }
    }));

    // Set on a step change until the hole's glide ends; the fallback serves a renderer without `transitionend`.
    let mut latest_index = use_hook(|| CopyValue::new(index));
    latest_index.set(index);
    let mut arrived = use_signal(|| index);
    let land = use_scheduled(move |_| {
        let index = *latest_index.peek();
        if *arrived.peek() != index {
            arrived.set(index);
        }
    });
    use_effect(use_reactive!(|index| {
        let _ = index;
        land.after(HOLE_GLIDE_FALLBACK_MS);
    }));
    let moving = arrived() != index;

    let mask_click = options.mask_click;
    let style = highlight_style(hole, step.radius.unwrap_or(defaults.radius));

    let onkeydown = use_callback(move |event: Event<KeyboardData>| {
        if escape_closes(&event) && layer.is_top() {
            act.call(Move::Close);
            return;
        }
        if event.key() == Key::Tab {
            // Off the card's edge to the target, before the trap cycles.
            let backwards = event.modifiers().shift();
            if let Some(target) = *bridged.peek()
                && !key_taken(&event)
                && edge_focusable(&positioner, !backwards).is_some_and(|item| item.is_focused())
                && enter_target(&target, backwards)
            {
                event.prevent_default();
            }
            return;
        }
        if key_taken(&event)
            || typing_target(&event)
            || arrow_target(&event)
            || has_shortcut_modifier(&event)
        {
            return;
        }
        // A held arrow steps once, and none finishes: the last step stops it (2262).
        match logical_key(&event) {
            Key::ArrowRight if !event.is_auto_repeating() => act.call(Move::Forward),
            Key::ArrowLeft if !event.is_auto_repeating() => act.call(Move::Prev),
            _ => {}
        }
    });
    // The trap is the root: only the card holds anything focusable.
    let layer_sx = parts_under_sx(&options.parts, (&TOUR_SX).into());
    // Keyed per step, so each step's popover hooks onto its own target. A target that
    // turns out missing mounts a fresh box: the popover's inline placement stays on the old.
    let card = rsx! {
        TourCard {
            key: "{index}-{step.key}-{centred}",
            act,
            options: options.clone(),
            step: step.clone(),
            index,
            centred,
            positioner,
            onkeydown,
            gap: theme.popover.gap + padding,
            edge: theme.popover.padding,
            max_width: defaults.max_width,
            remeasure: remeasure_key(hole),
        }
    };

    // An interactive step's mask goes around the hole, once both are measured.
    let strips = measured
        .zip(hole)
        .filter(|(_, hole)| interactive && has_size(*hole))
        .map(|((_, viewport), hole)| mask_strips(hole, viewport));
    // Keyed by the shape too: a full mask is a fresh node, never a strip with its style removed.
    let split = strips.is_some();
    let masks: Vec<Option<String>> = match strips {
        Some(strips) => strips.map(Some).into(),
        None => vec![None],
    };

    rsx! {
        FocusTrap { sx: layer_sx, "data-lsx-tour": "true",
            for (index, style) in masks.into_iter().enumerate() {
                div {
                    key: "{split}-{index}",
                    "data-slot": TourPart::Mask.slot(),
                    style,
                    // Keeps focus in the card.
                    onmousedown: move |event| event.prevent_default(),
                    onclick: move |_| match mask_click {
                        MaskClick::None => {}
                        MaskClick::Close => act.call(Move::Close),
                        MaskClick::Next => act.call(Move::Next),
                    },
                }
            }
            div {
                "data-slot": TourPart::Highlight.slot(),
                "data-ringed": hole.is_some_and(has_size).then_some("true"),
                "data-moving": moving.then_some("true"),
                ontransitionend: move |_| {
                    if *arrived.peek() != index {
                        arrived.set(index);
                    }
                },
                style,
            }
            {card}
        }
    }
}

/// Focuses the card inside `positioner`; a WebView finds it by the positioner's tag.
fn focus_card(positioner: &ElementHandle) {
    match positioner.query_selector("[data-autofocus]") {
        Ok(card) => {
            let _ = card.focus();
        }
        Err(_) => {
            if let Some(tag) = positioner.tag() {
                let _ = focus_first_of(&[format!("[{OBSERVE_ATTR}='{tag}'] [data-autofocus]")]);
            }
        }
    }
}

/// `root`'s first or last focusable; `None` without one or where the renderer cannot query.
fn edge_focusable(root: &ElementHandle, last: bool) -> Option<std::boxed::Box<dyn ElementApi>> {
    let items = root.query_selector_all(FOCUSABLE_SELECTOR).ok()?;
    match last {
        true => items.into_iter().last(),
        false => items.into_iter().next(),
    }
}

/// Focuses the target's first control (last, `backwards`), or the target itself; whether it took.
fn enter_target(target: &ElementHandle, backwards: bool) -> bool {
    match edge_focusable(target, backwards) {
        Some(item) => item.focus().is_ok() && item.is_focused(),
        None => target.focus().is_ok() && target.is_focused(),
    }
}

/// One step's card, drawn afresh per step: its popover anchors to that step's target.
#[component]
fn TourCard(
    act: Callback<Move>,
    options: TourOptions,
    step: TourStep,
    index: usize,
    centred: bool,
    positioner: ElementHandle,
    onkeydown: Callback<Event<KeyboardData>>,
    gap: f64,
    edge: f64,
    max_width: f64,
    remeasure: u64,
) -> Element {
    let labels = use_localization().tour;
    let fallback = use_element();
    let anchor = step.target.unwrap_or(fallback);
    // The positioner wraps the card exactly, so it is what the popover measures.
    let popover = use_popover_on(
        anchor,
        positioner,
        !centred,
        PopoverOptions::new(gap, edge)
            .side(step.side)
            .align(step.align)
            .remeasure(remeasure),
    );
    let body_id = use_id();
    let progress_id = use_id();
    let summary_id = use_id();
    // The target outside the card is reachable on an interactive step.
    let modal = match step.interactive && !centred {
        true => "false",
        false => "true",
    };

    // Brought into view once per step, through nested and sideways scrollers; the
    // scroll it causes measures the hole again.
    let scrolled = use_hook(|| Rc::new(Cell::new(false)));
    use_effect(move || {
        if anchor.mount_token().is_some()
            && step.target.is_some()
            && !scrolled.replace(true)
            && let Some(mounted) = anchor.mounted()
        {
            let _ = scroll_chain_into_view(&mounted, !prefers_reduced_motion());
        }
    });
    // Focus moves once the card shows: a hidden box takes no focus. Per mount:
    // the handle is the layer's, and may still hold the last step's positioner.
    let ready = centred || popover.placed();
    let focused = use_hook(|| Rc::new(Cell::new(None::<usize>)));
    let targeted = step.target.is_some();
    use_effect(use_reactive!(|ready| {
        let token = positioner.mount_token();
        if ready && token.is_some() && focused.get() != token {
            focused.set(token);
            focus_card(&positioner);
            // Chromium 113's focus stops a running smooth scroll midway (todo 2356): once more after it.
            if targeted && let Some(mounted) = anchor.mounted() {
                let _ = scroll_chain_into_view(&mounted, !prefers_reduced_motion());
            }
        }
    }));

    let total = options.steps.len();
    let first = index == 0;
    let last = index + 1 >= total;
    let progress = fill(labels.progress, &[("n", &(index + 1)), ("m", &total)]);
    let name = options
        .aria_label
        .clone()
        .or_else(|| step.title.is_none().then(|| labels.label.to_string()));
    // Next's key first: under RTL that is ArrowLeft, as `logical_key` reads it.
    let rtl = try_use_context::<LiberoContext>()
        .is_some_and(|context| *context.direction.read() == Direction::Rtl);
    let shortcuts = match rtl {
        true => "ArrowLeft ArrowRight",
        false => "ArrowRight ArrowLeft",
    };

    let positioned = use_box()
        .framework_sx(if centred { &CENTRED_SX } else { &PLACED_SX })
        .style((!centred).then(|| popover.style()))
        .prepare();
    let card_sx = sx()
        .width(format!("{max_width}px"))
        .and(CARD_SX.clone())
        .and(options.sx.as_ref().cloned().unwrap_or_default());

    let content = match options.card {
        Some(render) => {
            // The card is the author's: the tour describes it by its own hidden text (2264).
            let summary = match &step.description {
                Some(text) => format!("{progress}. {text}"),
                None => progress.clone(),
            };
            let view = TourView {
                index,
                total,
                step: step.clone(),
                progress,
                act,
            };
            let name = name.clone().or_else(|| step.title.clone());
            rsx! {
                layout::Box {
                    "data-slot": TourPart::Card.slot(),
                    role: "dialog",
                    "aria-modal": modal,
                    "aria-label": name,
                    "aria-describedby": summary_id(),
                    "aria-keyshortcuts": shortcuts,
                    tabindex: "-1",
                    "data-autofocus": "true",
                    sx: card_sx,
                    {render.call(view)}
                    div { id: "{summary_id}", hidden: true, "{summary}" }
                }
            }
        }
        None => {
            let body = step
                .content
                .clone()
                .or_else(|| step.description.clone().map(|text| rsx! { "{text}" }));
            // The step's text, then where the tour stands.
            let described = match body.is_some() {
                true => format!("{} {}", body_id(), progress_id()),
                false => progress_id(),
            };
            rsx! {
                Dialog {
                    "data-slot": TourPart::Card.slot(),
                    title: step.title.clone(),
                    aria_label: name,
                    close_label: labels.close,
                    onclose: move |_| act.call(Move::Close),
                    "aria-modal": modal,
                    "aria-describedby": described,
                    "aria-keyshortcuts": shortcuts,
                    tabindex: "-1",
                    "data-autofocus": "true",
                    sx: card_sx,
                    if let Some(body) = body {
                        div { id: body_id(), "data-slot": TourPart::Body.slot(), {body} }
                    }
                    div { "data-slot": TourPart::Footer.slot(),
                        span { id: "{progress_id}", "data-slot": TourPart::Progress.slot(), "{progress}" }
                        if !last {
                            Button {
                                "data-slot": TourPart::Skip.slot(),
                                variant: "standard",
                                size: "sm",
                                onclick: move |_| act.call(Move::Close),
                                "{labels.skip}"
                            }
                        }
                        if !first {
                            Button {
                                "data-slot": TourPart::Previous.slot(),
                                variant: "outlined",
                                size: "sm",
                                onclick: move |_| act.call(Move::Prev),
                                "{labels.previous}"
                            }
                        }
                        Button {
                            "data-slot": TourPart::Next.slot(),
                            size: "sm",
                            onclick: move |_| act.call(Move::Next),
                            if last { "{labels.done}" } else { "{labels.next}" }
                        }
                    }
                }
            }
        }
    };

    let mut attributes = positioner.attributes();
    attributes.push(attr("data-slot", TourPart::Positioner.slot()));
    positioned
        .element(&positioner)
        .event("onkeydown", move |event: Event<KeyboardData>| {
            onkeydown.call(event)
        })
        .render(HtmlTag::Div, attributes, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let card = "& > [data-slot='positioner'] > [data-slot='card']";
        let table: Vec<(&str, String)> = part_table::<TourPart>()
            .into_iter()
            .map(|(slot, selector)| (slot, selector.to_string()))
            .collect();
        assert_eq!(
            table,
            [
                ("mask", "& > [data-slot='mask']".to_string()),
                ("highlight", "& > [data-slot='highlight']".to_string()),
                ("positioner", "& > [data-slot='positioner']".to_string()),
                ("card", card.to_string()),
                ("header", format!("{card} > [data-slot='header']")),
                (
                    "title",
                    format!("{card} > [data-slot='header'] > [data-slot='title']")
                ),
                (
                    "close",
                    format!("{card} > [data-slot='header'] > [data-slot='close']")
                ),
                ("body", format!("{card} > [data-slot='body']")),
                ("footer", format!("{card} > [data-slot='footer']")),
                (
                    "progress",
                    format!("{card} > [data-slot='footer'] > [data-slot='progress']")
                ),
                (
                    "skip",
                    format!("{card} > [data-slot='footer'] > [data-slot='skip']")
                ),
                (
                    "previous",
                    format!("{card} > [data-slot='footer'] > [data-slot='previous']")
                ),
                (
                    "next",
                    format!("{card} > [data-slot='footer'] > [data-slot='next']")
                ),
            ]
        );
    }
}
