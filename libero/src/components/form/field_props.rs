/// A field's `Props` struct: [`base_props`](crate::components::common::base_props)
/// plus the props every field shares. No `value`, handler or `placeholder`.
///
/// ```ignore
/// # // Not compiled: `field_props!` is crate-internal, so a doc-test cannot name it.
/// field_props! {
///     extends(input);
///     pub struct TextFieldProps {
///         #[props(default, into)]
///         value: Option<String>,
///     }
/// }
/// ```
///
/// `without(readonly);` (not with `extends`) and `without(radius);` drop those
/// props. Separate dispatch arms: an optional `extends` is a macro ambiguity.
macro_rules! field_props {
    (@build [$($extra_extends:tt)*] [$($readonly:ident)?] [$($radius:ident)?]
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident $(< $($generic:ident : $bound:path),+ $(,)? >)? {
            $($fields:tt)*
        }
    ) => {
        crate::components::common::base_props! {
            @build [$($extra_extends)*]
            $(#[$struct_meta])*
            $vis struct $name $(< $($generic: $bound),+ >)? {
                $($fields)*
                /// The field's caption, above the control.
                #[props(default, into)]
                label: crate::components::form::Caption,
                /// Between the label and the control. What to enter.
                #[props(default, into)]
                description: crate::components::form::Caption,
                /// Under the control. Formatting rules, constraints, counters.
                #[props(default, into)]
                helper: crate::components::form::Caption,
                /// Validation state, under the helper. A bare `&str` is an error.
                #[props(default, into)]
                status: Input<crate::components::form::FieldStatus>,
                #[props(default, into)]
                size: Input<crate::theme::Size>,
                $(
                    /// Corner radius, independent of `size`.
                    #[props(default, into)]
                    $radius: Input<crate::theme::Size>,
                )?
                /// `None` is "not stated", so a `Fieldset` can cascade into it.
                #[props(default)]
                disabled: Option<bool>,
                #[props(default)]
                required: Option<bool>,
                $(
                    /// Focusable and posted, but not editable. `None` is "not stated".
                    #[props(default)]
                    $readonly: Option<bool>,
                )?
            }
        }
    };

    (extends($($extra_extends:ident),+ $(,)?); without(radius); $($rest:tt)*) => {
        crate::components::form::field_props!(@build [$(, extends = $extra_extends)+] [readonly] [] $($rest)*);
    };

    (extends($($extra_extends:ident),+ $(,)?); $($rest:tt)*) => {
        crate::components::form::field_props!(@build [$(, extends = $extra_extends)+] [readonly] [radius] $($rest)*);
    };

    (without(readonly); $($rest:tt)*) => {
        crate::components::form::field_props!(@build [] [] [radius] $($rest)*);
    };

    (without(radius); $($rest:tt)*) => {
        crate::components::form::field_props!(@build [] [readonly] [] $($rest)*);
    };

    ($($rest:tt)*) => {
        crate::components::form::field_props!(@build [] [readonly] [radius] $($rest)*);
    };
}

pub(crate) use field_props;
