//! Choices over a type the app can't derive `Options` for: an enum from a domain or third-party crate.

use std::{borrow::Cow, ops::Deref};

use super::{OptionList, Options};

/// One entry of a [`Choices`] list: the domain value, the key a form posts, and its label.
///
/// It derefs to the value, so a render callback reads the domain type directly.
#[derive(Clone)]
pub struct Choice<V> {
    pub value: V,
    key: Cow<'static, str>,
    label: Label<V>,
}

/// A label fn runs at render, so an app's i18n follows a language switch; text is fixed.
enum Label<V> {
    Fn(fn(&V) -> String),
    Text(Cow<'static, str>),
}

impl<V> Clone for Label<V> {
    fn clone(&self) -> Self {
        match self {
            Self::Fn(label) => Self::Fn(*label),
            Self::Text(text) => Self::Text(text.clone()),
        }
    }
}

impl<V> Choice<V> {
    /// A choice whose label is already known text, for lists built inside libero.
    pub(crate) fn text(
        value: V,
        key: impl Into<Cow<'static, str>>,
        label: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            value,
            key: key.into(),
            label: Label::Text(label.into()),
        }
    }

    /// What a form posts for this choice.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl<V> Deref for Choice<V> {
    type Target = V;

    fn deref(&self) -> &V {
        &self.value
    }
}

/// Identity is the value and the key; fixed text counts too, so a relabelled list redraws.
impl<V: PartialEq> PartialEq for Choice<V> {
    fn eq(&self, other: &Self) -> bool {
        let text = |label: &Label<V>| match label {
            Label::Fn(_) => None,
            Label::Text(text) => Some(text.clone()),
        };
        self.value == other.value
            && self.key == other.key
            && text(&self.label) == text(&other.label)
    }
}

impl<V: Clone + PartialEq + 'static> Options for Choice<V> {
    fn label(&self) -> String {
        match &self.label {
            Label::Fn(label) => label(&self.value),
            Label::Text(text) => text.to_string(),
        }
    }

    fn value(&self) -> String {
        self.key.to_string()
    }
}

/// An ordered list of a foreign type's values, each with a unique key, labelled by one fn.
///
/// Build an enum's list with [`choices!`](crate::choices), which fails the build when the
/// enum gains a variant; [`Choices::of`] is for lists that arrive at runtime.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// use libero::{choices, components::{Choice, Choices, NativeSelect}};
///
/// // From a domain crate: no `Options` derive there.
/// #[derive(Clone, Copy, PartialEq)]
/// enum Gender { Female, Male, Diverse }
///
/// // Written without `_`, so a new variant fails here too.
/// fn gender_label(gender: &Gender) -> String {
///     match gender {
///         Gender::Female => "Female",
///         Gender::Male => "Male",
///         Gender::Diverse => "Diverse",
///     }
///     .into()
/// }
///
/// static GENDERS: Choices<Gender> = choices!(Gender, gender_label, {
///     Female => "female",
///     Male => "male",
///     Diverse => "diverse",
/// });
///
/// # fn app() -> Element {
/// let mut gender = use_signal(|| None::<Choice<Gender>>);
/// rsx! {
///     NativeSelect {
///         label: "Gender",
///         placeholder: "Pick one",
///         value: gender(),
///         options: GENDERS.list(),
///         onchange: move |choice: Choice<Gender>| gender.set(Some(choice)),
///     }
/// }
/// # }
/// ```
pub struct Choices<V: Clone + 'static> {
    entries: Cow<'static, [(V, Cow<'static, str>)]>,
    label: fn(&V) -> String,
    key_of: Option<fn(&V) -> Option<&'static str>>,
    default: Option<V>,
}

impl<V: Clone + 'static> Clone for Choices<V> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            label: self.label,
            key_of: self.key_of,
            default: self.default.clone(),
        }
    }
}

