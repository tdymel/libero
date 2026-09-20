use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, focus_ring_sx, ring_overlay, ring_overlay_sx},
        layout::{BoxStyle, use_box},
    },
    platform::{
        PLACEHOLDER_ATTR, PLACEHOLDER_CELL_ATTR, PLACEHOLDER_SHOWN_ATTR, draws_placeholders,
        padding_press, placeholder_drawn,
    },
    sx::{StaticSx, Sx, sx},
    theme::{FieldDefaults, PaperDefaults, Size, SizeCss},
};

/// The bordered box every framed field's control sits in.
static FIELD_FRAME_SX: StaticSx = StaticSx::new(|| {
    FieldDefaults::frame_theme_vars()
        .display("flex")
        .align_items("center")
        .width("100%")
        .border_style("solid")
        .border_width("1px")
        // 3:1 on the page and on a dark `Paper` (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        // Matches the surface it sits on.
        .and(PaperDefaults::background_sx())
        .color("ink")
        .focus_within(sx().border_color("primary.6"))
        // The control's ring overlay covers the frame; a slot button rings itself
        // (todo 410). `:focus-visible`, so a click on a trigger draws no ring.
        .position("relative")
        .selector(
            "& [data-ring]",
            ring_overlay_sx()
                // Out by the border, as an outline on the frame would sit.
                .inset("-1px"),
        )
        .per_radius(|radius| {
            sx().selector(
                "& [data-ring]",
                sx().border_radius(SizeCss::RADIUS.value(radius)),
            )
        })
        .selector("& :focus-visible ~ [data-ring]", focus_ring_sx())
        .when("error", sx().border_color("error.7"))
        .when("warning", sx().border_color("warning.7"))
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .background("muted.1"),
        )
        .selector(
            "& > [data-slot]",
            sx().display("flex")
                .align_items("center")
                .flex("0 0 auto")
                // A slot holds text as often as an icon: dimmed text, not a grey (todo 240).
                .color("text-dimmed"),
        )
        // A placeholder cell takes the control's place.
        .selector(
            format!("& > [{PLACEHOLDER_CELL_ATTR}]"),
            sx().flex("1 1 auto").min_width("0"),
        )
        // The control is no sibling of the ring then, and Blitz, the only
        // renderer with a cell, rings a focused text control always.
        .selector(
            format!("& > [{PLACEHOLDER_CELL_ATTR}]:focus-within ~ [data-ring]"),
            focus_ring_sx(),
        )
        // Blitz places an overlay by its parent box, not the frame.
        .selector(
            format!("& > [{PLACEHOLDER_CELL_ATTR}] [data-ring]"),
            sx().display("none"),
        )
});

/// Where the renderer draws no placeholder, the control and a drawn one share
/// one grid cell.
static PLACEHOLDER_CELL_SX: StaticSx = StaticSx::new(|| {
    sx().display("grid")
        .grid_template_columns("minmax(0, 1fr)")
        .align_items("center")
        .selector("& > *", sx().grid_area("1 / 1").min_width("0"))
        .selector(
            format!("& > [{PLACEHOLDER_ATTR}]"),
            sx().overflow("hidden")
                .white_space("nowrap")
                .line_height("1.5")
                .pointer_events("none")
                .color("text-dimmed")
                .visibility("hidden")
                // Blitz lays inline text out from the left whatever the
                // direction; the web's placeholder starts at the right.
                .rtl(sx().text_align("right")),
        )
        .selector(
            format!("& > [{PLACEHOLDER_SHOWN_ATTR}]"),
            sx().visibility("visible"),
        )
        // A `Textarea`'s first line is at its top, and its placeholder wraps.
        .selector(
            format!("& > [{PLACEHOLDER_ATTR}][data-multiline]"),
            sx().align_self("start")
                .white_space("pre-wrap")
                .overflow_wrap("break-word"),
        )
});

/// A press on the frame's padding is a press on its control (todo 462).
const FRAME: &str = "[data-frame]";

/// The `ActionIcon` size for a slot button in a field of `size`: two field
/// steps per icon step, or the icon grows the frame (todo 495).
pub(crate) const fn slot_icon_size(size: Size) -> Size {
    match size {
        Size::Xs | Size::Sm => Size::Xs,
        Size::Md | Size::Lg => Size::Sm,
        Size::Xl | Size::Xxl => Size::Md,
    }
}

/// The native control inside the frame, stripped of the frame's chrome.
pub(crate) static FIELD_CONTROL_SX: StaticSx = StaticSx::new(field_control_sx);

