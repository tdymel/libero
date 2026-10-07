//! The `accept` attribute, applied in Rust: the picker applies it, a drop
//! does not.

/// Whether a file satisfies `accept` (`.ext`, `type/subtype`, `type/*`). An
/// empty list accepts everything.
pub(super) fn accepts(accept: &str, name: &str, content_type: Option<&str>) -> bool {
    accepts_from(accept, name, content_type, false)
}

/// [`accepts`] for a file the picker offered. The picker applied `accept` already,
/// so a file the platform gave no type (an HEIC on Windows) passes `image/*` too.
pub(super) fn picker_accepts(accept: &str, name: &str, content_type: Option<&str>) -> bool {
    accepts_from(accept, name, content_type, true)
}

fn accepts_from(accept: &str, name: &str, content_type: Option<&str>, picked: bool) -> bool {
    let mut entries = accept.split(',').map(str::trim).filter(|e| !e.is_empty());
    let mut empty = true;
    for entry in entries.by_ref() {
        empty = false;
        if matches(entry, name, content_type, picked) {
            return true;
        }
    }
    empty
}

fn matches(entry: &str, name: &str, content_type: Option<&str>, picked: bool) -> bool {
    // A suffix match, so `.tar.gz` works; case-insensitive, as `.PDF` is a `.pdf`.
    if entry.starts_with('.') {
        let (name, entry) = (name.as_bytes(), entry.as_bytes());
        return name.len() >= entry.len()
            && name[name.len() - entry.len()..].eq_ignore_ascii_case(entry);
    }

    // A dropped file with no media type only matches `*/*`.
    let Some(content_type) = content_type.filter(|content_type| !content_type.is_empty()) else {
        return picked || entry == "*/*";
    };
    // Parameters (`;charset=utf-8`) are not compared.
    let content_type = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let entry = entry.to_ascii_lowercase();

    // `*/*` first: it also ends in `/*`.
    if entry == "*/*" {
        return true;
    }
    match entry.strip_suffix("/*") {
        Some(group) => content_type
            .split_once('/')
            .is_some_and(|(actual, _)| actual == group),
        None => entry == content_type,
    }
}

/// What the dropzone says it takes (`.pdf` as `PDF`, `type/*` via `any_of`),
/// read off `accept` itself. `None` when it takes everything.
pub(super) fn accept_hint(accept: &str, any_of: fn(&str) -> String) -> Option<String> {
    let names: Vec<String> = accept
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty() && *entry != "*/*")
        .map(|entry| entry_name(entry, any_of))
        .collect();
    (!names.is_empty()).then(|| names.join(", "))
}

fn entry_name(entry: &str, any_of: fn(&str) -> String) -> String {
    if let Some(extension) = entry.strip_prefix('.') {
        return extension.to_ascii_uppercase();
    }
    match entry.split_once('/') {
        // `image/*` is "images", and an unknown group still reads as a plural
        // rather than as a media type nobody outside the console knows.
        Some((group, "*")) => any_of(group),
        Some((_, subtype)) => subtype.to_ascii_uppercase(),
        None => entry.to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::localization::FileFieldLabels;

    #[test]
    fn an_empty_accept_takes_everything() {
        assert!(accepts("", "a.pdf", Some("application/pdf")));
        assert!(accepts("  ,  ", "a.pdf", Some("application/pdf")));
    }

    #[test]
    fn an_extension_matches_the_name_whatever_its_case() {
        assert!(accepts(".pdf", "report.pdf", None));
        assert!(accepts(".pdf", "REPORT.PDF", None));
        assert!(!accepts(".pdf", "report.png", Some("image/png")));
        // No extension at all is not a match for one.
        assert!(!accepts(".pdf", "report", None));
    }

    /// Todo 2552: the picker offers `a.tar.gz` for `.tar.gz`, so the field takes it.
    #[test]
    fn a_compound_extension_matches_the_whole_suffix() {
        assert!(accepts(".tar.gz", "backup.tar.gz", None));
        assert!(accepts(".tar.gz", "BACKUP.TAR.GZ", None));
        assert!(accepts(".d.ts", "index.d.ts", Some("video/mp2t")));
        assert!(!accepts(".tar.gz", "backup.gz", None));
        assert!(!accepts(".gz", "backup.tgz", None));
    }

    #[test]
    fn a_wildcard_matches_the_group_only() {
        assert!(accepts("image/*", "a.png", Some("image/png")));
        assert!(!accepts("image/*", "a.pdf", Some("application/pdf")));
        assert!(accepts("*/*", "a.pdf", Some("application/pdf")));
    }

    #[test]
    fn a_media_type_matches_exactly_and_ignores_parameters() {
        assert!(accepts("text/plain", "a.txt", Some("text/plain")));
        assert!(accepts(
            "text/plain",
            "a.txt",
            Some("text/plain;charset=utf-8")
        ));
        assert!(accepts("TEXT/PLAIN", "a.txt", Some("text/plain")));
        assert!(!accepts("text/plain", "a.csv", Some("text/csv")));
    }

    #[test]
    fn a_list_takes_a_file_any_entry_accepts() {
        let accept = "image/png, .pdf, text/*";
        assert!(accepts(accept, "a.png", Some("image/png")));
        assert!(accepts(accept, "a.pdf", None));
        assert!(accepts(accept, "a.md", Some("text/markdown")));
        assert!(!accepts(accept, "a.zip", Some("application/zip")));
    }

    fn english(accept: &str) -> Option<String> {
        accept_hint(accept, FileFieldLabels::ENGLISH.any_of)
    }

    #[test]
    fn a_hint_names_what_the_attribute_takes() {
        assert_eq!(english("image/*"), Some("images".to_string()));
        assert_eq!(english(".pdf"), Some("PDF".to_string()));
        assert_eq!(english("image/png"), Some("PNG".to_string()));
        assert_eq!(
            english("image/png, .pdf, video/*"),
            Some("PNG, PDF, videos".to_string())
        );
    }

    /// Todo 656: a wildcard's name is the localization's, not an English `s`.
    #[test]
    fn a_wildcard_reads_as_the_localization_names_it() {
        let german: fn(&str) -> String = |group| match group {
            "image" => "Bilder".to_string(),
            group => format!("{group}-Dateien"),
        };
        assert_eq!(
            accept_hint("image/*, .pdf, audio/*", german),
            Some("Bilder, PDF, audio-Dateien".to_string())
        );
    }

    /// An attribute that excludes nothing has nothing to announce.
    #[test]
    fn an_open_accept_has_no_hint() {
        assert_eq!(english(""), None);
        assert_eq!(english("*/*"), None);
        assert_eq!(english(" , "), None);
    }

    /// A drop can carry a file the platform gave no type for, and only an
    /// extension entry or `*/*` can speak to it.
    #[test]
    fn a_typeless_file_needs_an_extension_or_a_star() {
        assert!(!accepts("image/*", "photo", None));
        assert!(!accepts("image/*", "photo", Some("")));
        assert!(accepts("*/*", "photo", None));
        assert!(accepts(".png", "photo.png", None));
    }

    /// Todo 2552: the picker offered the typeless file for `image/*`, so the field takes it.
    #[test]
    fn a_typeless_pick_passes_a_media_type_entry() {
        assert!(picker_accepts("image/*", "IMG_1.HEIC", None));
        assert!(picker_accepts("image/*", "IMG_1.HEIC", Some("")));
        assert!(!picker_accepts("image/*", "a.pdf", Some("application/pdf")));
        assert!(!picker_accepts(".png", "IMG_1.HEIC", None));
    }
}