impl<V: Clone + 'static> Choices<V> {
    /// A static list, in display order. A repeated key panics, so in a `static` it fails the build.
    pub const fn new(entries: &'static [(V, Cow<'static, str>)], label: fn(&V) -> String) -> Self {
        Self::__build(entries, label, None, None)
    }

    #[doc(hidden)]
    pub const fn __build(
        entries: &'static [(V, Cow<'static, str>)],
        label: fn(&V) -> String,
        key_of: Option<fn(&V) -> Option<&'static str>>,
        default: Option<V>,
    ) -> Self {
        let mut i = 0;
        while i < entries.len() {
            let mut j = i + 1;
            while j < entries.len() {
                if same_key(&entries[i].1, &entries[j].1) {
                    panic!("Choices::new: two entries share one key");
                }
                j += 1;
            }
            i += 1;
        }
        Self {
            entries: Cow::Borrowed(entries),
            label,
            key_of,
            default,
        }
    }

    /// A list that arrives at runtime, such as ids from a server.
    ///
    /// # Panics
    ///
    /// When two entries share one key.
    ///
    /// ```no_run
    /// use libero::components::Choices;
    ///
    /// #[derive(Clone, PartialEq)]
    /// struct Team { id: u32, name: String }
    ///
    /// fn team_label(team: &Team) -> String {
    ///     team.name.clone()
    /// }
    ///
    /// # let loaded: Vec<Team> = Vec::new();
    /// let teams = Choices::of(
    ///     loaded.into_iter().map(|team| (team.clone(), team.id.to_string().into())).collect(),
    ///     team_label,
    /// );
    /// # let _ = teams;
    /// ```
    pub fn of(entries: Vec<(V, Cow<'static, str>)>, label: fn(&V) -> String) -> Self {
        let mut seen = std::collections::HashSet::with_capacity(entries.len());
        for (_, key) in &entries {
            if !seen.insert(key.as_ref()) {
                panic!("Choices::of: the key `{key}` names two entries; each needs its own");
            }
        }
        Self {
            entries: Cow::Owned(entries),
            label,
            key_of: None,
            default: None,
        }
    }

    /// The value a single-pick field such as `SegmentedControl` shows while the store holds
    /// none: a foreign enum can't get a `Default` impl in the app. In [`choices!`](crate::choices)
    /// it is `default = Variant` before the list.
    ///
    /// ```no_run
    /// use libero::{choices, components::Choices};
    ///
    /// #[derive(Clone, Copy, PartialEq)]
    /// enum Size { Small, Large }
    ///
    /// fn size_label(size: &Size) -> String {
    ///     match size {
    ///         Size::Small => "Small",
    ///         Size::Large => "Large",
    ///     }
    ///     .into()
    /// }
    ///
    /// static SIZES: Choices<Size> =
    ///     choices!(Size, size_label, default = Small, { Small => "s", Large => "l" });
    /// let picked = SIZES.default_value(); // Some(&Size::Small)
    /// # let _ = picked;
    /// ```
    pub fn with_default(mut self, value: V) -> Self {
        self.default = Some(value);
        self
    }

    /// The value set through [`with_default`](Self::with_default).
    pub fn default_value(&self) -> Option<&V> {
        self.default.as_ref()
    }

    /// Every choice, in display order, as the `options` prop takes it, so groups and
    /// [`disabling`](OptionList::disabling) work as on any list.
    pub fn list(&self) -> OptionList<Choice<V>> {
        OptionList::new(self.choices())
    }

    /// Every choice, in display order.
    pub fn choices(&self) -> Vec<Choice<V>> {
        self.entries
            .iter()
            .map(|(value, key)| Choice {
                value: value.clone(),
                key: key.clone(),
                label: Label::Fn(self.label),
            })
            .collect()
    }

    /// The choice posted as `key`.
    pub fn by_key(&self, key: &str) -> Option<Choice<V>> {
        self.entries
            .iter()
            .find(|(_, entry_key)| entry_key == key)
            .map(|(value, entry_key)| Choice {
                value: value.clone(),
                key: entry_key.clone(),
                label: Label::Fn(self.label),
            })
    }
}

