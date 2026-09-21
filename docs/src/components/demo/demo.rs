use dioxus::prelude::*;
use libero::{
    components::{
        Box, CodeBlock, Flex, Input, NativeSelect, OptionLabel, SegmentedControl, Slider,
        SliderChangeEvent, Switch, Text,
    },
    sx::{Sx, sx},
    theme::{CODE_BLOCK_BORDER, Size, TEXT_FONT_SIZE},
};
use std::sync::LazyLock;

use super::{Control, ControlKind, color::ColorControl};

/// The card is the query container the control panel keys off.
const DEMO_CARD: &str = "demo-card";

/// The sidebar and page padding around the card on a desktop window.
const CARD_INSET: usize = 388;

/// `nested` once the card is `min_width` wide. Blitz has no container queries:
/// natively the window stands in, with the card's inset added.
fn card_query(base: Sx, min_width: usize, nested: Sx) -> Sx {
    if cfg!(any(feature = "native", feature = "native-cpu")) {
        base.media(format!("(min-width: {}px)", min_width + CARD_INSET), nested)
    } else {
        base.container_query(DEMO_CARD, format!("(min-width: {min_width}px)"), nested)
    }
}

/// Color inside the shorthand, or a later `border-left: 1px solid` resets it to `currentColor`.
/// `&'static` so the swatch closure can capture it without moving out of its `FnMut`.
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
    /// Every control at its default.
    pub fn defaults(controls: &[Control]) -> Self {
        Self(
            controls
                .iter()
                .map(|control| (control.name, control.default.clone()))
                .collect(),
            None,
        )
    }

    /// The same values with one control moved, for the snippet test.
    #[cfg(test)]
    pub fn with(&self, name: &str, value: &str) -> Self {
        let mut next = self.clone();
        if let Some(entry) = next.0.iter_mut().find(|(key, _)| *key == name) {
            entry.1 = value.to_string();
        }
        next
    }

    /// Writes a control's value from the preview, for a component whose own
    /// `onchange` should move the control that drives it.
    pub fn set(&self, name: &str, value: impl Into<String>) {
        let Some(mut values) = self.1 else {
            return;
        };
        // `render` is the page's callback, so its handlers run in the page's
        // scope; the write belongs to the `Demo` that owns the signal.
        dioxus::core::Runtime::current().in_scope(values.origin_scope(), || {
            if let Some(entry) = values.write().0.iter_mut().find(|(key, _)| *key == name) {
                entry.1 = value.into();
            }
        });
    }

    pub fn str(&self, name: &str) -> String {
        self.0
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    }
}

/// Wraps the generated rsx in what the preview puts around it. A page-level constant, so two
/// are always equal.
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

/// The narrowest segment of an `sm` `SegmentedControl` we plan for.
const SEGMENT_WIDTH: usize = 88;

/// A segment's width estimate per character at `sm` (14px), plus padding and border.
/// Measured: `Standard` 92px, `Bottom start` 117.5px.
const SEGMENT_CHAR_WIDTH: f32 = 7.5;
const SEGMENT_PADDING: f32 = 30.0;

/// Everything the panel spends on padding, either side of a control.
const PANEL_PADDING: usize = 48;

/// What the panel gives a control once it becomes the card's side column
/// (512px wide, see the panel's `sx` below).
const WIDE_PANEL_CONTROL: usize = 512 - PANEL_PADDING;

/// The card width a toggle's segments need, or `None` when segments fit at every width.
/// Narrow segments ellipsize their labels, so over three options fall back to a select.
fn segments_need(control: &Control) -> Option<usize> {
    let options = control.options.len();
    if options <= 3 {
        return None;
    }
    let longest = control
        .options
        .iter()
        .map(|option| control.label_of(option).chars().count())
        .max()
        .unwrap_or(0);
    let segment = (longest as f32 * SEGMENT_CHAR_WIDTH + SEGMENT_PADDING).ceil() as usize;
    let needed = options * SEGMENT_WIDTH.max(segment);
    Some(if needed > WIDE_PANEL_CONTROL {
        // Wider than the panel ever gets: a width no card reaches.
        99_999
    } else {
        needed + PANEL_PADDING
    })
}

