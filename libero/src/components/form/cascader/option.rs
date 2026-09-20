use crate::components::common::Options;

/// One entry of a `Cascader`'s tree. Values must be unique across the whole tree.
///
/// ```rust
/// # use libero::components::CascaderOption;
/// let fruit: CascaderOption<&str> = CascaderOption::new("fruit", "Fruit").children(vec![
///     CascaderOption::new("apple", "Apple"),
///     CascaderOption::new("quince", "Quince").disabled(true),
/// ]);
/// ```
///
/// Docs: <https://libero-ui.dev/form/cascader>
#[derive(Clone, Debug, PartialEq)]
pub struct CascaderOption<T> {
    pub value: T,
    pub label: String,
    pub children: Vec<CascaderOption<T>>,
    /// Also disables everything under it.
    pub disabled: bool,
}

impl<T> CascaderOption<T> {
    /// `impl Into<T>`, so a `CascaderOption<String>` takes a `&str` value.
    pub fn new(value: impl Into<T>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: Vec::new(),
            disabled: false,
        }
    }

    pub fn children(mut self, children: Vec<CascaderOption<T>>) -> Self {
        self.children = children;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// One option without its `T`, so the engine compiles once rather than once per `T`.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct CascaderNode {
    pub label: String,
    pub disabled: bool,
    pub children: Vec<CascaderNode>,
}

impl CascaderNode {
    pub(super) fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

pub(super) fn erase<T>(options: &[CascaderOption<T>]) -> Vec<CascaderNode> {
    options
        .iter()
        .map(|option| CascaderNode {
            label: option.label.clone(),
            disabled: option.disabled,
            children: erase(&option.children),
        })
        .collect()
}

/// The index path to the option holding `value`, depth first. `None` when no option holds it.
pub(super) fn indices_for_value<T: Options>(
    options: &[CascaderOption<T>],
    value: &T,
) -> Option<Vec<usize>> {
    for (index, option) in options.iter().enumerate() {
        if &option.value == value {
            return Some(vec![index]);
        }
        if let Some(mut below) = indices_for_value(&option.children, value) {
            below.insert(0, index);
            return Some(below);
        }
    }
    None
}

/// The options on the path to `indices`, root first.
pub(super) fn options_at<'a, T>(
    options: &'a [CascaderOption<T>],
    indices: &[usize],
) -> Vec<&'a CascaderOption<T>> {
    let mut chain = Vec::with_capacity(indices.len());
    let mut level = options;
    for index in indices {
        let Some(option) = level.get(*index) else {
            break;
        };
        chain.push(option);
        level = &option.children;
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Vec<CascaderOption<String>> {
        vec![
            CascaderOption::new("food", "Food").children(vec![
                CascaderOption::new("fruit", "Fruit")
                    .children(vec![CascaderOption::new("apple", "Apple")]),
                CascaderOption::new("veg", "Veg")
                    .children(vec![CascaderOption::new("leek", "Leek")]),
            ]),
            CascaderOption::new("other", "Other"),
        ]
    }

    #[test]
    fn a_value_finds_its_path_at_any_level() {
        let tree = tree();
        let indices = indices_for_value(&tree, &"leek".to_string()).unwrap();
        assert_eq!(indices, vec![0, 1, 0]);
        let labels: Vec<&str> = options_at(&tree, &indices)
            .iter()
            .map(|option| option.label.as_str())
            .collect();
        assert_eq!(labels, vec!["Food", "Veg", "Leek"]);
        // A branch's value, what `any_level` commits.
        assert_eq!(
            indices_for_value(&tree, &"fruit".to_string()),
            Some(vec![0, 0])
        );
        assert_eq!(
            indices_for_value(&tree, &"other".to_string()),
            Some(vec![1])
        );
    }

    #[test]
    fn a_value_that_is_not_there_resolves_to_nothing() {
        assert!(indices_for_value(&tree(), &"nope".to_string()).is_none());
        assert!(indices_for_value::<String>(&[], &"apple".to_string()).is_none());
    }

    #[test]
    fn erasing_keeps_labels_disabled_and_shape() {
        let erased = erase(&[CascaderOption::<String>::new("a", "A")
            .disabled(true)
            .children(vec![CascaderOption::new("b", "B")])]);
        assert_eq!(erased[0].label, "A");
        assert!(erased[0].disabled);
        assert_eq!(erased[0].children[0].label, "B");
    }
}
