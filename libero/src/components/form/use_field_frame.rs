use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, focus_ring_sx, ring_overlay, ring_overlay_sx},
        layout::{BoxStyle, use_box},
    },
    platform::{
        PLACEHOLDER_ATTR, PLACEHOLDER_SHOWN_ATTR, draws_placeholders, padding_press,
        placeholder_drawn,
    },
    sx::{StaticSx, Sx, sx},
    theme::{FIELD_FRAME_GAP, FieldDefaults, PaperDefaults, Size, SizeCss},
};

/// The bordered box a control sits in. Shared by every framed field, so the
/// border, the background, the per-size padding, the status colours and the
/// focus treatment are written once.
static FIELD_FRAME_SX: StaticSx = StaticSx::new(|| {
    FieldDefaults::frame_theme_vars()
        .display("flex")
        .align_items("center")
        .width("100%")
        .border_style("solid")
        .border_width("1px")
        // 3:1 on the page and on a dark `Paper` (WCAG 1.4.11, todo 490).
        .border_color("muted.6")
        // The surface's own colour rather than a control token of its own: a
        // field sits on a surface and matches it until a theme says otherwise.
        .and(PaperDefaults::background_sx())
        .color("ink")
        .focus_within(sx().border_color("primary.6"))
        // The control's ring is the frame's: the control is followed by a ring
        // overlay, and the frame is the overlay's containing block. A button in
        // a slot rings itself (todo 410). Keyed off `:focus-visible`, not
        // `:focus-within`, so a click on a trigger draws no ring.
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
                // A slot holds text as often as an icon - a unit, a prefix,
                // `PhoneField`'s country code, which inherits from here - so
                // it is dimmed text, not a grey (todo 240).
                .color("text-dimmed"),
        )
        // Where the renderer draws no placeholder: no width and a margin
        // taking back the gap, so the control keeps its place, and the text
        // overflows over the control's own.
        .selector(
            format!("& > [{PLACEHOLDER_ATTR}]"),
            sx().flex("0 0 0")
                .width("0")
                .margin_inline_end(format!("calc(-1 * {})", FIELD_FRAME_GAP.value()))
                .align_self("center")
                .white_space("nowrap")
                .line_height("1.5")
                .pointer_events("none")
                .color("text-dimmed")
                .visibility("hidden"),
        )
        .selector(
            format!("& > [{PLACEHOLDER_SHOWN_ATTR}]"),
            sx().visibility("visible"),
        )
        // A `Textarea`'s first line is at its top, not the frame's middle.
        .selector(
            format!("& > [{PLACEHOLDER_ATTR}][data-multiline]"),
            sx().align_self("flex-start"),
        )
});

/// A press on the frame's padding is a press on its control (todo 462).
const FRAME: &str = "[data-frame]";

/// The `ActionIcon` step for a slot button (stepper, reveal, clear, eye
/// dropper) in a field of `size`. `ActionIcon`'s scale (16, 20, 24, 32, 40,
/// 48px) climbs faster than a field's content box (18, 20, 22, 24, 26, 28px):
/// at `md` an `md` icon is 24px in a 22px box and would grow the frame 2px
/// (todo 495). Two field steps per icon step fits every step.
pub(crate) const fn slot_icon_size(size: Size) -> Size {
    match size {
        Size::Xs | Size::Sm => Size::Xs,
        Size::Md | Size::Lg => Size::Sm,
        Size::Xl | Size::Xxl => Size::Md,
    }
}

/// The native control inside the frame, stripped of the chrome that is now the
/// frame's. Every framed field renders its element with this as its
/// `framework_sx`.
pub(crate) static FIELD_CONTROL_SX: StaticSx = StaticSx::new(field_control_sx);

/// The same declarations, for a control that adds one of its own - `NativeSelect`'s
/// pointer cursor. `StaticSx` takes a single builder, so a field extending the
/// control's look calls this rather than a second `framework_sx`.
pub(crate) fn field_control_sx() -> Sx {
    sx().flex("1 1 auto")
        // Without it a long value pushes the frame wider instead of scrolling.
        .min_width("0")
        .border("none")
        .outline("none")
        .background("transparent")
        .padding("0")
        .color("inherit")
        // An `<input>` inherits none of these, so each would fall back to the
        // UA's.
        .font_family("inherit")
        .font_size("inherit")
        // The frame's padding sets the height now, so the control contributes
        // exactly one line box and a `Textarea` can contribute several.
        .line_height("1.5")
        .selector("::placeholder", sx().color("text-dimmed"))
}

/// The frame around a field's control, with room either side of it.
///
/// Separate from `use_field` rather than folded into it: `Checkbox`, `Radio`,
/// `RadioGroup` and `RangeSlider` have no frame at all, and folding it in would
/// make them prepare a `use_box` they never render.
///
/// `use_` because [`FieldFrameBuilder::prepare`] calls [`use_box`]. Build it in
/// the component body and return it - never inside an `if`, a `match` arm, or
/// after an early `return`.
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
    placeholder: Option<&'a str>,
    multiline: bool,
}

impl<'a> FieldFrameBuilder<'a> {
    /// The control's `placeholder`, drawn by the frame where the renderer
    /// draws none (Blitz). Pass what the control's attribute holds.
    #[inline]
    pub fn placeholder(mut self, placeholder: Option<&'a str>) -> Self {
        self.placeholder = placeholder;
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
                .filter(|text| !text.is_empty() && !draws_placeholders())
                .map(|text| placeholder(text, self.multiline)),
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
    placeholder: Option<Element>,
    trailing: Option<Element>,
}

impl PreparedFrame {
    pub fn render(self, control: Element) -> Element {
        let mut children = Vec::with_capacity(5);
        children.extend(self.leading);
        // Right before the control: the renderer finds it as the next element.
        children.extend(self.placeholder);
        children.push(control);
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

/// Hidden until the renderer marks its control empty; the control's own
/// `placeholder` still names it to assistive tech.
fn placeholder(text: &str, multiline: bool) -> Element {
    rsx! {
        span {
            // `PLACEHOLDER_ATTR`; `rsx!` takes only a literal name.
            "data-lsx-placeholder": true,
            "data-multiline": multiline.then_some(true),
            "aria-hidden": "true",
            onmounted: |_| placeholder_drawn(),
            "{text}"
        }
    }
}

/// No ring overlay in a slot: a button there draws its own ring, around
/// itself rather than the whole frame.
fn slot(slot: &'static str, id: Option<String>, content: &Element) -> Element {
    let content = content.clone();
    rsx! {
        span { "data-slot": slot, id, {content} }
    }
}
