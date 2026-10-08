use super::{Property, SxEntry, SxModifierKey, SxPropertyKey, ThemeAwareValue};
use crate::css::canonical_condition;
use crate::tokens::{CssVar, Size};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// The one reduced-motion query for [`Sx::media`], so a grep finds every guard.
pub(crate) const REDUCED_MOTION: &str = "(prefers-reduced-motion: reduce)";

/// The one forced-colours query for [`Sx::media`], same reason.
pub(crate) const FORCED_COLORS: &str = "(forced-colors: active)";

/// A style: declarations plus nested modifiers, rendered to one hashed class.
/// Values take theme tokens: sizes (`"md"`), palette colours (`"primary.7"`), vars.
/// Values, selectors and queries reach the stylesheet unescaped: never user text.
///
/// ```
/// # use libero::sx::sx;
/// let card = sx()
///     .padding("md")
///     .background_color("primary.1")
///     .hover(sx().background_color("primary.2"))
///     .media("(prefers-reduced-motion: reduce)", sx().transition("none"));
/// ```
///
/// Docs: <https://libero-ui.dev/about/styling>
#[derive(Debug, Clone, PartialEq, Default, Hash)]
pub struct Sx {
    entries: Vec<SxEntry>,
}

impl Sx {
    /// An empty style, the same as [`sx()`](crate::sx::sx).
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares any CSS property by name, for one without its own method.
    ///
    /// ```
    /// # use libero::sx::sx;
    /// let tinted = sx().with("accent-color", "primary.6");
    /// ```
    pub fn with(self, property: impl Into<String>, value: impl Into<ThemeAwareValue>) -> Self {
        self.with_declaration(SxPropertyKey::parse(property), value.into())
    }

    /// Declares a CSS custom property in this rule, the stylesheet twin of
    /// [`Variables`](crate::components::Variables).
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

    /// Styles while the pointer is over the element.
    pub fn hover(self, nested: Sx) -> Self {
        self.selector(":hover", nested)
    }

    /// Styles while the element has focus, by pointer too; prefer [`focus_visible`](Self::focus_visible) for rings.
    pub fn focus(self, nested: Sx) -> Self {
        self.selector(":focus", nested)
    }

    /// Styles while the element has focus the browser would show, as after a key.
    pub fn focus_visible(self, nested: Sx) -> Self {
        self.selector(":focus-visible", nested)
    }

    /// While focus is on the element or inside it, for a wrapper around the focusable child.
    pub fn focus_within(self, nested: Sx) -> Self {
        self.selector(":focus-within", nested)
    }

    /// Styles while the element's `data-state` holds the named states: `"open"`,
    /// `"open && disabled"` or `"sm || xs"` (`&&` binds tighter, no parentheses).
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

    /// Styles under any selector; `&` stands for the element, else the selector is appended
    /// to it (`":hover"`, `"[aria-current]"`).
    pub fn selector(self, selector: impl Into<String>, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Selector(selector.into()), nested)
    }

    /// Styles under a right-to-left direction. Blitz's stylo does not match
    /// `:dir()`, so an `[dir=rtl]` ancestor arm repeats it there (todo 735).
    pub(crate) fn rtl(self, nested: Sx) -> Self {
        self.selector("&:dir(rtl)", nested.clone())
            .selector(":where([dir=rtl]) &", nested)
    }

    /// Styles from the viewport width of `breakpoint` up (`min-width`, mobile first).
    pub fn breakpoint(self, breakpoint: Size, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Breakpoint(breakpoint), nested)
    }

    /// Styles while `query` matches, e.g. `"(prefers-reduced-motion: reduce)"`.
    /// Passed verbatim, so a typo silently matches nothing; nested ones join with `and`.
    pub fn media(self, query: impl Into<String>, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Media(query.into()), nested)
    }

    /// Styles where the browser knows `condition`, e.g. `"(height: 1dvh)"`: declare the plain
    /// value first, then override it here, since CSS has no two values for one property.
    ///
    /// ```
    /// # use libero::sx::sx;
    /// let tall = sx().height("70vh").supports("(height: 1dvh)", sx().height("70dvh"));
    /// ```
    pub fn supports(self, condition: impl Into<String>, nested: Sx) -> Self {
        self.modifier(SxModifierKey::Supports(condition.into()), nested)
    }

    /// Makes this a named inline-size container for [`container_query`](Self::container_query).
    /// Named, since an anonymous one binds to the nearest container once they nest.
    ///
    /// **Give it a width from outside**: a container is not sized by its content, so in a
    /// shrink-to-fit context (a content-sized flex or grid item, float, inline-block,
    /// absolute box, `max-content` wrapper) it measures 0px and every query silently misses.
    /// It also becomes a stacking context and the containing block for positioned descendants.
    pub fn container(self, name: impl Into<String>) -> Self {
        self.container_type("inline-size")
            .container_name(name.into())
    }

    /// Styles while the named ancestor container matches `condition`, e.g.
    /// `"(min-width: 640px)"`. Like `@media`, it cannot read custom properties.
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

    /// [`container_query`](Self::container_query) at a `Size`'s breakpoint.
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

    /// Runs `f` with the value when there is one, so an optional prop chains:
    /// `sx().apply_if(width, |sx, width| sx.width(width))`.
    pub fn apply_if<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        match value {
            Some(value) => f(self, value),
            None => self,
        }
    }

    /// Re-declaring moves a property to the end, since source order settles a shorthand
    /// against a longhand. Modifiers merge in place: specificity settles them.
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

    /// A second `:hover`/`when(..)` merges into the first, in place.
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

    /// Merges `other` on top; a property it re-declares moves last, so it beats an
    /// earlier shorthand. Build order thus shapes the CSS text and class name.
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

    /// Base62 hash of the rendered CSS. Tests only: at runtime `use_css` has the sheet.
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

/// Starts an empty [`Sx`].
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Box;
/// # use libero::sx::sx;
/// # fn app() -> Element {
/// rsx! {
///     Box { sx: sx().padding("md").color("primary"), "Hello" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/about/styling>
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
        use crate::tokens::Size;

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

    /// In place would emit `padding-top:4px;padding:8px`; `Flex`, `Divider` and
    /// `DataList` rely on the override winning.
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
    fn supports_wraps_its_block_in_an_at_supports_after_the_plain_value() {
        use crate::css::Stylesheet;

        let sheet = super::sx()
            .height("70vh")
            .supports("(height: 1dvh)", super::sx().height("70dvh"));
        let css = Stylesheet::from(&sheet);
        let css = css.as_str();

        let (plain, wrapped) = css.split_once("@supports (height: 1dvh){").unwrap();
        assert!(plain.contains("height:70vh;"), "{css}");
        assert!(wrapped.contains("height:70dvh;"), "{css}");
    }

    #[test]
    fn class_name_is_shorter_than_previous_hex_encoding() {
        let name = super::sx().padding("lg").class_name();
        assert!(name.starts_with("lsx-"));
        assert!(name.len() <= "lsx-".len() + 11);
    }
}
