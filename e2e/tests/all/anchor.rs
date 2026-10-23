//! `Anchor`: when each `underline` mode draws its line, at rest and under the pointer.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};

const LINE: &str = "text-decoration-line";

async fn lined<D: Driver>(d: &mut D, id: &str) -> Result<bool> {
    Ok(d.style(&format!("#{id}"), LINE)
        .await?
        .contains("underline"))
}

/// Todo 3996: `not-hover` is the mirror of `hover`: underlined at rest, bare under the pointer.
async fn the_underline_modes_draw_at_rest_and_under_the_pointer<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    for (id, at_rest) in [
        ("always", true),
        ("hover", false),
        ("not-hover", true),
        ("never", false),
    ] {
        ensure!(lined(d, id).await? == at_rest, "#{id} at rest");
    }

    for (id, under_pointer) in [
        ("always", true),
        ("hover", true),
        ("not-hover", false),
        ("never", false),
    ] {
        d.hover(&format!("#{id}")).await?;
        eventually(d, &format!("#{id} under the pointer"), async |d| {
            Ok(lined(d, id).await? == under_pointer)
        })
        .await?;
    }
    Ok(())
}

e2e::scenario!(
    the_underline_modes_draw_at_rest_and_under_the_pointer_in_every_renderer,
    "/anchor",
    the_underline_modes_draw_at_rest_and_under_the_pointer
);
