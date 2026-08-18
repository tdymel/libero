/// Declares a string-parsed prop enum: the enum itself, the standard
/// derives, and `From<&str>`/`From<String>`.
///
/// An unrecognised string parses to the `#[default]` variant. Matching is
/// case-insensitive; a variant may list several accepted spellings.
///
/// ```ignore
/// str_enum! {
///     /// How an `Icon` paints itself.
///     pub enum IconVariant {
///         #[default]
///         Filled = "filled",
///         Outlined = "outlined" | "outline",
///         Transparent = "transparent",
///     }
/// }
/// ```
///
/// Pair it with `input_from_str!` in the component that takes the enum as a
/// prop - the two are separate because some of these live in `theme`, which
/// can't reach up to `components::Input`.
macro_rules! str_enum {
    (
        $(#[doc = $enum_doc:expr])*
        pub enum $name:ident {
            $(
                $(#[doc = $variant_doc:expr])*
                $(#[default $($default:tt)?])?
                $variant:ident = $($pattern:literal)|+,
            )+
        }
    ) => {
        $(#[doc = $enum_doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
        pub enum $name {
            $(
                $(#[doc = $variant_doc])*
                $(#[default $($default)?])?
                $variant,
            )+
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                match value.to_lowercase().as_str() {
                    $($($pattern)|+ => Self::$variant,)+
                    _ => Self::default(),
                }
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::from(value.as_str())
            }
        }
    };
}

pub(crate) use str_enum;

#[cfg(test)]
mod tests {
    use crate::components::{ButtonVariant, IconVariant};

    #[test]
    fn parses_case_insensitively_and_accepts_aliases() {
        assert_eq!(IconVariant::from("OUTLINED"), IconVariant::Outlined);
        assert_eq!(IconVariant::from("outline"), IconVariant::Outlined);
    }

    #[test]
    fn an_unknown_string_falls_back_to_the_default_variant() {
        assert_eq!(IconVariant::from("nonsense"), IconVariant::default());
        assert_eq!(ButtonVariant::from(""), ButtonVariant::Outlined);
    }
}
