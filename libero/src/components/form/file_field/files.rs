//! What a `FileField` holds: none, one, or many.

use dioxus::html::FileData;

/// A `FileField`'s value, in either arity.
///
/// One type for both, because `multiple` is a runtime prop: a single-file
/// field holds at most one entry and a multi-file one holds any number. The
/// `From` impls are what let a call site write whichever it has -
/// `From<Option<FileData>> for Vec<FileData>` cannot exist, both being foreign.
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

    /// Keeps only the first file, for a field that takes one.
    pub(super) fn truncated(self, multiple: bool) -> Self {
        match multiple {
            true => self,
            false => Self(self.0.into_iter().take(1).collect()),
        }
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

/// A file's size for a human, in the units a file manager shows: powers of
/// 1000, one decimal once past a kilobyte.
pub(super) fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1000.0 && unit + 1 < UNITS.len() {
        size /= 1000.0;
        unit += 1;
    }
    match unit {
        0 => format!("{bytes} B"),
        _ => format!("{size:.1} {}", UNITS[unit]),
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
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(999), "999 B");
        assert_eq!(format_size(1_000), "1.0 kB");
        assert_eq!(format_size(12_345), "12.3 kB");
        assert_eq!(format_size(5_400_000), "5.4 MB");
        assert_eq!(format_size(2_000_000_000), "2.0 GB");
    }
}
