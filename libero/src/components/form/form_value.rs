/// What a `Form` or `Fieldset` can hold as its value.
pub trait FormValue: Clone + PartialEq + Default + 'static {}

impl<T: Clone + PartialEq + Default + 'static> FormValue for T {}
