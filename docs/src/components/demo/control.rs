use libero::components::SliderMark;

use super::DemoValues;

/// How a control offers its options.
#[derive(Clone, Copy, PartialEq)]
pub enum ControlKind {
    /// A swatch per value, filled with the color it names.
    Color,
    /// One step per value - for an ordered scale, where the options' order is
    /// the scale's.
    Slider,
    /// A segmented control, for a handful of unordered values.
    Toggle,
    /// An on/off switch, for a `bool` prop.
    Switch,
    /// A dropdown, for more values than a segmented control can fit.
    Select,
}

/// One prop of the demoed component, exposed as its literal values.
#[derive(Clone)]
pub struct Control {
    pub name: &'static str,
    pub kind: ControlKind,
    pub options: Vec<String>,
    pub default: String,
    /// What the code block prints for this control, when that isn't just
    /// `name: "value"` - an unquoted `bool`, a switch standing in for a real
    /// string, or one control driving two props (and reading the others, hence
    /// the whole `DemoValues`). Set, it also bypasses the omit-if-default
    /// rule, so the fn decides when to print nothing.
    pub code: Option<fn(&Control, &DemoValues) -> Vec<String>>,
    /// Display text per option, when the value alone doesn't read - `"python
    /// (not enabled)"` for a value that stays `"python"`. Positional.
    pub labels: Option<Vec<String>>,
    /// When this prop is inert because another control pins it - the control
    /// still renders, greyed out, rather than vanishing under the pointer.
    pub inert: Option<fn(&DemoValues) -> bool>,
}

/// `code` is a page-level constant, so whether one is set is all that can
/// change - and comparing fn pointers is not meaningful.
impl PartialEq for Control {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.kind == other.kind
            && self.options == other.options
            && self.default == other.default
            && self.code.is_some() == other.code.is_some()
            && self.labels == other.labels
            && self.inert.is_some() == other.inert.is_some()
    }
}

impl Control {
    /// The first option is the default until `default` says otherwise.
    pub fn color<const N: usize>(name: &'static str, options: [&str; N]) -> Self {
        Self::new(name, ControlKind::Color, &options)
    }

    /// Options in scale order; pair it with `default`, since that is rarely
    /// the first step.
    pub fn slider<const N: usize>(name: &'static str, options: [&str; N]) -> Self {
        Self::new(name, ControlKind::Slider, &options)
    }

    pub fn toggle<const N: usize>(name: &'static str, options: [&str; N]) -> Self {
        Self::new(name, ControlKind::Toggle, &options)
    }

    pub fn select<const N: usize>(name: &'static str, options: [&str; N]) -> Self {
        Self::new(name, ControlKind::Select, &options)
    }

    /// Off by default; `default("true")` flips that. Prints unquoted.
    pub fn switch(name: &'static str) -> Self {
        Self::new(name, ControlKind::Switch, &["false", "true"]).code(|control, values| {
            let value = values.str(control.name);
            if value == control.default {
                vec![]
            } else {
                vec![format!("{}: {value}", control.name)]
            }
        })
    }

    fn new(name: &'static str, kind: ControlKind, options: &[&str]) -> Self {
        let options: Vec<String> = options.iter().map(|o| o.to_string()).collect();
        let default = options.first().cloned().unwrap_or_default();
        Self {
            name,
            kind,
            options,
            default,
            code: None,
            labels: None,
            inert: None,
        }
    }

    /// Which step a value sits at - `ControlKind::Slider` addresses its
    /// options by index.
    pub fn step_of(&self, value: &str) -> f64 {
        self.options.iter().position(|o| o == value).unwrap_or(0) as f64
    }

    /// A tick per option, captioned only at the ends - six captions under a
    /// 172px track collide.
    pub fn marks(&self) -> Vec<SliderMark> {
        let last = self.options.len() - 1;
        self.options
            .iter()
            .enumerate()
            .map(|(at, option)| {
                if at == 0 || at == last {
                    SliderMark::labeled(at as f64, option)
                } else {
                    SliderMark::new(at as f64)
                }
            })
            .collect()
    }

    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = value.into();
        self
    }

    pub fn code(mut self, code: fn(&Control, &DemoValues) -> Vec<String>) -> Self {
        self.code = Some(code);
        self
    }

    /// Greys the control out while `inert` holds - for a prop another
    /// control has taken over.
    pub fn inert_when(mut self, inert: fn(&DemoValues) -> bool) -> Self {
        self.inert = Some(inert);
        self
    }

    pub fn is_inert(&self, values: &DemoValues) -> bool {
        self.inert.is_some_and(|inert| inert(values))
    }

    pub fn labels<const N: usize>(mut self, labels: [&str; N]) -> Self {
        self.labels = Some(labels.iter().map(|l| l.to_string()).collect());
        self
    }

    /// What an option reads as - its own value unless `labels` renames it.
    pub fn label_of(&self, value: &str) -> String {
        self.labels
            .as_ref()
            .and_then(|labels| labels.get(self.step_of(value) as usize))
            .cloned()
            .unwrap_or_else(|| value.to_string())
    }

    pub fn is_on(&self, value: &str) -> bool {
        value == "true"
    }
}

/// The rsx a caller would write for the current control values. Props left at
/// their default are omitted, matching what someone would actually type.
pub fn generate_code(
    component: &str,
    children_text: &str,
    children_code: Option<&str>,
    fixed: &[String],
    controls: &[Control],
    values: &DemoValues,
) -> String {
    let mut set: Vec<String> = fixed.to_vec();
    set.extend(controls.iter().flat_map(|control| {
        let value = values.str(control.name);
        match control.code {
            Some(code) => code(control, values),
            None if value != control.default => {
                vec![format!("{}: {value:?}", control.name)]
            }
            None => vec![],
        }
    }));

    // A component driven entirely by props (`Code`'s `source`) has no child.
    let child = children_code
        .map(str::to_string)
        .or_else(|| (!children_text.is_empty()).then(|| format!("{children_text:?}")));
    let one_line = child.as_ref().is_none_or(|child| !child.contains('\n'));

    match (set.is_empty() && one_line, &child) {
        (true, None) => format!("{component} {{}}"),
        (true, Some(child)) => format!("{component} {{ {child} }}"),
        (false, _) => {
            let props = set
                .iter()
                .map(|prop| format!("    {prop},\n"))
                .collect::<String>();
            let child = child.as_deref().map(super::indent).unwrap_or_default();
            format!("{component} {{\n{props}{child}}}")
        }
    }
}
