//! The props every date and time component shares, declared once. A field or
//! a picker names its value type, its `min`/`max` type and the groups of props
//! it uses; the declarations and their docs come from here.

/// Declares a date or time component's props.
///
/// ```ignore
/// # // Not compiled: `date_props!` is crate-internal, so a doc-test cannot name it.
/// date_props! {
///     field DayFieldProps(NaiveDate, NaiveDate): format, limits, exclude_date, today, close_on_change
/// }
/// ```
///
/// `field` builds on `field_props!` and adds `value`, `onchange`, `validate`,
/// `name` and `placeholder`. `picker` builds on `base_props!` and adds `value`,
/// `onchange`, `size`, `name` and `focusable`. The groups are `format`,
/// `time_format`, `limits` (`min`, `max`), `exclude_date`, `today`, `clock`
/// (`variant`, `with_seconds`, `step`, `twelve_hour`), `calendar` (`calendar`,
/// `days`), `columns`,
/// `close_on_change`, `allow_deselect` and `level`.
///
/// The docs are written for every value type at once. What is special about
/// one type goes in its component's doc comment.
macro_rules! date_props {
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] format $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// How the text shows a day, in dayjs tokens. Defaults to the
            /// theme's `DateDefaults::format`. Typing is lenient either way:
            /// only the order of day, month and year has to match.
            #[props(default, into)]
            format: Option<String>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] time_format $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// How the text shows a time, in dayjs tokens. Defaults to the
            /// theme's `DateDefaults::time_format`, adjusted for
            /// `with_seconds` and `twelve_hour`.
            #[props(default, into)]
            time_format: Option<String>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt ($v:ty, $b:ty) [$($acc:tt)*] limits $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head ($v, $b) [$($acc)*
            /// The earliest value accepted - for a range, its earliest end.
            #[props(default)]
            min: Option<$b>,
            /// The latest value accepted - for a range, its latest end.
            #[props(default)]
            max: Option<$b>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] exclude_date $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// Days that are not accepted, on top of `min` and `max`. Ignored
            /// for a time, a month and a year.
            #[props(default)]
            exclude_date: Option<Callback<::chrono::NaiveDate, bool>>,
        ] $($rest)*);
    };
    (@munch field $head:tt $types:tt [$($acc:tt)*] today $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch field $head $types [$($acc)*
            /// The day marked as today, and the year typed text without one
            /// falls back to. Unset, the platform clock answers after mount.
            #[props(default)]
            today: Option<::chrono::NaiveDate>,
        ] $($rest)*);
    };
    (@munch picker $head:tt $types:tt [$($acc:tt)*] today $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch picker $head $types [$($acc)*
            /// The day marked as today. Unset, the platform clock answers after
            /// mount - on the web; elsewhere no day is marked.
            #[props(default)]
            today: Option<::chrono::NaiveDate>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] clock $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// Columns of numbers, or a clock face. Defaults to the theme's
            /// `TimePickerDefaults::variant`.
            #[props(default, into)]
            variant: $crate::components::Input<$crate::theme::TimePickerVariant>,
            /// Seconds in the text and on the clock.
            #[props(default)]
            with_seconds: Option<bool>,
            /// Minutes between the offered minutes. Defaults to the theme's
            /// `TimePickerDefaults::step`, 5.
            #[props(default)]
            step: Option<u8>,
            /// A 12-hour clock with AM and PM. Defaults to whether the time
            /// format is one.
            #[props(default)]
            twelve_hour: Option<bool>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] calendar $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// A month of days, or one row of days with buttons that page it.
            /// Defaults to the theme's `DatePickerDefaults::calendar`. Ignored
            /// for times, ranges, months and years.
            #[props(default, into)]
            calendar: $crate::components::Input<$crate::theme::CalendarVariant>,
            /// Days in the mini calendar's row. Defaults to the theme's
            /// `DatePickerDefaults::days`.
            #[props(default)]
            days: Option<usize>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] columns $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// Months side by side. One by default, two for a range.
            #[props(default)]
            columns: Option<usize>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] close_on_change $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// A pick that leaves nothing more to pick closes the dropdown.
            /// Defaults to the theme's `DateFieldDefaults::close_on_change`.
            #[props(default)]
            close_on_change: Option<bool>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] allow_deselect $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// Clicking the picked day again clears it.
            #[props(default)]
            allow_deselect: Option<bool>,
        ] $($rest)*);
    };
    (@munch $kind:ident $head:tt $types:tt [$($acc:tt)*] level $($rest:ident)*) => {
        $crate::components::form::date::props::date_props!(@munch $kind $head $types [$($acc)*
            /// Whether a `NaiveDate` is picked as a day, a month or a year.
            /// Days by default.
            #[props(default)]
            level: Option<$crate::components::form::date::DateLevel>,
        ] $($rest)*);
    };
    (@munch field [$name:ident $(< $generic:ident : $bound:path >)?] ($v:ty, $b:ty) [$($acc:tt)*]) => {
        $crate::components::common::field_props! {
            extends(input);
            pub struct $name $(< $generic: $bound >)? {
                /// The value in the field; strictly controlled. `None` is the
                /// empty field. Inside a `Form`, a path `name` can supply it
                /// instead.
                #[props(default)]
                value: Option<$v>,
                /// Called with the value the caller should hold next: when
                /// typed text is committed - on blur or Enter - and on every
                /// pick. Emptied text commits `None`.
                #[props(default)]
                onchange: Option<EventHandler<Option<$v>>>,
                /// Rules over the value, shown once the field loses focus or
                /// its form is submitted.
                #[props(default, into)]
                validate: $crate::components::Validators<Option<$v>>,
                $($acc)*
                /// What the field posts as - the value in ISO 8601, whatever
                /// the text shows. A path also binds it to the surrounding
                /// `Form`'s value when it has no `onchange`.
                #[props(default, into)]
                name: $crate::components::FieldName<Option<$v>>,
                #[props(default, into)]
                placeholder: Option<String>,
            }
        }
    };
    (@munch picker [$name:ident $(< $generic:ident : $bound:path >)?] ($v:ty, $b:ty) [$($acc:tt)*]) => {
        $crate::components::common::base_props! {
            pub struct $name $(< $generic: $bound >)? {
                /// The picked value; strictly controlled. `None` picks nothing.
                #[props(default)]
                value: Option<$v>,
                /// Called with the value the caller should hold next.
                #[props(default)]
                onchange: Option<EventHandler<Option<$v>>>,
                $($acc)*
                #[props(default, into)]
                size: $crate::components::Input<$crate::theme::Size>,
                /// Emits a hidden input of that name, posting the value as ISO
                /// 8601.
                #[props(default, into)]
                name: Option<String>,
                /// `false` keeps the picker out of the tab order - for a picker
                /// inside a dropdown whose text input must keep focus. On by
                /// default.
                #[props(default)]
                focusable: Option<bool>,
            }
        }
    };
    ($kind:ident $name:ident $(< $generic:ident : $bound:path >)? ($v:ty, $b:ty): $($group:ident),* $(,)?) => {
        $crate::components::form::date::props::date_props!(
            @munch $kind [$name $(< $generic : $bound >)?] ($v, $b) [] $($group)*
        );
    };
}

pub(super) use date_props;
