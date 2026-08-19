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
/// A component can also ask for a specific tag's own attributes - e.g.
/// `option`'s `disabled`/`selected`, not part of `GlobalAttributes` - via a
/// leading `extends(...)` clause. Listing several tags works too (`Box`
/// extends `img`, `a` and `button`), but an attribute name two of them share
/// is ambiguous at the call site; declare it as an ordinary field to
/// disambiguate.
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
    (@build [$($extra_extends:tt)*]
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $($fields:tt)*
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Props, Clone, PartialEq)]
        $vis struct $name {
            #[props(extends = GlobalAttributes $($extra_extends)*)]
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

    (extends($($extra_extends:ident),+ $(,)?); $($rest:tt)*) => {
        base_props!(@build [$(, extends = $extra_extends)+] $($rest)*);
    };

    ($($rest:tt)*) => {
        base_props!(@build [] $($rest)*);
    };
}

pub(crate) use base_props;