/// The same declarations, for a control that extends them.
pub(crate) fn field_control_sx() -> Sx {
    sx().flex("1 1 auto")
        // Without it a long value pushes the frame wider instead of scrolling.
        .min_width("0")
        .border("none")
        .outline("none")
        .background("transparent")
        .padding("0")
        .color("inherit")
        // An `<input>` inherits neither.
        .font_family("inherit")
        .font_size("inherit")
        // The frame's padding sets the height; the control adds line boxes only.
        .line_height("1.5")
        .selector("::placeholder", sx().color("text-dimmed"))
}

/// The frame around a field's control, with slots either side. A hook:
/// `prepare` calls [`use_box`], so never build it conditionally.
pub(crate) fn use_field_frame<'a>() -> FieldFrameBuilder<'a> {
    FieldFrameBuilder::default()
}

#[derive(Default)]
pub(crate) struct FieldFrameBuilder<'a> {
    leading: Option<&'a Element>,
    trailing: Option<&'a Element>,
    states: Option<&'a Input<States>>,
    ondrop: Option<Rc<dyn Fn(DragEvent)>>,
    ids: [Option<String>; 2],
    placeholder: Option<Option<&'a str>>,
    multiline: bool,
}

impl<'a> FieldFrameBuilder<'a> {
    /// The control's `placeholder`, drawn where the renderer draws none (Blitz).
    /// Pass `None` too, so the control's place in the tree stays put.
    #[inline]
    pub fn placeholder(mut self, placeholder: Option<&'a str>) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// The control is a `Textarea`: the drawn placeholder sits at its top.
    #[inline]
    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    /// Makes the whole frame a drop target: a drop the control did not take
    /// (on the padding, in a slot) reaches `ondrop`. Opt-in; only `FileField`.
    #[inline]
    pub fn ondrop(mut self, ondrop: Option<impl Fn(DragEvent) + 'static>) -> Self {
        self.ondrop = ondrop.map(|ondrop| Rc::new(ondrop) as Rc<dyn Fn(DragEvent)>);
        self
    }

    /// Rendered before the control - an icon, a currency prefix, the chips a
    /// `MultiSelect` has already picked.
    #[inline]
    pub fn leading(mut self, leading: &'a Option<Element>) -> Self {
        self.leading = leading.as_ref();
        self
    }

    /// Rendered after the control - a chevron, a reveal toggle, a stepper.
    #[inline]
    pub fn trailing(mut self, trailing: &'a Option<Element>) -> Self {
        self.trailing = trailing.as_ref();
        self
    }

    /// The field's own states, so the frame carries the size, radius and
    /// status the captions already do.
    #[inline]
    pub fn states(mut self, states: &'a Input<States>) -> Self {
        self.states = Some(states);
        self
    }

    /// The slots' ids - [`PreparedField::slot_ids`](super::PreparedField::slot_ids) -
    /// so the control's `aria-describedby` finds a text slot.
    #[inline]
    pub fn ids(mut self, ids: [Option<String>; 2]) -> Self {
        self.ids = ids;
        self
    }

    /// Resolves the frame. **This is the hook** - see [`use_field_frame`].
    pub fn prepare(self) -> PreparedFrame {
        // Whether the press under way began on the padding. A press a slot's
        // own handler took (the multi-select chevron) must not be forwarded.
        let pending = use_hook(|| Rc::new(Cell::new(false)));
        let mut frame = use_box().framework_sx(&FIELD_FRAME_SX);
        if let Some(states) = self.states {
            frame = frame.states(states);
        }
        let mut frame = frame
            .prepare()
            .attr("data-frame", true)
            .event("onmousedown", {
                let pending = pending.clone();
                move |event: MouseEvent| {
                    pending.set(false);
                    if !event.default_action_enabled() {
                        return;
                    }
                    if let Some(control) = padding_press(&event, FRAME) {
                        // Keeps the focus off the body and starts no selection.
                        event.prevent_default();
                        pending.set(true);
                        // Outside the dispatch: dioxus drops a re-entrant listener.
                        spawn(async move {
                            let _ = control.focus();
                        });
                    }
                }
            })
            .event("onclick", move |event: MouseEvent| {
                if pending.replace(false)
                    && let Some(control) = padding_press(&event, FRAME)
                {
                    spawn(async move {
                        let _ = control.click();
                    });
                }
            });
        if let Some(ondrop) = self.ondrop {
            // A drag the control already took is its own, as with the press.
            frame = frame
                .event("ondragover", |event: DragEvent| {
                    if event.default_action_enabled() {
                        event.prevent_default();
                    }
                })
                .event("ondrop", move |event: DragEvent| {
                    if event.default_action_enabled() {
                        ondrop(event);
                    }
                });
        }

        let [leading_id, trailing_id] = self.ids;
        PreparedFrame {
            frame,
            leading: self
                .leading
                .map(|leading| slot("leading", leading_id, leading)),
            placeholder: self
                .placeholder
                .filter(|_| !draws_placeholders())
                .map(|text| text.map(str::to_string)),
            multiline: self.multiline,
            trailing: self
                .trailing
                .map(|trailing| slot("trailing", trailing_id, trailing)),
        }
    }
}

/// The resolved frame. Pure - no hooks, so one prepared frame can be cloned
/// and rendered once per cell - see `PinField`.
#[derive(Clone)]
pub(crate) struct PreparedFrame {
    frame: BoxStyle,
    leading: Option<Element>,
    /// `Some` where the frame draws the placeholder: its text, if any.
    placeholder: Option<Option<String>>,
    multiline: bool,
    trailing: Option<Element>,
}

impl PreparedFrame {
    pub fn render(self, control: Element) -> Element {
        let mut children = Vec::with_capacity(4);
        children.extend(self.leading);
        children.push(match self.placeholder {
            Some(text) => rsx! {
                PlaceholderCell {
                    text,
                    multiline: self.multiline,
                    inset: "0",
                    control,
                }
            },
            None => control,
        });
        children.extend(self.trailing);
        children.push(ring_overlay());

        self.frame.render(HtmlTag::Div, Vec::new(), children)
    }
}

/// A field's control in a scope of its own. The field's shell skips a value
/// change; `draw` reads the value's signal, so only this scope redraws.
#[derive(Props, Clone)]
pub(crate) struct LiveControlProps {
    draw: Rc<dyn Fn() -> Element>,
}

/// A new `draw` is a shell render, which always redraws the control.
impl PartialEq for LiveControlProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.draw, &other.draw)
    }
}