/// The segmented half of a toggle control, shared by the always-segmented
/// path and the one a container query hides on a narrow card.
#[component]
fn ToggleSegments(control: Control, value: String, onchange: EventHandler<String>) -> Element {
    rsx! {
        SegmentedControl {
            // The panel is chrome, not the demo - a filled default would
            // shout over the preview.
            variant: "outlined",
            size: "sm",
            full_width: true,
            "aria-label": label(control.name),
            value,
            options: control.options.clone(),
            option_label: move |option: String| OptionLabel::from(control.label_of(&option)),
            onchange: move |next: String| onchange.call(next),
        }
    }
}

/// The shown controls, switches last so they share rows as one group. A stable sort keeps
/// the page's order within each group.
fn shown_controls<'a>(controls: &'a [Control], values: &DemoValues) -> Vec<(usize, &'a Control)> {
    let mut shown: Vec<_> = controls
        .iter()
        .enumerate()
        .filter(|(_, control)| !control.is_hidden(values))
        .collect();
    shown.sort_by_key(|(_, control)| control.kind == ControlKind::Switch);
    shown
}

/// Everything the code block is generated from, apart from the values.
#[derive(Clone)]
pub struct DemoCode {
    pub component: String,
    pub children_text: String,
    pub children_code: Option<String>,
    pub code_child: Option<Child>,
    pub fixed: Vec<String>,
    pub controls: Vec<Control>,
    pub wrap: Option<Wrap>,
    pub child: Option<Child>,
}

impl DemoCode {
    /// What the code block prints for these values.
    pub fn source(&self, values: &DemoValues) -> String {
        let children_text = match self.child {
            Some(Child(child)) => child(values),
            None => self.children_text.clone(),
        };
        let children_code = match self.code_child {
            Some(Child(child)) => Some(child(values)),
            None => self.children_code.clone(),
        };
        let source = super::generate_code(
            &self.component,
            &children_text,
            children_code.as_deref(),
            &self.fixed,
            &self.controls,
            values,
        );
        match self.wrap {
            Some(Wrap(wrap)) => wrap(values, &source),
            None => source,
        }
    }
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
    let mut values = use_signal(|| DemoValues::defaults(&controls));

