use super::{Property, SxEntry, SxModifierKey, SxPropertyKey, ThemeAwareValue};
use crate::css::canonical_condition;
use crate::tokens::{CssVar, Size};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Default, Hash)]
pub struct Sx {
    entries: Vec<SxEntry>,
}

impl Sx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(self, property: impl Into<String>, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_declaration(SxPropertyKey::parse(property), value.into())
    }

    /// Declares a CSS custom property in this rule - the in-stylesheet twin
    /// of [`Variables`](crate::components::Variables).
    pub fn var(self, name: CssVar, value: impl Into<ThemeAwareValue>) -> Self {
        self.with(name.name().to_string(), value)
    }

    pub(super) fn with_known_property(
        self,
        property: Property,
        value: impl Into<ThemeAwareValue>,
    ) -> Self {
        self.with_declaration(SxPropertyKey::Known(property), value.into())
    }

    pub(super) fn modifier(self, modifier: SxModifierKey, nested: Sx) -> Self {
        self.with_modifier(modifier, nested)
    }

    pub fn hover(self, nested: Sx) -> Self {
        self.selector(":hover", nested)
    }

    pub fn focus(self, nested: Sx) -> Self {
        self.selector(":focus", nested)
    }

    pub fn focus_visible(self, nested: Sx) -> Self {
        self.selector(":focus-visible", nested)
    }

    /// The element, while focus is on it *or* on anything inside it. What a
    /// wrapper uses once the focusable thing is a child of it.
    pub fn focus_within(self, nested: Sx) -> Self {
        self.selector(":focus-within", nested)
    }

    /// [`focus_within`](Self::focus_within), restricted to the focus the
    /// browser would draw a ring for - so a mouse click does not get one.
    /// There is no `:focus-visible-within`, hence the `:has`.
    pub fn has_focus_visible(self, nested: Sx) -> Self {
        self.selector(":has(:focus-visible)", nested)
    }

    pub fn when(self, condition: impl Into<String>, nested: Sx) -> Self {
        self.modifier(
            SxModifierKey::Condition(canonical_condition(&condition.into())),
            nested,
        )
    }

    /// One `when(size.state_name(), ..)` block per `Size`.
    pub fn per_size(self, size_sx: impl Fn(Size) -> Sx) -> Self {
        Size::ALL.into_iter().fold(self, |base, size| {
            base.when(size.state_name(), size_sx(size))
        })
    }

    /// Same, keyed by `radius-{size}`, so radius is independent of `size`.
    pub fn per_radius(self, radius_sx: impl Fn(Size) -> Sx) -> Self {
        Size::ALL.into_iter().fold(self, |base, radius| {
            base.when(radius.radius_state_name(), radius_sx(radius))
        })
    }

    /// Same, keyed by `shadow-{size}`, for an elevation axis.
    pub fn per_shadow(self, shadow_sx: impl Fn(Size) -> Sx) -> Self {
        Size::ALL.into_iter().fold(self, |base, shadow| {
            base.when(shadow.shadow_state_name(), shadow_sx(shadow))
        })
    }

    pub fn selector(self, selector: impl Into<String>, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Selector(selector.into()), nested)
    }

    pub fn breakpoint(self, breakpoint: Size, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Breakpoint(breakpoint), nested)
    }

    /// Styles that apply while `query` matches, e.g.
    /// `"(prefers-reduced-motion: reduce)"`.
    ///
    /// The query is passed through verbatim. Nothing validates it, so a typo
    /// silently matches nothing - the same tradeoff [`when`](Self::when)
    /// makes. Nested media modifiers fold into one `and` query, exactly like
    /// nested [`breakpoint`](Self::breakpoint)s.
    pub fn media(self, query: impl Into<String>, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Media(query.into()), nested)
    }

    /// Marks this element as a named inline-size query container, so
    /// descendants can [`container_query`](Self::container_query) it.
    ///
    /// The name is mandatory: an anonymous container binds a query to the
    /// *nearest* ancestor container, which silently picks the wrong one as
    /// soon as containers nest.
    ///
    /// `container-type: inline-size` is `contain: layout style inline-size` -
    /// the element stops being sized by its own contents in the inline axis,
    /// becomes a stacking context, and becomes the containing block for
    /// absolutely and fixed positioned descendants. Never put it on a
    /// shrink-to-fit box.
    pub fn container(self, name: impl Into<String>) -> Self {
        self.container_type("inline-size")
            .container_name(name.into())
    }

    /// Styles that apply while the named ancestor container matches
    /// `condition`, e.g. `"(min-width: 640px)"`.
    ///
    /// Like `@media`, the condition cannot read CSS custom properties.
    pub fn container_query(
        self,
        name: impl Into<String>,
        condition: impl Into<String>,
        nested: Sx,
    ) -> Self {
        self.modifier(
            SxModifierKey::Container {
                name: name.into(),
                condition: condition.into(),
            },
            nested,
        )
    }

    /// [`container_query`](Self::container_query) at a `Size`'s breakpoint -
    /// the container twin of [`breakpoint`](Self::breakpoint).
    pub fn container_breakpoint(
        self,
        name: impl Into<String>,
        breakpoint: Size,
        nested: Sx,
    ) -> Self {
        self.container_query(
            name,
            format!("(min-width: {})", breakpoint.breakpoint_value()),
            nested,
        )
    }

    pub fn apply_if<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        match value {
            Some(value) => f(self, value),
            None => self,
        }
    }

    /// At most one declaration per property, and re-declaring *moves it to the
    /// end* rather than overwriting in place - unlike [`Sx::with_modifier`].
    /// Within one rule, source order is what settles a shorthand against a
    /// longhand (`padding` vs `padding-top`), so the latest declaration must
    /// land last for "re-declaring wins" to hold. Modifiers can keep their
    /// position because their `[data-state~=..]` selector settles them by
    /// specificity instead.
    fn with_declaration(mut self, property: SxPropertyKey, value: ThemeAwareValue) -> Self {
        let existing = self.entries.iter().position(|entry| {
            matches!(entry, SxEntry::Declaration { property: existing, .. } if existing == &property)
        });

        if let Some(index) = existing {
            self.entries.remove(index);
        }

        self.entries.push(SxEntry::Declaration { property, value });
        self
    }

    /// At most one block per modifier: a second `:hover`/`when(..)` merges
    /// into the first, in place, rather than emitting a duplicate scope.
    fn with_modifier(mut self, modifier: SxModifierKey, nested: Sx) -> Self {
        let existing = self.entries.iter().position(|entry| {
            matches!(entry, SxEntry::Nested { modifier: existing, .. } if existing == &modifier)
        });

        match existing {
            Some(index) => {
                let SxEntry::Nested { sx, .. } = &mut self.entries[index] else {
                    unreachable!("position() matched a nested entry")
                };
                *sx = std::mem::take(sx).and(nested);
            }
            None => self.entries.push(SxEntry::Nested {
                modifier,
                sx: nested,
            }),
        }

        self
    }

    /// Merges `other` on top of `self`. A property `other` re-declares moves
    /// to the end (see [`Sx::with_declaration`]), which is what lets it beat a
    /// shorthand `self` declared earlier.
    ///
    /// So declaration order depends on build order, and two equivalent `Sx`
    /// can render different CSS text under different class names. Cheap in
    /// practice: every call site composes a `StaticSx` once per process, and
    /// the `sx`/`framework_sx` override path never merges at all.
    pub fn and(mut self, other: Sx) -> Self {
        for entry in other.entries {
            match entry {
                SxEntry::Declaration { property, value } => {
                    self = self.with_declaration(property, value);
                }
                SxEntry::Nested { modifier, sx } => {
                    self = self.with_modifier(modifier, sx);
                }
            }
        }

        self
    }

    pub(crate) fn entries(&self) -> &[SxEntry] {
        &self.entries
    }

    /// Hash of the entries, not the derived `Hash::hash`.
    pub(crate) fn content_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash_into(&mut hasher);
        hasher.finish()
    }

    fn hash_into(&self, hasher: &mut impl Hasher) {
        self.entries.hash(hasher);
    }

    /// Base62 hash of the rendered CSS, and its stylesheet-registry key.
    /// Getting it means building the CSS, so at runtime it comes from
    /// `use_css`, which already has the sheet - this spelling is for tests.
    #[cfg(test)]
    pub(crate) fn class_name(&self) -> String {
        class_name_from_hash(crate::css::Stylesheet::from(self).hash())
    }
}

