use dioxus::prelude::*;
use libero::{
    components::{
        Box, CodeBlock, Flex, Input, NativeSelect, OptionLabel, SegmentedControl, Slider,
        SliderChangeEvent, Switch, Text,
    },
    sx::sx,
    theme::{CODE_BLOCK_BORDER, Size, TEXT_FONT_SIZE},
};
use std::sync::LazyLock;

use super::{Control, ControlKind, color::ColorControl};

/// The card is the query container the control panel keys off.
const DEMO_CARD: &str = "demo-card";

/// The color belongs *inside* the shorthand: a later `border-left: 1px solid`
/// would otherwise reset the color to `currentColor`. A `&'static str`, not a
/// local `String`, so the swatch closure can capture it without moving it out
/// of the `FnMut` it lives in.
static BORDER: LazyLock<String> =
    LazyLock::new(|| format!("1px solid {}", CODE_BLOCK_BORDER.value()));

pub(super) fn border() -> &'static str {
    BORDER.as_str()
}

/// `"line_numbers"` -> `"Line numbers"`. Prop names are the control labels.
fn label(name: &str) -> String {
    let name = name.replace('_', " ");
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The current value of every control, keyed by prop name - plus, in what
/// `render` receives, the way back into the controls.
#[derive(Clone, PartialEq)]
pub struct DemoValues(Vec<(&'static str, String)>, Option<Signal<DemoValues>>);

impl DemoValues {
    /// Writes a control's value from the preview, for a component whose own
    /// `onchange` should move the control that drives it.
    pub fn set(&self, name: &str, value: impl Into<String>) {
        let Some(mut values) = self.1 else {
            return;
        };
        if let Some(entry) = values.write().0.iter_mut().find(|(key, _)| *key == name) {
            entry.1 = value.into();
        }
    }

    pub fn str(&self, name: &str) -> String {
        self.0
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    }
}

/// Wraps the generated rsx in whatever the preview puts around it - for a
/// component whose point is how it sits inside something else. A page-level
/// constant, so two are always equal.
#[derive(Clone, Copy)]
pub struct Wrap(pub fn(&DemoValues, &str) -> String);

impl PartialEq for Wrap {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// The child the generated rsx prints, when a control decides it. A
/// page-level constant, so two are always equal.
#[derive(Clone, Copy)]
pub struct Child(pub fn(&DemoValues) -> String);

impl PartialEq for Child {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// `"auto"` is how a control says "leave it to the theme" - a control can
/// never produce *no* value.
pub fn or_unset<T>(value: String) -> Input<T>
where
    Input<T>: From<String>,
{
    match value.as_str() {
        "auto" => Input::None,
        _ => Input::from(value),
    }
}

/// The value a color control uses for "leave the prop unset". Kept apart from
/// the color names so no swatch has to double as a sentinel.
pub const UNSET: &str = "unset";

/// Indents generated rsx one level, for a `Wrap` that nests it.
pub fn indent(code: &str) -> String {
    code.lines().map(|line| format!("    {line}\n")).collect()
}

/// A live example: `render` on the left, a control per prop on the right,
/// and the rsx those values add up to below.
#[component]
pub fn Demo(
    component: String,
    children_text: String,
    /// Literal rsx for the children, printed verbatim - for children that are
    /// a subtree rather than one string. Wins over `children_text`.
    children_code: Option<String>,
    /// Overrides `children_code` per control state, for a subtree a control
    /// swaps out wholesale. Wins over both `children_code` and `children_text`.
    code_child: Option<Child>,
    /// Props the demo holds constant but the code block must still print - a
    /// required one like `aria_label`, which no control varies.
    #[props(default)]
    fixed: Vec<String>,
    controls: Vec<Control>,
    render: Callback<DemoValues, Element>,
    /// Must match what `render` draws around the component - the code block is
    /// a promise that copy-pasting reproduces the preview.
    wrap: Option<Wrap>,
    /// Overrides `children_text` per control state, for a child a control turns
    /// on and off.
    child: Option<Child>,
    /// Keeps the controls below the preview at every width, for a preview
    /// that needs the card's whole width - a notification host.
    #[props(default)]
    wide_preview: bool,
) -> Element {
    let mut values = use_signal(|| {
        DemoValues(
            controls
                .iter()
                .map(|control| (control.name, control.default.clone()))
                .collect(),
            None,
        )
    });

    let children_text = match child {
        Some(Child(child)) => child(&values()),
        None => children_text.clone(),
    };
    let children_code = match code_child {
        Some(Child(child)) => Some(child(&values())),
        None => children_code.clone(),
    };
    let source = super::generate_code(
        &component,
        &children_text,
        children_code.as_deref(),
        &fixed,
        &controls,
        &values(),
    );
    let source = match wrap {
        Some(Wrap(wrap)) => wrap(&values(), &source),
        None => source,
    };
    rsx! {
        Box {
            sx: sx()
                .border(border())
                .border_radius("6px")
                // Keeps the code block's own square corners inside the card's
                // rounded ones.
                .overflow("hidden")
                // Whether the control panel wraps below the preview depends on
                // this card's width, not the viewport's - under the docs shell
                // the two disagree by the nav's width.
                .container(DEMO_CARD),
            Flex {
                direction: "row",
                wrap: "wrap",
                align: "stretch",
                sx: sx().border_bottom(border()),
                Box {
                    sx: sx()
                        .flex("1")
                        .min_width("240px")
                        .padding("24px")
                        .display("flex")
                        .align_items("center")
                        // A preview wider than the card scrolls rather than
                        // being cut off; `safe` keeps its start reachable.
                        .justify_content("safe center")
                        .overflow_x("auto"),
                    {render.call(DemoValues(values().0, Some(values)))}
                }
                // No props, no panel - the preview and the code block are
                // the whole demo then.
                if !controls.is_empty() {
                    Flex {
                        // A wrapping row, not a column: every control claims a
                        // full line except a switch, which is a label and a
                        // 36px track - so a run of them shares one line and the
                        // panel stops growing a row per boolean.
                        direction: "row",
                        wrap: "wrap",
                        gap: "lg",
                        // Wrapped, the controls sit below the preview, so the
                        // divider has to move with them.
                        sx: {
                            let panel = sx()
                                .width("100%")
                                .flex_shrink("0")
                                .padding("24px")
                                .border_top(border());
                            if wide_preview {
                                // Below the preview at every width, so the
                                // preview gets the card's whole width.
                                panel
                            } else {
                                // The same 752px the flex row wraps at, plus
                                // slack so a subpixel rounding at the boundary
                                // can't put the wrap and the query on opposite
                                // sides.
                                panel.container_query(
                                    DEMO_CARD,
                                    "(min-width: 768px)",
                                    sx()
                                        // 464px of controls plus the padding
                                        // either side - `box-sizing` is
                                        // border-box here. Sized for the widest
                                        // control we have: a five-option
                                        // segmented group whose longest label is
                                        // `Elevated` (`Button`'s `variant`). Its
                                        // segments are `flex: 1 1 0`, so every
                                        // one is as wide as that longest label
                                        // needs - ~93px at `sm`, times five.
                                        .width("512px")
                                        .border_top("none")
                                        .border_left(border()),
                                )
                            }
                        },
                        // A hidden control is one the current mode does not
                        // have at all, so it is gone rather than greyed: the
                        // control that swaps the set sits above them and stays
                        // put under the pointer.
                        // Enumerated before the filter: `index` addresses
                        // `DemoValues`, which keeps every control's value,
                        // hidden or not.
                        for (index, control) in controls
                            .iter()
                            .enumerate()
                            .filter(|(_, control)| !control.is_hidden(&values()))
                        {
                            Flex {
                                key: "{control.name}",
                                direction: "column",
                                // `1 1 0` so a run of switches distributes over
                                // the whole width; the min stops a long label
                                // from wrapping under its own switch. Centred,
                                // because a track is far narrower than its
                                // share of the row - left-aligned it reads as
                                // three stray switches rather than a group.
                                sx: if control.kind == ControlKind::Switch {
                                    sx().flex("1 1 0").min_width("104px").align_items("center")
                                } else {
                                    sx().width("100%")
                                },
                                // The slider's bubble sits above its track, so it
                                // needs more room under the label than the rest.
                                gap: if control.kind == ControlKind::Slider { "sm" } else { "xs" },
                                // `NativeSelect` renders its own `<label>`, which is
                                // what names it - a second one would duplicate it.
                                if control.kind != ControlKind::Select {
                                    Text {
                                        size: "sm",
                                        sx: sx().font_weight("600"),
                                        {label(control.name)}
                                    }
                                }
                                match control.kind {
                                    ControlKind::Color => rsx! {
                                        ColorControl {
                                            control: control.clone(),
                                            label: label(control.name),
                                            value: values().str(control.name),
                                            onchange: move |value: String| {
                                                values.write().0[index].1 = value;
                                            },
                                        }
                                    },
                                    ControlKind::Slider => rsx! {
                                        Slider {
                                            size: "lg",
                                            aria_label: control.name,
                                            min: 0.0,
                                            max: (control.options.len() - 1) as f64,
                                            step: 1.0,
                                            value: control.step_of(&values().str(control.name)),
                                            // The step index means nothing to a
                                            // reader - the bubble shows the value
                                            // it stands for.
                                            format: {
                                                let options = control.options.clone();
                                                move |at: f64| options[at as usize].clone()
                                            },
                                            marks: control.marks(),
                                            oninput: {
                                                let options = control.options.clone();
                                                move |event: SliderChangeEvent| {
                                                    let at = event.value() as usize;
                                                    values.write().0[index].1 = options[at].clone();
                                                }
                                            },
                                        }
                                    },
                                    ControlKind::Select => rsx! {
                                        NativeSelect {
                                            size: "sm",
                                            label: label(control.name),
                                            // Matches the `Text { size: "sm" }`
                                            // label every other control kind
                                            // gets. The field owns its label
                                            // node, so the caption is styled
                                            // from the wrapper's `sx`.
                                            sx: sx().selector(
                                                "& > label",
                                                sx()
                                                    .font_weight("600")
                                                    .font_size(TEXT_FONT_SIZE.value(Size::Sm)),
                                            ),
                                            value: Some(values().str(control.name)),
                                            // A control panel's options are
                                            // data, so they arrive here rather
                                            // than from a `T` that could list
                                            // them statically.
                                            options: control.options.clone(),
                                            option_label: {
                                                let control = control.clone();
                                                move |option: String| control.label_of(&option)
                                            },
                                            onchange: move |value: String| {
                                                values.write().0[index].1 = value;
                                            },
                                        }
                                    },
                                    ControlKind::Switch => rsx! {
                                        Switch {
                                            aria_label: control.name,
                                            checked: control.is_on(&values().str(control.name)),
                                            onchange: move |on: bool| {
                                                values.write().0[index].1 = on.to_string();
                                            },
                                        }
                                    },
                                    ControlKind::Toggle => rsx! {
                                        SegmentedControl {
                                            // The panel is chrome, not the
                                            // demo - a filled default would
                                            // shout over the preview.
                                            variant: "outlined",
                                            size: "sm",
                                            full_width: true,
                                            value: values().str(control.name),
                                            // A control panel's options are data, so
                                            // they arrive here rather than from a `T`
                                            // that could list them statically.
                                            options: control.options.clone(),
                                            option_label: {
                                                let control = control.clone();
                                                move |option: String| OptionLabel::from(control.label_of(&option))
                                            },
                                            onchange: move |next: String| {
                                                values.write().0[index].1 = next;
                                            },
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
            CodeBlock {
                language: "rust",
                source,
                header: false,
                sx: sx().border("none").border_radius("0"),
            }
        }
    }
}
