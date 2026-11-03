use libero::components::FieldStatus;

use super::{Control, DemoValues};

/// A field demo's captions and messages. Consts, so the controls' `fn` code can print them.
pub trait FieldCopy {
    const LABEL: &'static str;
    const DESCRIPTION: &'static str;
    const HELPER: &'static str;
    const WARNING: &'static str;
    const ERROR: &'static str;
    /// What names an unlabelled field; `"aria-label"` (quoted) where it has no `aria_label` prop.
    const ARIA_LABEL: &'static str = "aria_label";
}

/// The `status` toggle, the `label`, `description` and `helper` switches and the
/// `description_position` toggle; `label` off prints `C::ARIA_LABEL` instead.
pub fn field_controls<C: FieldCopy>() -> Vec<Control> {
    vec![
        Control::toggle("status", ["valid", "pending", "warning", "error"])
            .labels(["Valid", "Pending", "Warning", "Error"])
            .default("valid")
            .code(|_, values| match values.str("status").as_str() {
                "pending" => vec!["status: FieldStatus::Pending".to_string()],
                "warning" => vec![format!(
                    "status: FieldStatus::Warning({:?}.into())",
                    C::WARNING
                )],
                "error" => vec![format!("status: {:?}", C::ERROR)],
                _ => vec![],
            }),
        Control::switch("label").default("true").code(|_, values| {
            match values.str("label") == "true" {
                true => vec![format!("label: {:?}", C::LABEL)],
                false => vec![format!("{}: {:?}", C::ARIA_LABEL, C::LABEL)],
            }
        }),
        Control::switch("description")
            .code(|_, values| caption(values, "description", C::DESCRIPTION)),
        Control::switch("helper").code(|_, values| caption(values, "helper", C::HELPER)),
        description_position_control(),
    ]
}

/// The `description_position` toggle, shown once `description` is on.
pub fn description_position_control() -> Control {
    Control::toggle("description_position", ["above", "below"])
        .labels(["Above", "Below"])
        .default("above")
        .hidden_when(|values| values.str("description") != "true")
        .code(|_, values| {
            let below = values.str("description_position") == "below";
            match below && values.str("description") == "true" {
                true => vec![r#"description_position: "below""#.to_string()],
                false => vec![],
            }
        })
}

/// [`field_controls`] without `description_position`, for a field that draws its
/// description under the label either way.
pub fn inline_field_controls<C: FieldCopy>() -> Vec<Control> {
    let mut controls = field_controls::<C>();
    controls.retain(|control| control.name != "description_position");
    controls
}

fn caption(values: &DemoValues, name: &str, text: &str) -> Vec<String> {
    match values.str(name) == "true" {
        true => vec![format!("{name}: {text:?}")],
        false => vec![],
    }
}

/// What `field_controls` set, for the preview.
pub struct FieldProps {
    pub label: Option<String>,
    pub aria_label: Option<&'static str>,
    pub description: Option<String>,
    pub description_position: String,
    pub helper: Option<String>,
    pub status: FieldStatus,
}

pub fn field_props<C: FieldCopy>(values: &DemoValues) -> FieldProps {
    let on = |name: &str| values.str(name) == "true";
    FieldProps {
        label: on("label").then(|| C::LABEL.to_string()),
        aria_label: (!on("label")).then_some(C::LABEL),
        description: on("description").then(|| C::DESCRIPTION.to_string()),
        description_position: values.str("description_position"),
        helper: on("helper").then(|| C::HELPER.to_string()),
        status: match values.str("status").as_str() {
            "pending" => FieldStatus::Pending,
            "warning" => FieldStatus::Warning(C::WARNING.to_string()),
            "error" => FieldStatus::Error(C::ERROR.to_string()),
            _ => FieldStatus::Valid,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Notes;

    impl FieldCopy for Notes {
        const LABEL: &'static str = "Notes";
        const DESCRIPTION: &'static str = "What changed.";
        const HELPER: &'static str = "Markdown works.";
        const WARNING: &'static str = "Long.";
        const ERROR: &'static str = "Write a few words.";
    }

    #[test]
    fn field_controls_print_what_field_props_set() {
        let controls = field_controls::<Notes>();
        let values = DemoValues::defaults(&controls)
            .with("label", "false")
            .with("helper", "true")
            .with("description", "true")
            .with("description_position", "below")
            .with("status", "error");
        let code: Vec<String> = controls
            .iter()
            .flat_map(|control| (control.code.unwrap())(control, &values))
            .collect();
        assert_eq!(
            code,
            [
                r#"status: "Write a few words.""#,
                r#"aria_label: "Notes""#,
                r#"description: "What changed.""#,
                r#"helper: "Markdown works.""#,
                r#"description_position: "below""#
            ]
        );
        let props = field_props::<Notes>(&values);
        assert_eq!(props.label, None);
        assert_eq!(props.aria_label, Some("Notes"));
        assert_eq!(props.helper.as_deref(), Some("Markdown works."));
        assert_eq!(
            props.status,
            FieldStatus::Error("Write a few words.".to_string())
        );
    }

    /// Todo 5402: Above/Below does nothing without a description, so it hides until one is on.
    #[test]
    fn description_position_shows_only_with_a_description() {
        let controls = field_controls::<Notes>();
        let position = controls
            .iter()
            .find(|control| control.name == "description_position")
            .unwrap();
        let values = DemoValues::defaults(&controls);
        assert!(position.is_hidden(&values));
        assert!(!position.is_hidden(&values.with("description", "true")));
    }

    #[test]
    fn pending_status_prints_and_sets_pending() {
        let controls = field_controls::<Notes>();
        let values = DemoValues::defaults(&controls).with("status", "pending");
        let status = &controls[0];
        assert_eq!(
            (status.code.unwrap())(status, &values),
            ["status: FieldStatus::Pending"]
        );
        assert_eq!(field_props::<Notes>(&values).status, FieldStatus::Pending);
    }
}
