use libero::components::SliderMark;

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
}

/// One prop of the demoed component, exposed as its literal values.
#[derive(Clone, PartialEq)]
pub struct Control {
    pub name: &'static str,
    pub kind: ControlKind,
    pub options: Vec<String>,
    pub default: String,
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

    fn new(name: &'static str, kind: ControlKind, options: &[&str]) -> Self {
        let options: Vec<String> = options.iter().map(|o| o.to_string()).collect();
        let default = options.first().cloned().unwrap_or_default();
        Self {
            name,
            kind,
            options,
            default,
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
}

/// The rsx a caller would write for the current control values. Props left at
/// their default are omitted, matching what someone would actually type.
pub fn generate_code(
    component: &str,
    children_text: &str,
    controls: &[Control],
    values: &[(&'static str, String)],
) -> String {
    let set: Vec<String> = controls
        .iter()
        .zip(values)
        .filter(|(control, (_, value))| *value != control.default)
        .map(|(control, (_, value))| format!("{}: {value:?}", control.name))
        .collect();

    if set.is_empty() {
        return format!("{component} {{ {children_text:?} }}");
    }

    let props = set
        .iter()
        .map(|prop| format!("    {prop},\n"))
        .collect::<String>();
    format!("{component} {{\n{props}    {children_text:?}\n}}")
}
