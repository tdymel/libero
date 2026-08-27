use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States,
        common::focus_ring_sx,
        layout::{BoxStyle, use_box},
    },
    sx::{StaticSx, Sx, sx},
    theme::FieldDefaults,
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
        .border_color("grey.5")
        .background("white")
        .color("black")
        .focus_within(sx().border_color("primary.6"))
        // The ring is the frame's, because focus lands on a child. There is no
        // `:focus-visible-within`, so `:has` is what keeps a mouse click from
        // drawing one.
        .has_focus_visible(focus_ring_sx())
        .when("error", sx().border_color("error.7"))
        .when("warning", sx().border_color("warning.7"))
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .background("grey.1"),
        )
        .selector(
            "& > [data-slot]",
            sx().display("flex")
                .align_items("center")
                .flex("0 0 auto")
                .color("grey.6"),
        )
});

/// The native control inside the frame, stripped of the chrome that is now the
/// frame's. Every framed field renders its element with this as its
/// `framework_sx`.
pub(crate) static FIELD_CONTROL_SX: StaticSx = StaticSx::new(field_control_sx);

/// The same declarations, for a control that adds one of its own - `Select`'s
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
        .selector("::placeholder", sx().color("grey.6"))
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

/// The resolved frame. Pure - no hooks.
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

        self.frame.render(HtmlTag::Div, Vec::new(), children)
    }
}

fn slot(slot: &'static str, content: &Element) -> Element {
    let content = content.clone();
    rsx! {
        span { "data-slot": slot, {content} }
    }
}
