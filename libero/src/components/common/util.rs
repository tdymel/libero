#[macro_export]
macro_rules! sx_var {
    ($name:literal) => {
        concat!("var(--lsx-", $name, ")")
    };
}

/// Builds a non-global attribute (e.g. `src`/`href`) to push into a
/// `Vec<Attribute>` by hand, bypassing `extends = GlobalAttributes`.
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
