//! What a `FileField` holds: none, one, or many.

use dioxus::html::FileData;

use crate::localization::FileFieldLabels;

/// A `FileField`'s value: one type for both arities, since `multiple` is a
/// runtime prop. Converts from an `Option` or a `Vec`.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{FileField, Files};
/// # fn app() -> Element {
/// # let mut avatar = use_signal(|| None::<dioxus::html::FileData>);
/// # let mut attachments = use_signal(Vec::<dioxus::html::FileData>::new);
/// # rsx! {
/// FileField { value: avatar(), onchange: move |files: Files| avatar.set(files.one()) }
/// FileField { value: attachments(), multiple: true, onchange: move |files: Files| attachments.set(files.into_vec()) }
/// # } }
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Files(Vec<FileData>);

impl Files {
    pub fn new(files: Vec<FileData>) -> Self {
        Self(files)
    }

    /// The first file, which is the whole value of a single-file field.
    pub fn one(&self) -> Option<FileData> {
        self.0.first().cloned()
    }

    pub fn into_vec(self) -> Vec<FileData> {
        self.0
    }

    /// Drops the file at `index`, for the default chip's x.
    pub(super) fn without(&self, index: usize) -> Self {
        let mut files = self.0.clone();
        if index < files.len() {
            files.remove(index);
        }
        Self(files)
    }
}

/// A picked or dropped file the field refused, handed to `onreject`.
#[derive(Clone, Debug, PartialEq)]
pub struct FileRejection {
    pub file: FileData,
    pub reason: RejectReason,
}

/// Why a `FileField` refused a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectReason {
    /// Not a type `accept` lists.
    Type,
    /// Past the first on a field without `multiple`.
    TooMany,
}

/// A file's size for a human: powers of 1000, one decimal past a kilobyte,
/// in the localization's units.
pub(super) fn format_size(bytes: u64, words: &FileFieldLabels, decimal_separator: &str) -> String {
    let units = &words.size_units;
    let mut size = bytes as f64;
    let mut unit = 0;
    // Compared as shown, so 999,999 bytes is 1.0 MB, not 1000.0 kB.
    while (size * 10.0).round() >= 10_000.0 && unit + 1 < units.len() {
        size /= 1000.0;
        unit += 1;
    }
    match unit {
        0 => format!("{bytes} {}", units[0]),
        _ => {
            let number = format!("{size:.1}").replacen('.', decimal_separator, 1);
            format!("{number} {}", units[unit])
        }
    }
}

impl std::ops::Deref for Files {
    type Target = [FileData];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<FileData>> for Files {
    fn from(files: Vec<FileData>) -> Self {
        Self(files)
    }
}

impl From<Option<FileData>> for Files {
    fn from(file: Option<FileData>) -> Self {
        Self(file.into_iter().collect())
    }
}

impl From<FileData> for Files {
    fn from(file: FileData) -> Self {
        Self(vec![file])
    }
}

impl FromIterator<FileData> for Files {
    fn from_iter<T: IntoIterator<Item = FileData>>(files: T) -> Self {
        Self(files.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_reads_the_way_a_file_manager_shows_it() {
        let size = |bytes| format_size(bytes, &FileFieldLabels::ENGLISH, ".");
        assert_eq!(size(0), "0 B");
        assert_eq!(size(999), "999 B");
        assert_eq!(size(1_000), "1.0 kB");
        assert_eq!(size(12_345), "12.3 kB");
        assert_eq!(size(999_949), "999.9 kB");
        assert_eq!(size(999_999), "1.0 MB");
        assert_eq!(size(999_999_999), "1.0 GB");
        assert_eq!(size(5_400_000), "5.4 MB");
        assert_eq!(size(2_000_000_000), "2.0 GB");
    }

    /// Todos 656, 689 and 709: the units are the localization's, the
    /// separator the formats'.
    #[test]
    fn a_size_reads_in_the_localized_units() {
        let french = FileFieldLabels {
            size_units: ["o", "ko", "Mo", "Go", "To"],
            ..FileFieldLabels::ENGLISH
        };
        assert_eq!(format_size(12, &french, ","), "12 o");
        assert_eq!(format_size(5_400_000, &french, ","), "5,4 Mo");
    }
}