/// The one place a `.lsx-*` class name is minted.
pub(crate) fn class_name_from_hash(hash: u64) -> String {
    format!("lsx-{}", encode_base62(hash))
}

const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// 11 chars for a `u64`, against hex's 16.
fn encode_base62(mut value: u64) -> String {
    if value == 0 {
        return "0".to_string();
    }

    let mut chars = Vec::new();
    while value > 0 {
        chars.push(BASE62_ALPHABET[(value % 62) as usize]);
        value /= 62;
    }
    chars.reverse();

    String::from_utf8(chars).expect("base62 alphabet is ASCII")
}

pub fn sx() -> Sx {
    Sx::new()
}

#[cfg(test)]
mod tests {
    use super::encode_base62;

    #[test]
    fn encode_base62_roundtrips_edge_values() {
        assert_eq!(encode_base62(0), "0");
        assert_eq!(encode_base62(61), "z");
        assert_eq!(encode_base62(62), "10");
        assert_eq!(encode_base62(u64::MAX), "LygHa16AHYF");
    }

    #[test]
    fn per_size_and_per_radius_match_the_folds_they_replaced() {
        use crate::theme::Size;

        let helpers = super::sx()
            .per_size(|size| super::sx().padding(size.state_name()))
            .per_radius(|radius| super::sx().border_radius(radius.state_name()));
        let base = Size::ALL.into_iter().fold(super::sx(), |base, size| {
            base.when(size.state_name(), super::sx().padding(size.state_name()))
        });
        let folded = Size::ALL.into_iter().fold(base, |base, radius| {
            base.when(
                radius.radius_state_name(),
                super::sx().border_radius(radius.state_name()),
            )
        });

        assert_eq!(helpers.class_name(), folded.class_name());
    }

    /// Overwriting in place instead would emit `padding-top:4px;padding:8px`,
    /// letting the shorthand swallow the override - which `Flex`, `Divider`
    /// and `DataList`'s `and` compositions rely on not happening.
    #[test]
    fn a_re_declared_property_moves_last_so_it_beats_an_earlier_shorthand() {
        use crate::css::Stylesheet;

        let merged = super::sx()
            .padding_top("2px")
            .padding("8px")
            .and(super::sx().padding_top("4px"));

        assert!(
            Stylesheet::from(&merged)
                .as_str()
                .contains("padding:8px;padding-top:4px;"),
            "re-declared property must render last, got {}",
            Stylesheet::from(&merged).as_str()
        );
    }

    #[test]
    fn class_name_is_shorter_than_previous_hex_encoding() {
        let name = super::sx().padding("lg").class_name();
        assert!(name.starts_with("lsx-"));
        assert!(name.len() <= "lsx-".len() + 11);
    }
}
