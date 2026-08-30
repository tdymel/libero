/// Declares a component `Props` struct, appending `attributes` (extends
/// `GlobalAttributes`, so it also captures DOM events), `class`, `sx` and
/// `states`. The body lists only what's specific to the component.
///
/// Expands at the call site, so `Props`, `Attribute` and `Input` must be in
/// scope there - every component file imports them anyway. The shared fields'
/// own types are named by path, so a component that does not otherwise mention
/// `Sx` or `States` need not import them.
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
/// A generic struct declares its parameters inline, one bound each - a `where`
/// clause and a `A + B` bound are both unsupported, and nothing has wanted
/// either. Capturing the parameter list as `tt` instead is the
/// `local ambiguity when calling macro` trap, since a `tt` repetition cannot
/// be followed by `>`.
///
/// ```ignore
/// base_props! {
///     pub struct SliderProps<V: SliderValue> {
///         value: V,
///     }
/// }
/// ```
///
/// A leading `extends(...)` adds a specific tag's own attributes (e.g.
/// `option`'s `selected`). Several tags work, but an attribute two of them
/// share is ambiguous at the call site - declare it as a field instead.
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
        $vis:vis struct $name:ident $(< $($generic:ident : $bound:path),+ $(,)? >)? {
            $($fields:tt)*
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Props, Clone, PartialEq)]
        $vis struct $name $(< $($generic: $bound),+ >)? {
            // The component's own fields come first, and `children` with them,
            // so the derived `PartialEq` that `memoize` runs bails on the
            // never-equal `children` before it compares the four expensive
            // shared fields below.
            $($fields)*
            #[props(extends = GlobalAttributes $($extra_extends)*)]
            attributes: Vec<Attribute>,
            #[props(default, into)]
            class: Input<crate::components::ClassList>,
            #[props(default, into)]
            sx: Input<crate::sx::Sx>,
            #[props(default, into)]
            states: Input<crate::components::States>,
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