impl<V: Clone + PartialEq + 'static> Choices<V> {
    /// The choice that stands for `value`; `None` when the list doesn't hold it.
    pub fn choice(&self, value: &V) -> Option<Choice<V>> {
        let key: Cow<'static, str> = match self.key_of {
            Some(key_of) => key_of(value)?.into(),
            None => self
                .entries
                .iter()
                .find(|(entry, _)| entry == value)?
                .1
                .clone(),
        };
        Some(Choice {
            value: value.clone(),
            key,
            label: Label::Fn(self.label),
        })
    }

    /// [`choice`](Self::choice), or one with no key for a value the list no longer holds:
    /// it matches no option, so the field shows its placeholder, and the value is kept.
    pub(crate) fn choice_or_unlisted(&self, value: &V) -> Choice<V> {
        self.choice(value).unwrap_or_else(|| Choice {
            value: value.clone(),
            key: Cow::Borrowed(""),
            label: Label::Fn(self.label),
        })
    }

    /// What a single pick shows while nothing is stored: the default, else the first entry.
    pub(crate) fn fallback(&self) -> Option<Choice<V>> {
        match &self.default {
            Some(value) => Some(self.choice_or_unlisted(value)),
            None => self.choices().into_iter().next(),
        }
    }

    /// The same entries, label and default: a list rebuilt per render is the same list.
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.entries == other.entries
            && std::ptr::fn_addr_eq(self.label, other.label)
            && self.default == other.default
    }
}