#[component]
pub(crate) fn LiveControl(props: LiveControlProps) -> Element {
    (props.draw)()
}

/// A caller's slot content in a signal, so the field's shell skips the new
/// `Element` every caller render brings. `None` while the slot is empty.
pub(crate) fn use_live_slot(content: Option<Element>) -> Option<Signal<Option<Element>>> {
    let mut slot = use_signal(|| None::<Element>);
    let filled = content.is_some();
    if filled || slot.peek().is_some() {
        slot.set(content);
    }
    filled.then_some(slot)
}

/// Draws a [`use_live_slot`] slot, alone on a caller render.
#[component]
pub(crate) fn LiveSlot(content: Signal<Option<Element>>) -> Element {
    content.cloned().unwrap_or_else(|| rsx! {})
}

/// `control` with its `placeholder` drawn where the renderer draws none
/// (Blitz); `inset` is the control's padding and border, as a `margin`.
pub(crate) fn with_drawn_placeholder(
    text: Option<&str>,
    inset: &'static str,
    control: Element,
) -> Element {
    if draws_placeholders() {
        return control;
    }
    rsx! {
        PlaceholderCell {
            text: text.map(str::to_string),
            multiline: false,
            inset,
            control,
        }
    }
}

/// The control and its drawn placeholder, shown once the renderer marks the
/// control empty.
#[component]
fn PlaceholderCell(
    text: Option<String>,
    multiline: bool,
    inset: &'static str,
    control: Element,
) -> Element {
    let cell = use_box().framework_sx(&PLACEHOLDER_CELL_SX).prepare();
    let text = text.filter(|text| !text.is_empty());
    cell.attr(PLACEHOLDER_CELL_ATTR, true).render(
        HtmlTag::Div,
        Vec::new(),
        rsx! {
            if let Some(text) = text {
                // Right before the control: the renderer finds it as the next element.
                span {
                    // `PLACEHOLDER_ATTR`; `rsx!` takes only a literal name.
                    "data-lsx-placeholder": true,
                    "data-multiline": multiline.then_some(true),
                    "aria-hidden": "true",
                    margin: inset,
                    onmounted: |_| placeholder_drawn(),
                    "{text}"
                }
            }
            {control}
        },
    )
}

/// No ring overlay in a slot: a button there draws its own ring, around
/// itself rather than the whole frame.
fn slot(slot: &'static str, id: Option<String>, content: &Element) -> Element {
    let content = content.clone();
    rsx! {
        span { "data-slot": slot, id, {content} }
    }
}
