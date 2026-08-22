use dioxus::prelude::*;
use libero::{
    components::{
        Box, CodeBlock, Flex, Input, Option as SelectOption, Select, Slider, SliderChangeEvent,
        Switch, Text, ToggleButton, ToggleButtonGroup,
    },
    sx::sx,
    theme::{CODE_BLOCK_BORDER, Size, TEXT_FONT_SIZE},
};
use std::sync::LazyLock;

use crate::icons::CheckmarkIcon;

use super::{Control, ControlKind};

/// The card is the query container the control panel keys off.
const DEMO_CARD: &str = "demo-card";

/// The color belongs *inside* the shorthand: a later `border-left: 1px solid`
/// would otherwise reset the color to `currentColor`. A `&'static str`, not a
/// local `String`, so the swatch closure can capture it without moving it out
/// of the `FnMut` it lives in.
static BORDER: LazyLock<String> =
    LazyLock::new(|| format!("1px solid {}", CODE_BLOCK_BORDER.value()));

fn border() -> &'static str {
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

/// The current value of every control, keyed by prop name.
#[derive(Clone, PartialEq)]
pub struct DemoValues(Vec<(&'static str, String)>);

impl DemoValues {
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
/// the color names so no swatch has to double as a sentinel; it fills white,
/// which is what an unset color renders as.
pub const UNSET: &str = "unset";

/// Fill and tick color for one swatch.
fn swatch(option: &str) -> (String, String) {
    match option {
        UNSET => ("white".to_string(), "black".to_string()),
        // The palette's own contrast color, so the tick reads on every swatch.
        color => (color.to_string(), format!("{color}-contrast")),
    }
}

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
) -> Element {
    let mut values = use_signal(|| {
        DemoValues(
            controls
                .iter()
                .map(|control| (control.name, control.default.clone()))
                .collect(),
        )
    });

    let children_text = match child {
        Some(Child(child)) => child(&values()),
        None => children_text.clone(),
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
                        .justify_content("center"),
                    {render.call(values())}
                }
                // No props, no panel - the preview and the code block are
                // the whole demo then.
                if !controls.is_empty() {
                    Flex {
                        direction: "column",
                        gap: "lg",
                        // Wrapped, the controls sit below the preview, so the
                        // divider has to move with them.
                        sx: sx()
                            .width("100%")
                            .flex_shrink("0")
                            .padding("24px")
                            .border_top(border())
                            // The same 628px the flex row wraps at, plus slack
                            // so a subpixel rounding at the boundary can't put
                            // the wrap and the query on opposite sides.
                            .container_query(
                                DEMO_CARD,
                                "(min-width: 640px)",
                                sx()
                                    // 340px of controls plus the padding either
                                    // side - `box-sizing` is border-box here.
                                    // Wide enough for a four-option segmented
                                    // group (`ScrollArea`'s `scrollbars`).
                                    .width("388px")
                                    .border_top("none")
                                    .border_left(border()),
                            ),
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
                                // The slider's bubble sits above its track, so it
                                // needs more room under the label than the rest.
                                gap: if control.kind == ControlKind::Slider { "sm" } else { "xs" },
                                // `Select` renders its own `<label>`, which is
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
                                        ToggleButtonGroup {
                                            size: "sm",
                                            full_width: true,
                                            // Swatches read as separate chips, not one
                                            // segmented control.
                                            gap: "xs",
                                            variant: "text",
                                            value: vec![values().str(control.name)],
                                            // Clicking the selected swatch would otherwise
                                            // clear it, leaving the prop with no value.
                                            onchange: move |next: Vec<String>| {
                                                if let Some(value) = next.into_iter().next() {
                                                    values.write().0[index].1 = value;
                                                }
                                            },
                                            for option in control.options.iter() {
                                                ToggleButton {
                                                    key: "{option}",
                                                    value: "{option}",
                                                    aria_label: "{option}",
                                                    // The swatch is the whole button, so
                                                    // the fill reaches the border - and
                                                    // the variant's hover tint would show
                                                    // as a halo around it.
                                                    sx: sx()
                                                        .padding("0")
                                                        .hover(sx().background("transparent")),
                                                    Box {
                                                        sx: sx()
                                                            .width("100%")
                                                            .height("100%")
                                                            .border_radius("inherit")
                                                            .background(swatch(option).0)
                                                            .color(swatch(option).1)
                                                            // A pale swatch needs an edge
                                                            // to read as a swatch at all.
                                                            .border(border())
                                                            .display("flex")
                                                            .align_items("center")
                                                            .justify_content("center")
                                                            .selector(
                                                                "& svg",
                                                                sx().width("18px").height("18px"),
                                                            ),
                                                        if values().str(control.name) == *option {
                                                            CheckmarkIcon {}
                                                        }
                                                    }
                                                }
                                            }
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
                                            label: {
                                                let options = control.options.clone();
                                                move |at: f64| options[at as usize].clone()
                                            },
                                            marks: control.marks(),
                                            on_change: {
                                                let options = control.options.clone();
                                                move |event: SliderChangeEvent| {
                                                    let at = event.value() as usize;
                                                    values.write().0[index].1 = options[at].clone();
                                                }
                                            },
                                        }
                                    },
                                    ControlKind::Select => rsx! {
                                        Select {
                                            size: "sm",
                                            label: label(control.name),
                                            // Matches the `Text { size: "sm" }`
                                            // label every other control kind gets.
                                            label_sx: sx()
                                                .font_weight("600")
                                                .font_size(TEXT_FONT_SIZE.value(Size::Sm)),
                                            value: values().str(control.name),
                                            onchange: move |value: String| {
                                                values.write().0[index].1 = value;
                                            },
                                            for option in control.options.iter() {
                                                SelectOption {
                                                    key: "{option}",
                                                    value: "{option}",
                                                    {control.label_of(option)}
                                                }
                                            }
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
                                        ToggleButtonGroup {
                                            size: "sm",
                                            full_width: true,
                                            value: vec![values().str(control.name)],
                                            onchange: move |next: Vec<String>| {
                                                if let Some(value) = next.into_iter().next() {
                                                    values.write().0[index].1 = value;
                                                }
                                            },
                                            for option in control.options.iter() {
                                                ToggleButton {
                                                    key: "{option}",
                                                    value: "{option}",
                                                    {control.label_of(option)}
                                                }
                                            }
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
