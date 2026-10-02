/// A `FileField`'s own words. A chip's x reads [`CommonLabels::remove`](super::CommonLabels::remove).
///
/// ```
/// use libero::localization::FileFieldLabels;
///
/// const WORDS: FileFieldLabels = FileFieldLabels {
///     any_of: |group| match group {
///         "image" => "Fotos".to_string(),
///         group => (FileFieldLabels::GERMAN.any_of)(group),
///     },
///     ..FileFieldLabels::GERMAN
/// };
/// assert_eq!((WORDS.any_of)("image"), "Fotos");
/// assert_eq!((WORDS.any_of)("video"), "Videos");
/// assert_eq!((FileFieldLabels::ENGLISH.any_of)("audio"), "audios");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct FileFieldLabels {
    /// The `Input` variant's Browse button, heard after the field's label.
    pub browse: &'static str,
    /// The Browse button's description on a `required` field.
    pub required: &'static str,
    /// The dropzone's prompt when no `placeholder` or children are given.
    pub drop_file: &'static str,
    /// The same, for a `multiple` field.
    pub drop_files: &'static str,
    /// Names a `type/*` entry of `accept` in the dropzone's hint: `image`
    /// reads as "images". A fn rather than a template, for plural forms.
    pub any_of: fn(&str) -> String,
    /// A card's file size units, bytes to terabytes, in powers of 1000. The
    /// decimal separator is [`Formats::decimal_separator`](crate::localization::Formats).
    pub size_units: [&'static str; 5],
    /// The picture's alt text in a `crop` dialog; `{name}` is the file's name.
    pub crop_image: &'static str,
    /// Said when a pick or drop holds a type `accept` excludes; `{names}` lists them.
    pub rejected_type: &'static str,
    /// Said when a single-file field gets more than one; `{names}` lists the rest.
    pub rejected_many: &'static str,
}

/// `FileFieldLabels::ENGLISH.any_of`. A named fn, so every copy compares equal.
fn english_any_of(group: &str) -> String {
    format!("{group}s")
}

/// `FileFieldLabels::GERMAN.any_of`: a German plural rarely adds a letter.
fn german_any_of(group: &str) -> String {
    match group {
        "image" => "Bilder".to_string(),
        "video" => "Videos".to_string(),
        "audio" => "Audiodateien".to_string(),
        "text" => "Textdateien".to_string(),
        "font" => "Schriftarten".to_string(),
        group => format!("{group}-Dateien"),
    }
}
impl FileFieldLabels {
    pub const ENGLISH: Self = Self {
        browse: "Browse files",
        required: "Required",
        drop_file: "Drop a file here, or click to pick",
        drop_files: "Drop files here, or click to pick",
        any_of: english_any_of,
        size_units: ["B", "kB", "MB", "GB", "TB"],
        crop_image: "Picture to crop: {name}",
        rejected_type: "Not added, not an accepted file type: {names}",
        rejected_many: "Not added, the field takes one file: {names}",
    };

    pub const GERMAN: Self = Self {
        browse: "Dateien durchsuchen",
        required: "Pflichtfeld",
        drop_file: "Datei hier ablegen oder zum Auswählen klicken",
        drop_files: "Dateien hier ablegen oder zum Auswählen klicken",
        any_of: german_any_of,
        size_units: ["B", "kB", "MB", "GB", "TB"],
        crop_image: "Zu beschneidendes Bild: {name}",
        rejected_type: "Nicht hinzugefügt, kein erlaubter Dateityp: {names}",
        rejected_many: "Nicht hinzugefügt, das Feld nimmt nur eine Datei: {names}",
    };
}
