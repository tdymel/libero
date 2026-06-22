use crate::{SxDyn, use_sx};

pub fn use_resolved_class<BaseSx, OverrideSx>(
    base_sx: BaseSx,
    override_sx: Option<OverrideSx>,
    user_class: Option<String>,
) -> String
where
    BaseSx: Into<SxDyn> + Clone + PartialEq + 'static,
    OverrideSx: Into<SxDyn> + Clone + PartialEq + 'static,
{
    let base_class = use_sx(base_sx);
    let sx_class = override_sx.map(use_sx);
    let user_class = user_class.unwrap_or_default();

    match sx_class {
        Some(sx_class) if user_class.is_empty() => format!("{} {}", base_class, sx_class),
        Some(sx_class) => format!("{} {} {}", base_class, sx_class, user_class),
        None if user_class.is_empty() => base_class,
        None => format!("{} {}", base_class, user_class),
    }
}
