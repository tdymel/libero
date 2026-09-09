use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::{focus_ring_sx, ring_overlay, ring_overlay_sx},
        layout::{BoxStyle, use_box},
    },
    sx::{StaticSx, Sx, sx},
    theme::{FieldDefaults, PaperDefaults, SizeCss},
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
        .border_color("muted.5")
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
});

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
}

impl<'a> FieldFrameBuilder<'a> {
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

    /// Resolves the frame. **This is the hook** - see [`use_field_frame`].
    pub fn prepare(self) -> PreparedFrame {
        let mut frame = use_box().framework_sx(&FIELD_FRAME_SX);
        if let Some(states) = self.states {
            frame = frame.states(states);
        }

        PreparedFrame {
            frame: frame.prepare(),
            leading: self.leading.map(|leading| slot("leading", leading)),
            trailing: self.trailing.map(|trailing| slot("trailing", trailing)),
        }
    }
}

/// The resolved frame. Pure - no hooks, so one prepared frame can be cloned
/// and rendered once per cell - see `PinField`.
#[derive(Clone)]
pub(crate) struct PreparedFrame {
    frame: BoxStyle,
    leading: Option<Element>,
    trailing: Option<Element>,
}

impl PreparedFrame {
    pub fn render(self, control: Element) -> Element {
        let mut children = Vec::with_capacity(3);
        children.extend(self.leading);
        children.push(control);
        children.extend(self.trailing);
        children.push(ring_overlay());

        self.frame.render(HtmlTag::Div, Vec::new(), children)
    }
}

/// No ring overlay in a slot: a button there draws its own ring, around
/// itself rather than the whole frame.
fn slot(slot: &'static str, content: &Element) -> Element {
    let content = content.clone();
    rsx! {
        span { "data-slot": slot, {content} }
    }
}
