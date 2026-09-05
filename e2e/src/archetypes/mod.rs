//! Interaction archetypes: one contract, many components.
//!
//! This is where the composition pays off, and it is the file that decides
//! whether this suite reaches the whole library or stalls at five components.
//! The library has far fewer *patterns* than components:
//!
//! | Archetype | Components that are one |
//! |---|---|
//! | [`Combobox`] | `Autocomplete`, `TagsField`, `Select`, `MultiSelect`, `Cascader` |
//! | [`RovingTabindex`] | `Tabs`, `Menubar`, `Toolbar`, `Tree` |
//! | [`RadioSet`] | `RadioGroup`, `SegmentedControl` |
//! | [`Overlay`] | `Modal`, `Drawer`, `Menu`, `Lightbox`, `Spotlight`, `FloatingWindow` |
//!
//! So the keyboard and focus contract is written once per pattern and
//! parameterised, rather than copied into eighteen test files that then drift.
//!
//! An archetype asserts only what the **pattern** guarantees, and APG is the
//! source for what that is ([[principles/follow-the-a11y-specs]]). Anything
//! specific to one component belongs in that component's own test.
//!
//! ## Adding an archetype
//!
//! One file here, a struct holding the selectors it needs, and an
//! `assert_contract(&self, page)`. Keep the assertions to what the spec
//! actually requires: an archetype that encodes one component's incidental
//! behaviour will fail on the next component that adopts it, and the fix will
//! look like weakening the test.

mod combobox;
mod overlay;
mod radio_set;
mod roving;

pub use combobox::Combobox;
pub use overlay::Overlay;
pub use radio_set::RadioSet;
pub use roving::{Orientation, RovingTabindex};
