//! A demo's `responsive` switch (todo 5655): on, the `size` control's pick holds from `md` up
//! and `xs` below it, so a resize shows the breakpoint.

use libero::{
    sx::ThemeAwareValue,
    theme::{Responsive, Size, responsive},
};

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
    responsive_step(values, "size")
}

/// A step control (`size`, `thickness`) read the same way.
pub fn responsive_step(values: &DemoValues, name: &str) -> Responsive<Size> {
    let size = Size::from(values.str(name).as_str());
    match responsive_on(values) {
        true => responsive(Size::Xs).md(size),
        false => size.into(),
    }
}

/// The `size` control's code line, the responsive form while the switch is on.
pub fn responsive_size_code(control: &Control, values: &DemoValues) -> Vec<String> {
    responsive_step_code("size", control, values)
}

/// A step control's code line, the responsive form while the switch is on.
pub fn responsive_step_code(name: &str, control: &Control, values: &DemoValues) -> Vec<String> {
    match (responsive_on(values), values.str(name)) {
        (true, size) => vec![format!(
            "{name}: responsive(Size::Xs).md(Size::{:?})",
            Size::from(size.as_str())
        )],
        (false, size) if size == control.default => vec![],
        (false, size) => vec![format!("{name}: {size:?}")],
    }
}

/// A size-or-CSS control (`radius`, `gap`, `size`): `narrow` below `md` and the pick from it
/// while the switch is on, else the pick.
pub fn responsive_value(
    values: &DemoValues,
    name: &str,
    narrow: Size,
) -> Responsive<ThemeAwareValue> {
    let pick = ThemeAwareValue::from(values.str(name));
    match responsive_on(values) {
        true => responsive(ThemeAwareValue::from(narrow)).md(pick),
        false => pick.into(),
    }
}

/// [`responsive_value`] as the expression the snippet prints.
pub fn responsive_value_expr(values: &DemoValues, name: &str, narrow: Size) -> String {
    let pick = values.str(name);
    let size = ["xs", "sm", "md", "lg", "xl", "xxl"].contains(&pick.as_str());
    match (responsive_on(values), size) {
        (true, true) => format!(
            "responsive(Size::{narrow:?}).md(Size::{:?})",
            Size::from(pick.as_str())
        ),
        (true, false) => {
            format!("responsive(ThemeAwareValue::from(Size::{narrow:?})).md({pick:?}.into())")
        }
        (false, _) => format!("{pick:?}"),
    }
}

/// A size-or-CSS control's code line, `name: <expression>`; the plain default prints nothing.
pub fn responsive_value_code(
    name: &str,
    narrow: Size,
    control: &Control,
    values: &DemoValues,
) -> Vec<String> {
    match responsive_on(values) || values.str(name) != control.default {
        true => vec![format!(
            "{name}: {}",
            responsive_value_expr(values, name, narrow)
        )],
        false => vec![],
    }
}
