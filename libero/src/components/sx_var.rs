#[macro_export]
macro_rules! sx_var {
    ($name:literal) => {
        concat!("var(--lsx-", $name, ")")
    };
}
