use crate::components::common::Input;

/// The validation state of a field, and the message that goes with it.
///
/// A variant keeps its text as given: `Error("")` still marks the field invalid and
/// blocks a submit, with nothing to read. Debug builds warn about a blank message.
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

    /// `Valid` only: false for a warning too, though only an error blocks a `Form`'s submit.
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

/// A bare message is an error; a warning has to be spelled out. An empty or
/// blank message is `Valid`, so `""` can stand for "all is well" (1525).
impl From<&str> for FieldStatus {
    fn from(message: &str) -> Self {
        Self::from(message.to_string())
    }
}

impl From<String> for FieldStatus {
    fn from(message: String) -> Self {
        match message.trim().is_empty() {
            true => Self::Valid,
            false => Self::Error(message),
        }
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

/// `None` is `Valid`, so a server's `Option` answer goes straight in (2343).
impl From<Option<FieldStatus>> for Input<FieldStatus> {
    fn from(status: Option<FieldStatus>) -> Self {
        status.map_or(Input::None, Input::Value)
    }
}

impl From<Option<String>> for Input<FieldStatus> {
    fn from(message: Option<String>) -> Self {
        message.map_or(Input::None, Self::from)
    }
}

impl From<Option<&str>> for Input<FieldStatus> {
    fn from(message: Option<&str>) -> Self {
        message.map_or(Input::None, Self::from)
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
        // Todo 2344: a warning is not `Valid`, as the doc says.
        assert!(!warning.is_valid());

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
    fn an_empty_or_blank_message_is_valid() {
        assert_eq!(FieldStatus::from(""), FieldStatus::Valid);
        assert_eq!(FieldStatus::from(" \n".to_string()), FieldStatus::Valid);
        let status: Input<FieldStatus> = "".into();
        assert_eq!(status.as_ref(), Some(&FieldStatus::Valid));
    }

    #[test]
    fn none_and_a_blank_option_are_valid() {
        let unset: [Input<FieldStatus>; 3] = [
            None::<FieldStatus>.into(),
            None::<String>.into(),
            None::<&str>.into(),
        ];
        for status in unset {
            assert!(status.unwrap_or_default().is_valid());
        }
        let blank: Input<FieldStatus> = Some(" ").into();
        assert_eq!(blank.as_ref(), Some(&FieldStatus::Valid));
        let server: Input<FieldStatus> = Some("taken".to_string()).into();
        assert_eq!(
            server.as_ref(),
            Some(&FieldStatus::Error("taken".to_string()))
        );
        let warning: Input<FieldStatus> = Some(FieldStatus::Warning("weak".to_string())).into();
        assert!(warning.as_ref().is_some_and(FieldStatus::is_warning));
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
