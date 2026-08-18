/// Declares a component `Props` struct, prepending the fields nearly every
/// visual component needs - `attributes` (`extends = GlobalAttributes`, so
/// it also captures any DOM event a caller writes), `class`, `sx`,
/// `states` - so the body only needs to list what's actually specific to
/// this component, instead of repeating these four every time.
///
/// Expands at the call site (ordinary `macro_rules!` path resolution, not
/// definition-site hygiene) - relies on `Props`/`Attribute` (from `dioxus::
/// prelude::*`) and `Input`/`States`/`Sx` already being in scope there,
/// which every component file needs anyway for its own fields (`ClassList`
/// is fully qualified here instead, since not every file has a reason to
/// import it on its own).
///
/// ```ignore
/// base_props! {
///     pub struct ContainerProps {
///         #[props(default, into)]
///         size: Input<ThemeAwareValue>,
///         children: Element,
///     }
/// }
/// ```
///
/// A component that always renders one specific tag (unlike `Box`, which
/// is polymorphic across many) can also ask for that tag's own
/// attributes - e.g. `option`'s `disabled`/`selected`, not part of
/// `GlobalAttributes` - via a leading `extends(...)` clause, instead of
/// hand-wiring them the way `Box` has to:
///
/// ```ignore
/// base_props! {
///     extends(option);
///     pub struct OptionProps {
///         #[props(into)]
///         value: String,
///         children: Element,
///     }
/// }
/// ```
macro_rules! base_props {
    (
        extends($($extra_extends:ident),+ $(,)?);
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $($fields:tt)*
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Props, Clone, PartialEq)]
        $vis struct $name {
            #[props(extends = GlobalAttributes, $(extends = $extra_extends),+)]
            attributes: Vec<Attribute>,
            #[props(default, into)]
            class: Input<crate::components::ClassList>,
            #[props(default, into)]
            sx: Input<Sx>,
            #[props(default, into)]
            states: Input<States>,
            $($fields)*
        }
    };

    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $($fields:tt)*
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Props, Clone, PartialEq)]
        $vis struct $name {
            #[props(extends = GlobalAttributes)]
            attributes: Vec<Attribute>,
            #[props(default, into)]
            class: Input<crate::components::ClassList>,
            #[props(default, into)]
            sx: Input<Sx>,
            #[props(default, into)]
            states: Input<States>,
            $($fields)*
        }
    };
}

pub(crate) use base_props;
