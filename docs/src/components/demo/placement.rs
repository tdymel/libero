use libero::hooks::{Align, Side};

use super::{Control, DemoValues};

impl Control {
    /// A `side: Side` toggle over `options` (some of top, end, bottom, start), `bottom` first.
    pub fn side<const N: usize>(options: [&str; N]) -> Self {
        Self::toggle("side", options)
            .capitalised()
            .default("bottom")
            .code(enum_code)
    }

    /// An `align: Align` toggle, `start` first.
    pub fn align() -> Self {
        Self::toggle("align", ["start", "center", "end"])
            .capitalised()
            .default("start")
            .code(enum_code)
    }

    /// A delay in milliseconds, `auto` first: the theme's own delay, printed as nothing.
    pub fn delay<const N: usize>(name: &'static str, options: [&str; N]) -> Self {
        Self::slider(name, options).code(delay_code)
    }
}

pub fn side_of(value: &str) -> Side {
    match value {
        "top" => Side::Top,
        "start" => Side::Start,
        "end" => Side::End,
        _ => Side::Bottom,
    }
}

pub fn align_of(value: &str) -> Align {
    match value {
        "center" => Align::Center,
        "end" => Align::End,
        _ => Align::Start,
    }
}

/// `None` for `auto`, the theme's delay.
pub fn delay_of(value: &str) -> Option<u32> {
    value.parse().ok()
}

/// `side` and `align` are the popover's enums, not strings, so the default
/// printer's `side: "top"` would not compile.
fn enum_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    if value == control.default {
        return vec![];
    }
    let (kind, variant) = match control.name {
        "side" => ("Side", format!("{:?}", side_of(&value))),
        _ => ("Align", format!("{:?}", align_of(&value))),
    };
    vec![format!("{}: {kind}::{variant}", control.name)]
}

/// Milliseconds print unquoted.
fn delay_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match values.str(control.name).as_str() {
        "auto" => vec![],
        delay => vec![format!("{}: {delay}", control.name)],
    }
}
