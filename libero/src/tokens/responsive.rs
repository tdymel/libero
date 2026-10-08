use std::fmt;

use super::Size;

/// A prop value that changes with the viewport: a `base`, and an override from each
/// named [`Size`] breakpoint up. A plain value converts, so `cols: 3` also works.
///
/// ```rust
/// use libero::theme::{Size, responsive};
///
/// let cols = responsive(1u8).sm(2).lg(3);
/// assert_eq!(cols.base(), 1);
/// assert_eq!(cols.at(Size::Md), 2);
/// ```
///
/// Docs: <https://libero-ui.dev/about/styling>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Responsive<T> {
    base: T,
    breakpoints: [Option<T>; Size::ALL.len()],
}

impl<T> Responsive<T> {
    pub const fn new(base: T) -> Self {
        Self {
            base,
            breakpoints: [const { None }; Size::ALL.len()],
        }
    }

    /// From `size`'s breakpoint up. Re-declaring a size replaces it.
    pub const fn with(mut self, size: Size, value: T) -> Self {
        // A `const fn` cannot drop a generic `T`: a replaced owning value leaks.
        std::mem::forget(self.breakpoints[size.index()].replace(value));
        self
    }

    pub const fn xs(self, value: T) -> Self {
        self.with(Size::Xs, value)
    }

    pub const fn sm(self, value: T) -> Self {
        self.with(Size::Sm, value)
    }

    pub const fn md(self, value: T) -> Self {
        self.with(Size::Md, value)
    }

    pub const fn lg(self, value: T) -> Self {
        self.with(Size::Lg, value)
    }

    pub const fn xl(self, value: T) -> Self {
        self.with(Size::Xl, value)
    }

    pub const fn xxl(self, value: T) -> Self {
        self.with(Size::Xxl, value)
    }

    /// Below every breakpoint.
    pub const fn base(&self) -> T
    where
        T: Copy,
    {
        self.base
    }

    /// Below every breakpoint, for a value that is not `Copy`.
    pub const fn base_ref(&self) -> &T {
        &self.base
    }

    /// The value in effect at `size`: the nearest breakpoint at or below it, else `base`.
    pub fn at(&self, size: Size) -> T
    where
        T: Clone,
    {
        self.breakpoints[..=size.index()]
            .iter()
            .rev()
            .find_map(Option::as_ref)
            .unwrap_or(&self.base)
            .clone()
    }

    /// The named breakpoints, ascending, so the widest matching query wins.
    pub fn breakpoints(&self) -> impl Iterator<Item = (Size, T)> + '_
    where
        T: Clone,
    {
        Size::ALL
            .into_iter()
            .zip(&self.breakpoints)
            .filter_map(|(size, value)| value.clone().map(|value| (size, value)))
    }

    /// Applies `f` to the base and every breakpoint.
    pub fn map<U>(&self, mut f: impl FnMut(T) -> U) -> Responsive<U>
    where
        T: Clone,
    {
        Responsive {
            base: f(self.base.clone()),
            breakpoints: self.breakpoints.clone().map(|value| value.map(&mut f)),
        }
    }
}

/// A per-breakpoint value, starting from `base`: `responsive(1).sm(2).lg(3)`.
pub const fn responsive<T>(base: T) -> Responsive<T> {
    Responsive::new(base)
}

impl<T> From<T> for Responsive<T> {
    fn from(base: T) -> Self {
        Self::new(base)
    }
}

/// What a caller would write: `2`, or `responsive(1).sm(2)`.
impl<T: Clone + fmt::Display> fmt::Display for Responsive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.breakpoints().next().is_none() {
            return write!(f, "{}", self.base);
        }
        write!(f, "responsive({})", self.base)?;
        for (size, value) in self.breakpoints() {
            write!(f, ".{}({value})", size.as_str())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_value_has_no_breakpoints() {
        let value = Responsive::from(3u8);

        assert_eq!(value.base(), 3);
        assert_eq!(value.breakpoints().count(), 0);
        assert_eq!(value.to_string(), "3");
    }

    #[test]
    fn re_declaring_a_size_replaces_it() {
        let value = responsive(1u8).md(2).md(3);

        assert_eq!(value.breakpoints().collect::<Vec<_>>(), [(Size::Md, 3)]);
    }

    #[test]
    fn breakpoints_come_out_ascending_whatever_the_call_order() {
        let value = responsive(1u8).lg(4).sm(2);

        assert_eq!(
            value.breakpoints().collect::<Vec<_>>(),
            [(Size::Sm, 2), (Size::Lg, 4)]
        );
        assert_eq!(value.to_string(), "responsive(1).sm(2).lg(4)");
    }

    /// Theme defaults and docs build these in a `const`.
    const GAP: Responsive<Size> = responsive(Size::Xs).md(Size::Lg).md(Size::Xl);

    #[test]
    fn a_copy_value_still_builds_in_a_const() {
        assert_eq!(GAP.base(), Size::Xs);
        assert_eq!(GAP.at(Size::Lg), Size::Xl);
    }

    #[test]
    fn an_owning_value_builds_and_re_declares() {
        let value = responsive("0".to_string())
            .sm("1rem".into())
            .sm("2rem".into());

        assert_eq!(value.base_ref(), "0");
        assert_eq!(value.at(Size::Md), "2rem");
        assert_eq!(value.map(|css| css.len()).at(Size::Sm), 4);
        assert_eq!(value.to_string(), "responsive(0).sm(2rem)");
    }

    #[test]
    fn only_a_re_declared_slot_leaks() {
        let shared = std::rc::Rc::new(());

        drop(
            responsive(shared.clone())
                .sm(shared.clone())
                .lg(shared.clone()),
        );
        assert_eq!(std::rc::Rc::strong_count(&shared), 1);

        drop(
            responsive(shared.clone())
                .sm(shared.clone())
                .sm(shared.clone()),
        );
        assert_eq!(std::rc::Rc::strong_count(&shared), 2);
    }

    #[test]
    fn at_falls_back_to_the_nearest_narrower_value() {
        let value = responsive(1u8).sm(2).lg(4);

        assert_eq!(value.at(Size::Xs), 1);
        assert_eq!(value.at(Size::Md), 2);
        assert_eq!(value.at(Size::Xxl), 4);
    }
}
