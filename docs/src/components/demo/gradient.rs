use libero::theme::Gradient;

use super::{Control, DemoValues};

const TO: &str = "gradient_to";
const DEG: &str = "gradient_deg";
const DEFAULT_TO: &str = "secondary";
const DEFAULT_DEG: &str = "45";

/// The gradient's second colour and angle, printed as one `gradient: (to, deg)` tuple.
/// `color` is the first stop; `hidden` drops both while no gradient shows.
pub fn gradient_controls(hidden: fn(&DemoValues) -> bool) -> [Control; 2] {
    [
        Control::color(TO)
            .default(DEFAULT_TO)
            .hidden_when(hidden)
            .code(gradient_code),
        Control::slider(DEG, ["0", "45", "90", "135", "180", "225", "270", "315"])
            .default(DEFAULT_DEG)
            .hidden_when(hidden)
            .code(|_, _| vec![]),
    ]
}

fn gradient_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let (to, deg) = (values.str(TO), values.str(DEG));
    if control.is_hidden(values) || (to == DEFAULT_TO && deg == DEFAULT_DEG) {
        return vec![];
    }
    vec![format!("gradient: ({to:?}, {deg})")]
}

/// `hidden` for a component with a `variant` control.
pub fn not_gradient_variant(values: &DemoValues) -> bool {
    values.str("variant") != "gradient"
}

/// The gradient the two controls describe.
pub fn gradient_value(values: &DemoValues) -> Gradient {
    let deg = values.str(DEG).parse().unwrap_or(45);
    (values.str(TO), deg).into()
}
