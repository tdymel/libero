#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JustifyInput {
    Default,
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Other(String),
}

impl Default for JustifyInput {
    fn default() -> Self {
        Self::Default
    }
}

impl JustifyInput {
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Default => None,
            Self::Start => Some("flex-start"),
            Self::End => Some("flex-end"),
            Self::Center => Some("center"),
            Self::SpaceBetween => Some("space-between"),
            Self::SpaceAround => Some("space-around"),
            Self::SpaceEvenly => Some("space-evenly"),
            Self::Other(value) => Some(value.as_str()),
        }
    }
}

impl From<&str> for JustifyInput {
    fn from(value: &str) -> Self {
        match value {
            "flex-start" | "start" => Self::Start,
            "flex-end" | "end" => Self::End,
            "center" => Self::Center,
            "space-between" => Self::SpaceBetween,
            "space-around" => Self::SpaceAround,
            "space-evenly" => Self::SpaceEvenly,
            _ => Self::Other(value.to_string()),
        }
    }
}

impl From<String> for JustifyInput {
    fn from(value: String) -> Self {
        match value.as_str() {
            "flex-start" | "start" => Self::Start,
            "flex-end" | "end" => Self::End,
            "center" => Self::Center,
            "space-between" => Self::SpaceBetween,
            "space-around" => Self::SpaceAround,
            "space-evenly" => Self::SpaceEvenly,
            _ => Self::Other(value),
        }
    }
}
