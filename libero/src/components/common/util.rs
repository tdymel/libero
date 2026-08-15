#[macro_export]
macro_rules! sx_var {
    ($name:literal) => {
        concat!("var(--lsx-", $name, ")")
    };
}

/// Builds a plain attribute (e.g. `src`/`alt`/`value`/`href` - anything not
/// part of `GlobalAttributes`) to push into a `Vec<Attribute>` by hand.
/// `extends = GlobalAttributes` only lets a component accept *global*
/// attribute names at its call site; this bypasses that entirely, since a
/// `Vec<Attribute>` spread (`..attrs`) onto an element isn't re-validated
/// per-tag the way named `rsx!` attribute syntax is.
pub(crate) fn attr<T>(
    name: &'static str,
    value: impl dioxus::core::IntoAttributeValue<T>,
) -> dioxus::prelude::Attribute {
    dioxus::prelude::Attribute::new(name, value, None, false)
}

pub(crate) fn class_list(classes: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    let classes = classes
        .into_iter()
        .flatten()
        .filter(|class| !class.is_empty())
        .collect::<Vec<_>>();

    if classes.is_empty() {
        None
    } else {
        Some(classes.join(" "))
    }
}
