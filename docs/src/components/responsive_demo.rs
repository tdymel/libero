//! A demo's `responsive` switch (todo 5655): on, the `size` control's pick holds from `md` up
//! and `xs` below it, so a resize shows the breakpoint.

use libero::theme::{Responsive, Size, responsive};

use super::{Control, DemoValues};

pub fn responsive_on(values: &DemoValues) -> bool {
    values.str("responsive") == "true"
}

/// On by default, so the breakpoint shows without another switch: resize the window.
pub fn responsive_switch() -> Control {
    Control::switch("responsive")
        .default("true")
        .code(|_, _| vec![])
}

/// `size`: `xs` below `md` and the pick from it while the switch is on, else the pick.
pub fn responsive_size(values: &DemoValues) -> Responsive<Size> {
    let size = Size::from(values.str("size").as_str());
    match responsive_on(values) {
        true => responsive(Size::Xs).md(size),
        false => size.into(),
    }
}

/// The `size` control's code line, the responsive form while the switch is on.
pub fn responsive_size_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match (responsive_on(values), values.str("size")) {
        (true, size) => vec![format!(
            "size: responsive(Size::Xs).md(Size::{:?})",
            Size::from(size.as_str())
        )],
        (false, size) if size == control.default => vec![],
        (false, size) => vec![format!("size: {size:?}")],
    }
}
