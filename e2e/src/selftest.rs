//! The suite's self-test: a pass must fail on a broken or planted fixture, for the reason named.

use anyhow::{Result, bail};
use chromiumoxide::Page;

use crate::{Fixture, Viewport, wait};

/// Opens `route`, plants `defect` (JavaScript, run once the app has mounted), runs `check`
/// and requires an error containing `because`: a bare `is_err()` also passes on a check
/// that gave up earlier for another reason. Panics otherwise.
///
/// ```ignore
/// selftest::must_fail("/broken/focus-ring", None, "produced no visible ring", async |page| {
///     focus::assert_focus_ring(page, "#no-ring", 5).await.map(|_| ())
/// })
/// .await;
/// ```
pub async fn must_fail(
    route: &str,
    defect: Option<&str>,
    because: &str,
    check: impl AsyncFn(&Page) -> Result<()>,
) {
    match caught(route, defect, because, check).await {
        Ok(error) => println!("the defect on {route} correctly caught: {error}"),
        Err(error) => panic!("{error}"),
    }
}

/// `must_fail` without the panic: the caught error, or why the check did not catch it.
pub async fn caught(
    route: &str,
    defect: Option<&str>,
    because: &str,
    check: impl AsyncFn(&Page) -> Result<()>,
) -> Result<String> {
    // A short budget first; a wrong reason may be a setup step timed out, so it reruns longer.
    let mut quick = None;
    for share in [wait::QUICK_FAILURE_SHARE, 3] {
        let fixture = Fixture::open(route, Viewport::Desktop)
            .await
            .map_err(|e| anyhow::anyhow!("opening {route}: {e}"))?;
        if let Some(defect) = defect
            && let Err(e) = fixture.page.evaluate(defect).await
        {
            let _ = fixture.close().await;
            bail!("planting the defect on {route}: {e}");
        }

        let outcome = wait::expecting_failure_in(share, check(&fixture.page)).await;
        let _ = fixture.close().await;

        let Err(error) = outcome else {
            bail!(
                "the check stayed green on {route}, which is broken on purpose. The pass \
                 cannot see the defect it exists for, and every component relying on it is \
                 unguarded."
            );
        };
        let error = format!("{error:#}");
        if error.contains(because) {
            return Ok(error);
        }
        if let Some(quick) = &quick {
            bail!(
                "the check failed on {route}, but not for the broken reason.\n  expected: \
                 {because:?}\n  got: {error}\n  at the short budget: {quick}"
            );
        }
        quick = Some(error);
    }
    unreachable!("the second share returns or bails")
}
