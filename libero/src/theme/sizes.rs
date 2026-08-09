use super::Size;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sizes<T> {
    pub xs: T,
    pub sm: T,
    pub md: T,
    pub lg: T,
    pub xl: T,
}

impl<T> Sizes<T> {
    pub const fn new(xs: T, sm: T, md: T, lg: T, xl: T) -> Self {
        Self { xs, sm, md, lg, xl }
    }
}

impl<T: Copy> Sizes<T> {
    pub const fn get(&self, size: Size) -> T {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_happy_path() {
        const SIZES: Sizes<u8> = Sizes::new(1, 2, 3, 4, 5);

        assert_eq!(SIZES.get(Size::Xs), 1);
        assert_eq!(SIZES.get(Size::Md), 3);
        assert_eq!(SIZES.get(Size::Xl), 5);
    }
}