    let code = DemoCode {
        component,
        children_text,
        children_code,
        code_child,
        fixed,
        controls: controls.clone(),
        wrap,
        child,
    };
    #[cfg(test)]
    use_hook(|| crate::snippets::record(&code));
    let source = code.source(&values());
    rsx! {
        Box {
            sx: sx()
                .border(border())
                .border_radius("6px")
                // Keeps the code block's own square corners inside the card's
                // rounded ones.
                .overflow("hidden")
                // The panel wraps on the card's width, not the viewport's (they differ by the nav).
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
                        // A wrapping row: every control takes a full line but a switch, so switches share one.
                        direction: "row",
                        wrap: "wrap",
                        // Stretched, a wrapped line's spare height goes below
                        // its controls instead of centring them in it.
                        align: "stretch",
                        gap: "lg",
                        sx: {
                            // Packed at the top, or a taller preview spreads the lines apart.
                            let panel = sx()
                                .align_content("start")
                                .width("100%")
                                .flex_shrink("0")
                                .padding("24px")
                                .border_top(border());
                            if wide_preview {
                                // Below the preview at every width, so the
                                // preview gets the card's whole width.
                                panel
                            } else {
                                // The flex row's 752px wrap plus slack against subpixel rounding.
                                card_query(
                                    panel,
                                    768,
                                    sx()
                                        // 464px of controls plus padding (border-box): five
                                        // `Elevated`-wide segments, ~93px each at `sm`.
                                        .width("512px")
                                        .border_top("none")
                                        .border_left(border()),
                                )
                            }
                        },
                        // `index` is from before the filter: it addresses `DemoValues`,
                        // which keeps hidden controls' values too.
                        for (index, control) in shown_controls(&controls, &values()) {
                            Flex {
                                key: "{control.name}",
                                direction: "column",
                                // Switches spread over the row, centred to read as a group;
                                // the min keeps a long label from wrapping under its switch.
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
                                            aria_label: label(control.name),
                                            min: 0.0,
                                            max: (control.options.len() - 1) as f64,
                                            step: 1.0,
                                            value: control.step_of(&values().str(control.name)),
                                            // The bubble shows the option, not the step index.
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
                                            // Matches the other controls' `Text { size: "sm" }` label,
                                            // styled from the wrapper since the field owns its label.
                                            sx: sx().selector(
                                                "& > label",
                                                sx()
                                                    .font_weight("600")
                                                    .font_size(TEXT_FONT_SIZE.value(Size::Sm)),
                                            ),
                                            value: Some(values().str(control.name)),
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
                                            aria_label: label(control.name),
                                            checked: control.is_on(&values().str(control.name)),
                                            onchange: move |on: bool| {
                                                values.write().0[index].1 = on.to_string();
                                            },
                                        }
                                    },
                                    // Too wide for the card, a select: a container query shows one,
                                    // and `display: none` keeps the other out of tab order and a11y tree.
                                    ControlKind::Toggle if segments_need(control).is_some() => {
                                        let needed = segments_need(control).unwrap_or_default();
                                        rsx! {
                                            Box {
                                                sx: card_query(sx().display("none"), needed, sx().display("block")),
                                                ToggleSegments {
                                                    control: control.clone(),
                                                    value: values().str(control.name),
                                                    onchange: move |next: String| {
                                                        values.write().0[index].1 = next;
                                                    },
                                                }
                                            }
                                            Box {
                                                sx: card_query(sx().display("block"), needed, sx().display("none")),
                                                NativeSelect {
                                                    size: "sm",
                                                    // The row's `Text` names it: no second label.
                                                    "aria-label": label(control.name),
                                                    value: Some(values().str(control.name)),
                                                    options: control.options.clone(),
                                                    option_label: {
                                                        let control = control.clone();
                                                        move |option: String| control.label_of(&option)
                                                    },
                                                    onchange: move |value: String| {
                                                        values.write().0[index].1 = value;
                                                    },
                                                }
                                            }
                                        }
                                    },
                                    ControlKind::Toggle => rsx! {
                                        ToggleSegments {
                                            control: control.clone(),
                                            value: values().str(control.name),
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

#[cfg(test)]
mod tests {
    use super::{Control, segments_need};

    /// Todo 866: the longest label sizes every segment, so four placements
    /// never fit the side column, while short labels keep the 88px floor.
    #[test]
    fn the_longest_label_sizes_the_segments() {
        let placement = Control::toggle("placement", ["a", "b", "c", "d"]).labels([
            "Top end",
            "Top start",
            "Bottom end",
            "Bottom start",
        ]);
        assert_eq!(segments_need(&placement), Some(99_999));
        let variant = Control::toggle("variant", ["f", "t", "e", "o", "s"])
            .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"]);
        assert_eq!(segments_need(&variant), Some(5 * 90 + 48));
        let sizes = Control::toggle("size", ["xs", "sm", "md", "lg", "xl"]);
        assert_eq!(segments_need(&sizes), Some(5 * 88 + 48));
    }

    /// WCAG 2.5.3: a control's accessible name must contain its visible caption.
    #[test]
    fn controls_are_named_by_their_caption() {
        let source = include_str!("demo.rs");
        let sites: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("aria_label:") || line.starts_with("\"aria-label\":"))
            .collect();
        assert_eq!(sites.len(), 4, "{sites:?}");
        for site in sites {
            assert!(site.ends_with(": label(control.name),"), "{site}");
        }
    }
}
