/// Declares a string-parsed prop enum: the enum itself, the standard
/// derives, `ALL`/`as_str`/`state_name`, and `From<&str>`/`From<String>`.
///
/// An unrecognised string parses to the `#[default]` variant. Matching is
/// case-insensitive; a variant may list several accepted spellings, the
/// first of which is its canonical one (what `as_str` returns).
///
/// ```ignore
/// str_enum! {
///     /// How a control paints itself.
///     pub enum ButtonVariant {
///         #[default]
///         Filled = "filled",
///         Outlined = "outlined" | "outline",
///         Standard = "standard" | "text" | "transparent",
///     }
/// }
/// ```
///
/// `state_name` is the `data-state` token a component writes for the variant,
/// and what its `sx().when(..)` block keys on. It is `as_str` by default; an
/// optional `#[state_prefix = ".."]` prefixes it, for an enum whose bare
/// names would collide with another enum's on the same element:
///
/// ```ignore
/// str_enum! {
///     #[state_prefix = "fit"]
///     pub enum ImageFit {
///         #[default]
///         Fill = "fill",
///         ScaleDown = "scale-down" | "scaledown",
///     }
/// }
/// // ImageFit::ScaleDown.state_name() == "fit-scale-down"
/// ```
///
/// Pair with `input_from_str!` in the component taking the prop. Separate
/// macros because some of these live in `theme`, which can't reach up to
/// `components::Input`.
macro_rules! str_enum {
    // Joined at compile time: `States::with` needs a `&'static str`.
    (@state_name $name:ident; $prefix:literal; $($variant:ident => $pattern:literal,)+) => {
        impl $name {
            /// This variant's `data-state` token.
            pub const fn state_name(self) -> &'static str {
                match self {
                    $(Self::$variant => concat!($prefix, "-", $pattern),)+
                }
            }
        }
    };

    (@state_name $name:ident; ; $($variant:ident => $pattern:literal,)+) => {
        impl $name {
            /// This variant's `data-state` token.
            pub const fn state_name(self) -> &'static str {
                self.as_str()
            }
        }
    };

    (
        $(#[doc = $enum_doc:expr])*
        $(#[state_prefix = $prefix:literal])?
        pub enum $name:ident {
            $(
                $(#[doc = $variant_doc:expr])*
                $(#[default $($default:tt)?])?
                $variant:ident = $pattern:literal $(| $alias:literal)*,
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

        impl $name {
            /// Every variant, in declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            /// This variant's canonical spelling - the first of its accepted
            /// patterns.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $pattern,)+
                }
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                match value.to_lowercase().as_str() {
                    $($pattern $(| $alias)* => Self::$variant,)+
                    // The default renders something plausible, so a typo is
                    // otherwise invisible. `""` warns too - a prop is unset
                    // via `Input`'s `None`, not an empty string.
                    _ => {
                        $crate::utils::warn(&format!(
                            concat!(
                                stringify!($name),
                                ": unrecognized value {:?}, using {:?}. Accepted: {}."
                            ),
                            value,
                            Self::default().as_str(),
                            Self::ALL
                                .iter()
                                .map(|variant| format!("{:?}", variant.as_str()))
                                .collect::<Vec<_>>()
                                .join(", "),
                        ));
                        Self::default()
                    }
                }
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::from(value.as_str())
            }
        }

        str_enum!(@state_name $name; $($prefix)?; $($variant => $pattern,)+);
    };
}

pub(crate) use str_enum;

#[cfg(test)]
mod tests {

    use crate::components::{
        AnchorUnderline, ButtonVariant, DrawerAnchor, FlexDirection, FlexWrap, HeaderPosition,
        ImageFit, LabelPosition, Orientation, PinKind, SidebarSide,
    };
    use crate::theme::{
        LoaderVariant, Placement, QrRobustness, ScrollAxis, ScrollbarSize, ScrollbarVisibility,
    };

    /// Add every new `str_enum!` here - the checks below iterate this list.
    macro_rules! for_every_str_enum {
        ($check:ident) => {
            $check!(AnchorUnderline);
            $check!(ButtonVariant);
            $check!(DrawerAnchor);
            $check!(FlexDirection);
            $check!(FlexWrap);
            $check!(HeaderPosition);
            $check!(ImageFit);
            $check!(LabelPosition);
            $check!(LoaderVariant);
            $check!(Orientation);
            $check!(PinKind);
            $check!(Placement);
            $check!(QrRobustness);
            $check!(ScrollAxis);
            $check!(ScrollbarSize);
            $check!(ScrollbarVisibility);
            $check!(SidebarSide);
        };
    }

    /// A canonical spelling that no longer parses back means the enum and its
    /// string form have drifted apart.
    #[test]
    fn every_canonical_spelling_parses_back_to_its_own_variant() {
        macro_rules! check {
            ($ty:ty) => {
                for &variant in <$ty>::ALL {
                    assert_eq!(
                        <$ty>::from(variant.as_str()),
                        variant,
                        concat!(
                            stringify!($ty),
                            "::{:?} does not round-trip through as_str()"
                        ),
                        variant,
                    );
                }
            };
        }

        for_every_str_enum!(check);
    }

    /// The `sx()` folds iterate `ALL`, so a duplicate or missing variant
    /// silently corrupts that variant's `when(..)` block.
    #[test]
    fn all_lists_every_variant_exactly_once() {
        macro_rules! check {
            ($ty:ty) => {
                let mut seen = <$ty>::ALL.to_vec();
                let count = seen.len();
                seen.dedup();
                assert_eq!(
                    seen.len(),
                    count,
                    concat!(stringify!($ty), "::ALL has a duplicate")
                );
            };
        }

        for_every_str_enum!(check);
    }

    #[test]
    fn parses_case_insensitively_and_accepts_aliases() {
        assert_eq!(ButtonVariant::from("OUTLINED"), ButtonVariant::Outlined);
        assert_eq!(ButtonVariant::from("outline"), ButtonVariant::Outlined);
        // The lowest arm answers to M3's name and to both of ours.
        assert_eq!(ButtonVariant::from("text"), ButtonVariant::Standard);
        assert_eq!(ButtonVariant::from("transparent"), ButtonVariant::Standard);
    }

    #[test]
    fn an_unknown_string_falls_back_to_the_default_variant() {
        assert_eq!(ButtonVariant::from("nonsense"), ButtonVariant::default());
        assert_eq!(ButtonVariant::from(""), ButtonVariant::Filled);
    }

    #[test]
    fn as_str_is_the_canonical_spelling_not_an_alias() {
        assert_eq!(ImageFit::ScaleDown.as_str(), "scale-down");
        assert_eq!(ButtonVariant::Standard.as_str(), "standard");
    }

    #[test]
    fn state_name_is_as_str_unless_a_prefix_is_declared() {
        assert_eq!(ButtonVariant::Outlined.state_name(), "outlined");
        assert_eq!(ImageFit::ScaleDown.state_name(), "fit-scale-down");
    }
}
