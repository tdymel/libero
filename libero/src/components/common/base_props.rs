/// Declares a component `Props` struct, appending `attributes`, `class`, `sx` and `states`.
/// `Props`, `Attribute` and `Input` must be in scope at the call site.
///
/// ```ignore
/// # // Not compiled: `base_props!` is crate-internal, so a doc-test cannot name it.
/// base_props! {
///     pub struct ContainerProps {
///         #[props(default, into)]
///         size: Input<ThemeAwareValue>,
///         children: Element,
///     }
/// }
/// ```
///
/// Generics inline, one bound each: no `where`, no `A + B`. A `tt` capture would hit the
/// `local ambiguity when calling macro` trap.
///
/// ```ignore
/// # // Not compiled: `base_props!` is crate-internal, so a doc-test cannot name it.
/// base_props! {
///     pub struct SliderProps<V: SliderValue> {
///         value: V,
///     }
/// }
/// ```
///
/// `extends(...)` adds a tag's own attributes. An attribute two tags share is ambiguous:
/// declare it as a field instead.
///
/// ```ignore
/// # // Not compiled: `base_props!` is crate-internal, so a doc-test cannot name it.
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
        $vis:vis struct $name:ident $(< $($generic:ident : $bound:path),+ $(,)? >)? {
            $($fields:tt)*
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Props, Clone, PartialEq)]
        $vis struct $name $(< $($generic: $bound),+ >)? {
            // Own fields first: `PartialEq` bails on the never-equal `children`
            // before comparing the four expensive shared fields.
            $($fields)*
            #[props(extends = GlobalAttributes $($extra_extends)*)]
            attributes: Vec<Attribute>,
            #[props(default, into)]
            class: Input<crate::components::common::ClassList>,
            #[props(default, into)]
            sx: Input<crate::sx::Sx>,
            #[props(default, into)]
            states: Input<crate::components::common::States>,
        }
    };

    (extends($($extra_extends:ident),+ $(,)?); $($rest:tt)*) => {
        crate::components::common::base_props!(@build [$(, extends = $extra_extends)+] $($rest)*);
    };

    ($($rest:tt)*) => {
        crate::components::common::base_props!(@build [] $($rest)*);
    };
}

pub(crate) use base_props;
