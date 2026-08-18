use super::Input;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

impl From<&str> for Orientation {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "horizontal" => Self::Horizontal,
            _ => Self::Vertical,
        }
    }
}

impl From<String> for Orientation {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<Orientation> {
    fn from(value: &str) -> Self {
        Input::Value(Orientation::from(value))
    }
}

impl From<String> for Input<Orientation> {
    fn from(value: String) -> Self {
        Input::Value(Orientation::from(value))
    }
}

impl From<Orientation> for Input<Orientation> {
    fn from(value: Orientation) -> Self {
        Input::Value(value)
    }
}
