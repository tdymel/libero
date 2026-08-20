/// One prop of the demoed component, exposed as a swatch per literal value.
#[derive(Clone, PartialEq)]
pub struct Control {
    pub name: &'static str,
    pub options: Vec<String>,
    pub default: String,
}

impl Control {
    /// The first option is the default until `default` says otherwise.
    pub fn color<const N: usize>(name: &'static str, options: [&str; N]) -> Self {
        let options: Vec<String> = options.iter().map(|o| o.to_string()).collect();
        let default = options.first().cloned().unwrap_or_default();
        Self {
            name,
            options,
            default,
        }
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
