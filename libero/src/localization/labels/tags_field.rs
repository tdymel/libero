/// What a `TagsField` announces when it refuses a tag. `{labels}` are the
/// refused tags joined by `, `; the draft keeps the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagsFieldLabels {
    /// The tag is already in the list.
    pub duplicate: &'static str,
    /// The list holds `max_tags` already.
    pub full: &'static str,
    /// `tag_rules` turned it down.
    pub not_allowed: &'static str,
}

impl TagsFieldLabels {
    pub const ENGLISH: Self = Self {
        duplicate: "Already added: {labels}",
        full: "Tag limit reached, not added: {labels}",
        not_allowed: "Not allowed: {labels}",
    };

    pub const GERMAN: Self = Self {
        duplicate: "Bereits hinzugefügt: {labels}",
        full: "Höchstzahl erreicht, nicht hinzugefügt: {labels}",
        not_allowed: "Nicht erlaubt: {labels}",
    };
}
