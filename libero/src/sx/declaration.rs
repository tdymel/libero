#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Declaration {
    pub property: &'static str,
    pub value: &'static str,
}
