//! `Table` toolbar pieces (1416): Export hands over every page's rows in the
//! shown columns, Columns hides one, Density sets the row height.

use anyhow::{Result, bail};
use chromiumoxide::cdp::browser_protocol::browser::{
    SetDownloadBehaviorBehavior, SetDownloadBehaviorParams,
};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

/// Clicks the open menu's entry reading `label`, once it shows (todo 1502).
async fn pick<D: Driver>(d: &mut D, label: &str) -> Result<()> {
    let mut found = None;
    eventually(d, &format!("the {label} entry"), async |d| {
        for index in 0..12 {
            let item = format!("[role^=menuitem][data-menu-index=\"{index}\"]");
            if d.exists(&item).await? && d.text(&item).await?.trim() == label {
                found = Some(item);
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await?;
    d.click(&found.expect("the wait held")).await
}

async fn open<D: Driver>(d: &mut D, tool: &str) -> Result<()> {
    d.click(&format!("[data-table-tool={tool}]")).await?;
    eventually(d, "the menu to open", async |d| {
        d.exists("[role^=menuitem]").await
    })
    .await
}

async fn the_pieces_work<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("[data-table-tool=export]").await?;
    eventually_text(
        d,
        "#csv",
        "Name,Stock|Apple,12|Banana,0|Cherry,3|",
        "an export over both pages",
    )
    .await?;
    // Todo 1460: the export changes nothing on screen, so it is said.
    eventually_text(
        d,
        "[role=status]",
        "Exported 3 rows",
        "the export announcement",
    )
    .await?;

    open(d, "columns").await?;
    pick(d, "Stock").await?;
    eventually(d, "Stock to hide", async |d| {
        Ok(d.text("thead").await?.contains("Name") && !d.text("thead").await?.contains("Stock"))
    })
    .await?;
    // Todo 1460: the last shown column's checkbox is off and says why.
    const NAME: &str = "[role=menuitemcheckbox][aria-disabled=true]";
    eventually(d, "Name's checkbox to turn off", async |d| {
        d.exists(NAME).await
    })
    .await?;
    let reason = d.attr(NAME, "aria-describedby").await?.unwrap_or_default();
    if d.text(&format!("[id=\"{reason}\"]")).await? != "One column stays shown" {
        bail!("the last column's checkbox is described by {reason:?}, not the reason");
    }
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the menu to close", async |d| {
        Ok(!d.exists("[role^=menuitem]").await?)
    })
    .await?;
    d.click("[data-table-tool=export]").await?;
    eventually_text(
        d,
        "#csv",
        "Name|Apple|Banana|Cherry|",
        "an export without Stock",
    )
    .await?;

    // A cell: Blitz lays out no box for a `tr`.
    const CELL: &str = "tbody tr > *";
    let before = d.rect(CELL).await?.height;
    open(d, "density").await?;
    pick(d, "Comfortable").await?;
    eventually_text(d, "#density", "Lg", "a density pick").await?;
    eventually(d, "taller rows", async |d| {
        Ok(d.rect(CELL).await?.height > before + 2.0)
    })
    .await
}

e2e::scenario!(
    the_toolbar_pieces_export_hide_columns_and_set_the_density,
    "/table-toolbar",
    the_pieces_work
);

/// 1448: the Filters piece sits where the toolbar puts it, opens the panel
/// under itself and takes the focus back on Escape.
async fn the_filter_piece_opens_the_panel<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const BUTTON: &str = "[data-filter-panel-button]";
    let (button, density) = (
        d.rect(BUTTON).await?,
        d.rect("[data-table-tool=density]").await?,
    );
    if button.x <= density.x || button.x >= d.rect("[data-table-tool=export]").await?.x {
        bail!("the Filters button is not between Density and Export");
    }
    d.click(BUTTON).await?;
    eventually(d, "the panel under the button", async |d| {
        Ok(d.exists("[data-filter-panel]").await?
            && d.rect("[data-filter-panel]").await?.y >= button.y + button.height)
    })
    .await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the closed panel", async |d| {
        Ok(!d.exists("[data-filter-panel]").await?)
    })
    .await?;
    eventually_focused(d, BUTTON, "Escape").await
}

e2e::scenario!(
    the_filter_piece_opens_the_panel_under_itself,
    "/table-toolbar",
    the_filter_piece_opens_the_panel
);

/// 2084: in a flex parent narrower than the table the toolbar wraps at the parent's width.
#[test]
fn the_toolbar_wraps_in_a_narrow_flex_parent() {
    block_on(async {
        let fixture = Fixture::open("/table-toolbar", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "[data-toolbar]").await.unwrap();
        let outside: Vec<String> = page
            .evaluate(
                "(() => { const root = document.querySelector('[data-toolbar]').parentElement; \
                 Object.assign(root.parentElement.style, { display: 'flex', width: '160px' }); \
                 root.querySelector('table').style.minWidth = '600px'; \
                 const edge = root.parentElement.getBoundingClientRect().right + 0.5; \
                 return [...root.querySelector('[data-toolbar]').children] \
                   .filter(tool => tool.getBoundingClientRect().right > edge) \
                   .map(tool => tool.textContent.trim()); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(outside.is_empty(), "past the parent's edge: {outside:?}");
        fixture.close().await.unwrap();
    });
}

/// 2016: Export through `save_file` downloads the CSV on the web, named and whole.
#[test]
fn an_export_saved_with_save_file_downloads_the_csv() {
    block_on(async {
        let dir = std::env::temp_dir().join(format!("e2e-save-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let fixture = Fixture::open("/table-toolbar/save", Viewport::Desktop)
            .await
            .unwrap();
        fixture
            .page
            .execute(
                SetDownloadBehaviorParams::builder()
                    .behavior(SetDownloadBehaviorBehavior::Allow)
                    .download_path(dir.to_string_lossy())
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap();
        pointer::click(&fixture.page, "[data-table-tool=export]")
            .await
            .unwrap();
        let file = dir.join("fruit.csv");
        wait::until("fruit.csv with the CSV", || async {
            Ok(std::fs::read_to_string(&file).ok().as_deref()
                == Some("Name,Stock\r\nApple,12\r\nBanana,0\r\nCherry,3\r\n"))
        })
        .await
        .unwrap();
        fixture.console.assert_clean("saving the export").unwrap();
        fixture.close().await.unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    });
}
