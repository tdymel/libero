//! A value kept across renders and rebuilt only when its inputs change.

use std::rc::Rc;

/// The last output with the key it was built from.
pub(super) struct Memo<K, O>(Option<(K, O)>);

impl<K, O> Default for Memo<K, O> {
    fn default() -> Self {
        Self(None)
    }
}

impl<K, O> Memo<K, O> {
    /// The kept output while `fresh` accepts its key, else the one `compute` builds with its key.
    pub fn get(&mut self, fresh: impl FnOnce(&K) -> bool, compute: impl FnOnce() -> (K, O)) -> &O {
        if self.0.as_ref().is_some_and(|(key, _)| !fresh(key)) {
            self.0 = None;
        }
        &self.0.get_or_insert_with(compute).1
    }

    pub fn clear(&mut self) {
        self.0 = None;
    }
}

/// Whether `known` holds the rows of `data`: the same allocation, else equal rows.
pub(super) fn same_rows<T: PartialEq>(known: &Rc<Vec<T>>, data: &Rc<Vec<T>>) -> bool {
    Rc::ptr_eq(known, data) || known == data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_output_is_built_again_only_once_its_key_is_stale() {
        let mut memo = Memo::<u32, String>::default();
        let mut built = 0;
        let mut get = |memo: &mut Memo<u32, String>, key: u32| {
            memo.get(
                |kept| *kept == key,
                || {
                    built += 1;
                    (key, format!("v{key}"))
                },
            )
            .clone()
        };
        assert_eq!(get(&mut memo, 1), "v1");
        assert_eq!(get(&mut memo, 1), "v1");
        assert_eq!(get(&mut memo, 2), "v2");
        memo.clear();
        assert_eq!(get(&mut memo, 2), "v2");
        assert_eq!(built, 3);
    }

    #[test]
    fn rows_match_by_allocation_or_by_equality() {
        let rows = Rc::new(vec![1, 2]);
        assert!(same_rows(&rows, &rows.clone()));
        assert!(same_rows(&rows, &Rc::new(vec![1, 2])));
        assert!(!same_rows(&rows, &Rc::new(vec![1, 3])));
    }
}
