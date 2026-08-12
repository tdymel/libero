use std::ops::Deref;
use std::sync::LazyLock;

use super::Sx;

pub struct StaticSx(LazyLock<Sx>);

impl std::fmt::Debug for StaticSx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StaticSx").finish()
    }
}

impl PartialEq for StaticSx {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl StaticSx {
    pub const fn new(init: fn() -> Sx) -> Self {
        Self(LazyLock::new(init))
    }
}

impl Deref for StaticSx {
    type Target = Sx;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