// `&Cow`, not `&str`: a const fn can't deref a `Cow`, only match it.
#[allow(clippy::ptr_arg)]
const fn same_key(a: &Cow<'static, str>, b: &Cow<'static, str>) -> bool {
    let (a, b) = match (a, b) {
        (Cow::Borrowed(a), Cow::Borrowed(b)) => (a.as_bytes(), b.as_bytes()),
        _ => panic!("Choices::new: a static key is borrowed"),
    };
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// A [`Choices`] list for an enum, in display order. The keys are one `match` with no `_`
/// arm, so a variant added to the enum fails the build here.
///
/// ```no_run
/// use libero::{choices, components::Choices};
///
/// #[derive(Clone, Copy, PartialEq)]
/// enum Gender { Female, Male, Diverse }
///
/// fn gender_label(gender: &Gender) -> String {
///     format!("{}", match gender {
///         Gender::Female => "Female",
///         Gender::Male => "Male",
///         Gender::Diverse => "Diverse",
///     })
/// }
///
/// static GENDERS: Choices<Gender> = choices!(Gender, gender_label, {
///     Female => "female",
///     Male => "male",
///     Diverse => "diverse",
/// });
/// ```
///
/// A `#[non_exhaustive]` enum from another crate can't be matched in full: end with
/// `_ => skip`, so the gap shows at the call site. A missing variant fails the build:
///
/// ```compile_fail
/// use libero::{choices, components::Choices};
///
/// #[derive(Clone, Copy, PartialEq)]
/// enum Gender { Female, Male, Diverse }
///
/// fn gender_label(_: &Gender) -> String { String::new() }
///
/// static GENDERS: Choices<Gender> = choices!(Gender, gender_label, {
///     Female => "female",
///     Male => "male",
/// });
/// ```
#[macro_export]
macro_rules! choices {
    (@build $($enum:ident)::+, $label:expr, $default:expr, [$($variant:ident => $key:literal),+], [$($rest:tt)*]) => {{
        type Enum = $($enum)::+;
        const ENTRIES: &[(Enum, ::std::borrow::Cow<'static, str>)] = &[
            $((Enum::$variant, ::std::borrow::Cow::Borrowed($key))),+
        ];
        #[allow(unreachable_patterns)]
        fn key_of(value: &Enum) -> ::std::option::Option<&'static str> {
            match value {
                $(Enum::$variant => ::std::option::Option::Some($key),)+
                $($rest)*
            }
        }
        $crate::components::Choices::__build(ENTRIES, $label, ::std::option::Option::Some(key_of), $default)
    }};
    (@default $($enum:ident)::+) => { ::std::option::Option::None };
    (@default $($enum:ident)::+, $default:ident) => { ::std::option::Option::Some($($enum)::+::$default) };
    (@list $($enum:ident)::+, $label:expr, [$($default:ident)?], { $($variant:ident => $key:literal,)+ _ => skip $(,)? }) => {
        $crate::choices!(@build $($enum)::+, $label, $crate::choices!(@default $($enum)::+ $(, $default)?), [$($variant => $key),+], [_ => ::std::option::Option::None])
    };
    (@list $($enum:ident)::+, $label:expr, [$($default:ident)?], { $($variant:ident => $key:literal),+ $(,)? }) => {
        $crate::choices!(@build $($enum)::+, $label, $crate::choices!(@default $($enum)::+ $(, $default)?), [$($variant => $key),+], [])
    };
    ($($enum:ident)::+, $label:expr, default = $default:ident, $list:tt) => {
        $crate::choices!(@list $($enum)::+, $label, [$default], $list)
    };
    ($($enum:ident)::+, $label:expr, $list:tt) => {
        $crate::choices!(@list $($enum)::+, $label, [], $list)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Gender {
        Female,
        Male,
        Diverse,
    }

    fn gender_label(gender: &Gender) -> String {
        format!("{gender:?}")
    }

    static GENDERS: Choices<Gender> = crate::choices!(Gender, gender_label, {
        Female => "female",
        Male => "male",
        Diverse => "diverse",
    });

    static PARTIAL: Choices<Gender> = crate::choices!(Gender, gender_label, default = Male, {
        Female => "female",
        Male => "male",
        _ => skip
    });

    #[test]
    fn the_macro_lists_in_order_with_keys_and_labels() {
        let choices = GENDERS.choices();
        let keys: Vec<_> = choices.iter().map(Options::value).collect();
        let labels: Vec<_> = choices.iter().map(Options::label).collect();
        assert_eq!(keys, ["female", "male", "diverse"]);
        assert_eq!(labels, ["Female", "Male", "Diverse"]);
        assert!(GENDERS.default_value().is_none());
        assert_eq!(PARTIAL.default_value(), Some(&Gender::Male));
        assert!(PARTIAL.choice(&Gender::Diverse).is_none());
    }

    #[test]
    fn a_choice_derefs_to_its_value_and_is_found_by_value_or_key() {
        let male = GENDERS.choice(&Gender::Male).unwrap();
        assert_eq!(*male, Gender::Male);
        assert_eq!(male.key(), "male");
        assert!(GENDERS.by_key("male") == Some(male));
        assert!(GENDERS.by_key("other").is_none());
    }

    #[test]
    fn list_carries_flags_like_any_option_list() {
        let list = GENDERS.list().disabling(|choice| **choice == Gender::Male);
        assert_eq!(list.disabled(), [false, true, false]);
        assert_eq!(list.values().len(), 3);
    }

    #[test]
    fn a_runtime_list_finds_keys_by_value() {
        let teams = Choices::of(vec![(7, "seven".into()), (9, "nine".into())], |n| {
            n.to_string()
        });
        assert_eq!(teams.choice(&9).unwrap().key(), "nine");
        assert!(teams.choice(&1).is_none());
        assert!(teams.default_value().is_none());
    }

    #[test]
    #[should_panic(expected = "the key `a` names two entries")]
    fn a_runtime_list_rejects_a_repeated_key() {
        Choices::of(vec![(1, "a".into()), (2, "a".into())], |n: &i32| {
            n.to_string()
        });
    }

    #[test]
    #[should_panic(expected = "two entries share one key")]
    fn a_static_list_rejects_a_repeated_key() {
        static ENTRIES: &[(i32, Cow<'static, str>)] =
            &[(1, Cow::Borrowed("a")), (2, Cow::Borrowed("a"))];
        Choices::new(ENTRIES, |n: &i32| n.to_string());
    }
}
