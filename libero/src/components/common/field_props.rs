/// Declares a field's `Props` struct: everything [`base_props`] emits, plus the
/// eight props every field shares - the four text slots, the validation status,
/// `size`, `radius`, `disabled` and `required`. The body lists only what is
/// specific to the field.
///
/// ```ignore
/// field_props! {
///     extends(input);
///     pub struct TextFieldProps {
///         #[props(default, into)]
///         value: Option<String>,
///     }
/// }
/// ```
///
/// It deliberately does **not** emit `value` or a change handler: the value
/// type differs per field (`String`, `bool`, `f64`, `T`, `Vec<T>`), and a macro
/// cannot emit a type it does not know. Nor `placeholder` - six of the planned
/// fields have none, and a prop that silently does nothing is worse than one
/// repeated line.
///
/// Like [`base_props`], the leading `extends(..)` clause needs the internal
/// `@build` arm plus two dispatch arms: `$(extends(..);)?` before the struct is
/// a `local ambiguity when calling macro` error, since both can match empty.
macro_rules! field_props {
    (@build [$($extra_extends:tt)*]
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
                label: crate::components::Caption,
                /// Between the label and the control. What to enter.
                #[props(default, into)]
                description: crate::components::Caption,
                /// Under the control. Formatting rules, constraints, counters.
                #[props(default, into)]
                helper: crate::components::Caption,
                /// Validation state, under the helper. A bare `&str` is an
                /// error.
                #[props(default, into)]
                status: Input<crate::components::FieldStatus>,
                #[props(default, into)]
                size: Input<crate::theme::Size>,
                /// Corner radius, independent of `size`.
                #[props(default, into)]
                radius: Input<crate::theme::Size>,
                /// `None` is "not stated" - what a `Fieldset` will cascade
                /// into later.
                #[props(default)]
                disabled: Option<bool>,
                #[props(default)]
                required: Option<bool>,
                /// Focusable and posted with the form, but not editable.
                /// `disabled` instead drops the field from the tab order and
                /// from the post, which is wrong for a review-your-answers
                /// view. `None` is "not stated".
                #[props(default)]
                readonly: Option<bool>,
            }
        }
    };

    (extends($($extra_extends:ident),+ $(,)?); $($rest:tt)*) => {
        crate::components::common::field_props!(@build [$(, extends = $extra_extends)+] $($rest)*);
    };

    ($($rest:tt)*) => {
        crate::components::common::field_props!(@build [] $($rest)*);
    };
}

pub(crate) use field_props;
