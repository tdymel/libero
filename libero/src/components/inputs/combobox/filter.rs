/// What a row is matched against. `name` is the label `option_label` already
/// resolved, so the default filter searches what the user can actually read -
/// which a filter on `Options::label` could not, once a label is translated.
#[derive(Clone, PartialEq)]
pub struct ComboboxFilterArgs<T> {
    pub query: String,
    pub value: T,
    pub name: String,
}

pub(super) fn contains_ignoring_case(query: &str, name: &str) -> bool {
    name.to_lowercase().contains(&query.to_lowercase())
}
