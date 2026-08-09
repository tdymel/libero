use crate::common::ConstVec;

use super::declaration::Declaration;

const DEFAULT_SX_DECLARATION_CAPACITY: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sx {
    declarations: ConstVec<Declaration, DEFAULT_SX_DECLARATION_CAPACITY>,
}

impl Default for Sx {
    fn default() -> Self {
        Self::new()
    }
}

impl Sx {
    pub const fn new() -> Self {
        Self {
            declarations: ConstVec::new_with_max_size(),
        }
    }

    pub const fn with(mut self, property: &'static str, value: &'static str) -> Self {
        self.declarations.push(Declaration { property, value });
        self
    }

    pub const fn background(self, value: &'static str) -> Self {
        self.with("background", value)
    }

    pub const fn width(self, value: &'static str) -> Self {
        self.with("width", value)
    }

    pub const fn height(self, value: &'static str) -> Self {
        self.with("height", value)
    }

    pub const fn declarations(&self) -> &[Declaration] {
        self.declarations.as_ref()
    }
}

pub const fn sx() -> Sx {
    Sx::new()
}

#[test]
fn builds_sx_with_common_properties_in_const_context() {
    const STYLE: Sx = sx().background("red").width("100px").height("200px");

    assert_eq!(
        STYLE.declarations(),
        &[
            Declaration {
                property: "background",
                value: "red",
            },
            Declaration {
                property: "width",
                value: "100px",
            },
            Declaration {
                property: "height",
                value: "200px",
            },
        ]
    );
}
