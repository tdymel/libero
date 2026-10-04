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

/// The `status` toggle and the `label`, `description` and `helper` switches; `label` off
/// prints `C::ARIA_LABEL` instead.
pub fn field_controls<C: FieldCopy>() -> Vec<Control> {
    vec![
        Control::toggle("status", ["valid", "warning", "error"])
            .labels(["Valid", "Warning", "Error"])
            .default("valid")
            .code(|_, values| match values.str("status").as_str() {
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
    ]
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
    pub helper: Option<String>,
    pub status: FieldStatus,
}

pub fn field_props<C: FieldCopy>(values: &DemoValues) -> FieldProps {
    let on = |name: &str| values.str(name) == "true";
    FieldProps {
        label: on("label").then(|| C::LABEL.to_string()),
        aria_label: (!on("label")).then_some(C::LABEL),
        description: on("description").then(|| C::DESCRIPTION.to_string()),
        helper: on("helper").then(|| C::HELPER.to_string()),
        status: match values.str("status").as_str() {
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
                r#"helper: "Markdown works.""#
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
}
