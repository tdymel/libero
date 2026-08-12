#[macro_export]
macro_rules! sx_var {
    ($name:literal) => {
        concat!("var(--lsx-", $name, ")")
    };
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
