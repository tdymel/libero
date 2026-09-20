use crate::components::common::Input;

/// The validation state of a field, and the message that goes with it.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum FieldStatus {
    #[default]
    Valid,
    Warning(String),
    Error(String),
}

impl FieldStatus {
    /// The message carried by the status, if any. `Valid` never has one.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Valid => None,
            Self::Warning(message) | Self::Error(message) => Some(message),
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// The `data-state` name for the status; `None` for `Valid`.
    pub fn state(&self) -> Option<&'static str> {
        match self {
            Self::Valid => None,
            Self::Warning(_) => Some("warning"),
            Self::Error(_) => Some("error"),
        }
    }
}

/// A bare message is an error; a warning has to be spelled out.
impl From<&str> for FieldStatus {
    fn from(message: &str) -> Self {
        Self::Error(message.to_string())
    }
}

impl From<String> for FieldStatus {
    fn from(message: String) -> Self {
        Self::Error(message)
    }
}

impl From<FieldStatus> for Input<FieldStatus> {
    fn from(value: FieldStatus) -> Self {
        Input::Value(value)
    }
}

impl From<&str> for Input<FieldStatus> {
    fn from(message: &str) -> Self {
        Input::Value(FieldStatus::from(message))
    }
}

impl From<String> for Input<FieldStatus> {
    fn from(message: String) -> Self {
        Input::Value(FieldStatus::from(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_valid() {
        assert_eq!(FieldStatus::default(), FieldStatus::Valid);
        assert!(FieldStatus::default().is_valid());
    }

    #[test]
    fn valid_has_no_message_and_no_state() {
        assert_eq!(FieldStatus::Valid.message(), None);
        assert_eq!(FieldStatus::Valid.state(), None);
    }

    #[test]
    fn message_and_state_follow_the_variant() {
        let warning = FieldStatus::Warning("close".to_string());
        assert_eq!(warning.message(), Some("close"));
        assert_eq!(warning.state(), Some("warning"));
        assert!(warning.is_warning());

        let error = FieldStatus::Error("too short".to_string());
        assert_eq!(error.message(), Some("too short"));
        assert_eq!(error.state(), Some("error"));
        assert!(error.is_error());
    }

    #[test]
    fn a_bare_message_becomes_an_error() {
        assert_eq!(
            FieldStatus::from("required"),
            FieldStatus::Error("required".to_string())
        );
        assert_eq!(
            FieldStatus::from("required".to_string()),
            FieldStatus::Error("required".to_string())
        );
    }

    #[test]
    fn a_bare_message_reaches_the_prop_type() {
        let status: Input<FieldStatus> = "required".into();
        assert_eq!(
            status.as_ref(),
            Some(&FieldStatus::Error("required".to_string()))
        );
    }
}
