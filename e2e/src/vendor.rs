//! Extracts `vendor/axe.zip` (axe-core 4.10.2 with its MPL-2.0 licence, which must travel with it).
//! Failure is a hard error: a contrast pass without axe would assert nothing.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

const AXE: &str = "axe.min.js";
const LICENCE: &str = "LICENSE-MPL-2.0.txt";
/// Below this, the file is a truncated extraction.
const AXE_MINIMUM_BYTES: u64 = 400_000;

fn vendor_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor")
}

/// Extracts the archive unless a plausibly sized file is there; returns the `axe.min.js` path.
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
