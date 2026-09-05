//! The vendored axe archive, and getting it onto disk.
//!
//! `vendor/axe.zip` holds axe-core 4.10.2 and its MPL-2.0 licence. The licence
//! is **inside the archive** because MPL-2.0 requires the notice to accompany
//! the code; an archive with the code and no licence would be worse than
//! committing the file raw.
//!
//! Both extracted files are gitignored, so the repository carries one 150 KB
//! archive rather than a 553 KB minified blob. After extraction the licence
//! sits at `e2e/vendor/LICENSE-MPL-2.0.txt`; someone auditing licences without
//! running the suite can read it straight out of the archive with
//! `unzip -p e2e/vendor/axe.zip LICENSE-MPL-2.0.txt`.
//!
//! **Extraction failure is a hard error, never a warning.** A contrast pass
//! that silently degrades to "no axe, so no findings" is the
//! check-that-measures-nothing this suite exists to avoid: it would report as
//! coverage while asserting nothing at all.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

const AXE: &str = "axe.min.js";
const LICENCE: &str = "LICENSE-MPL-2.0.txt";
/// Smaller than this and the file on disk is a truncated write from an
/// interrupted extraction, not axe.
const AXE_MINIMUM_BYTES: u64 = 400_000;

fn vendor_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor")
}

/// Extract the archive if needed and return the path to `axe.min.js`.
///
/// Idempotent: a present, plausibly sized file is left alone, so the cost is
/// one `stat` on every run after the first.
pub fn ensure_axe() -> Result<PathBuf> {
    let dir = vendor_dir();
    let axe = dir.join(AXE);

    if let Ok(meta) = std::fs::metadata(&axe)
        && meta.len() >= AXE_MINIMUM_BYTES
    {
        return Ok(axe);
    }

    let archive_path = dir.join("axe.zip");
    let file = std::fs::File::open(&archive_path).with_context(|| {
        format!(
            "cannot open {}. The contrast pass needs axe, and running without it would \
             report a clean result while checking nothing.",
            archive_path.display()
        )
    })?;

    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("{} is not a zip", archive_path.display()))?;

    for name in [AXE, LICENCE] {
        let mut entry = archive
            .by_name(name)
            .with_context(|| format!("{name} is missing from {}", archive_path.display()))?;
        let target = dir.join(name);
        let mut out = std::fs::File::create(&target)
            .with_context(|| format!("cannot write {}", target.display()))?;
        std::io::copy(&mut entry, &mut out).with_context(|| format!("cannot extract {name}"))?;
    }

    let extracted = std::fs::metadata(&axe)
        .with_context(|| format!("{} is missing after extraction", axe.display()))?;
    if extracted.len() < AXE_MINIMUM_BYTES {
        bail!(
            "{} extracted to {} bytes, which is too small to be axe",
            axe.display(),
            extracted.len()
        );
    }

    Ok(axe)
}
