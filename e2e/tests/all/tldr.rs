//! `Tldr`: a menu button whose items are new-tab links to an assistant's chat.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::{keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

/// `a11y_attributes()` gives the trigger a generated id, so the attribute finds it.
const TRIGGER: &str = "[aria-haspopup=menu]";
const ITEMS: &str = "[role=menu] a[role=menuitem]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("tldr", "/tldr")
        .focusable(TRIGGER)
        .targets(TRIGGER)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            "[role=menu]",
        )
        .run();
}

/// Both triggers at every size step the page shows, so each is measured and ringed.
#[test]
fn the_sized_triggers_meet_the_baseline() {
    Suite::new("tldr_sized", "/tldr/sized")
        .focusable(TRIGGER)
        .targets(TRIGGER)
        .run();
}
/// `https://libero-ui.dev/md/menu.md`, percent-encoded.
const PAGE: &str = "https%3A%2F%2Flibero-ui.dev%2Fmd%2Fmenu.md";

fn link(prefix: &str) -> String {
    format!("{ITEMS}[href^=\"{prefix}\"]")
}

async fn open<D: Driver>(d: &mut D) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "the menu to open", async |d| d.exists(ITEMS).await).await
}

async fn every_provider_links_to_its_chat<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, TRIGGER, "TLDR", "mounting").await?;
    // Focus on the first link: the web test below; on Android 958 blocks it.
    open(d).await?;
    for prefix in [
        "https://chat.openai.com/?q=Summarize%20and%20analyze",
        "https://www.google.com/search?udm=50&aep=11&q=Summarize",
        "https://claude.ai/new?q=Summarize",
        "https://www.perplexity.ai/search/new?q=Summarize",
    ] {
        let item = link(prefix);
        ensure!(d.exists(&item).await?, "no link starting {prefix}");
        ensure!(
            d.attr(&item, "target").await?.as_deref() == Some("_blank"),
            "{prefix} opens in this tab"
        );
        ensure!(
            d.attr(&item, "href")
                .await?
                .unwrap_or_default()
                .contains(PAGE),
            "{prefix} does not name the page"
        );
        let mark = d.rect(&format!("{item} svg")).await?;
        ensure!(
            mark.width > 4.0 && mark.width < 40.0 && mark.height > 4.0,
            "{prefix}'s mark is {mark:?}"
        );
    }
    Ok(())
}

async fn escape_closes_and_returns_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the menu to close", async |d| {
        Ok(!d.exists(ITEMS).await?)
    })
    .await?;
    eventually_focused(d, TRIGGER, "Escape").await
}

async fn icon_only_is_named<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let name = async |d: &mut D| {
        Ok(d.attr(TRIGGER, "aria-label").await?.as_deref() == Some("Summarize with AI"))
    };
    eventually(d, "the icon-only trigger's name", name).await?;
    ensure!(
        d.text(TRIGGER).await?.trim().is_empty(),
        "the icon-only trigger shows text"
    );
    open(d).await
}

async fn german_words<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let name = async |d: &mut D| {
        Ok(d.attr(TRIGGER, "aria-label").await?.as_deref() == Some("Mit KI zusammenfassen"))
    };
    eventually(d, "the German trigger name", name).await?;
    open(d).await?;
    let group = d
        .attr("[role=menu] [role=group]", "aria-labelledby")
        .await?
        .unwrap_or_default();
    eventually_text(d, &format!("#{group}"), "Zusammenfassen mit", "opening").await?;
    ensure!(
        d.exists(&link("https://claude.ai/new?q=Fasse%20die"))
            .await?,
        "the prompt is not German"
    );
    Ok(())
}

async fn custom_providers_and_prompt<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, TRIGGER, "Summary", "mounting").await?;
    open(d).await?;
    ensure!(
        !d.exists(&link("https://www.google.com")).await?,
        "Google was not dropped"
    );
    let example = link("https://example.test/chat?q=");
    ensure!(
        d.attr(&example, "href").await?.as_deref()
            == Some(format!("https://example.test/chat?q=Read%20{PAGE}").as_str()),
        "the added provider's prompt"
    );
    ensure!(
        !d.exists(&format!("{example} svg")).await?,
        "a mark with none given"
    );
    ensure!(
        d.exists(&format!("{} svg", link("https://claude.ai")))
            .await?,
        "Claude lost its mark"
    );
    Ok(())
}

async fn trigger_height<D: Driver>(d: &mut D, wrapper: &str) -> Result<f64> {
    Ok(d.rect(&format!("#{wrapper} {TRIGGER}")).await?.height)
}

async fn radius<D: Driver>(d: &mut D, wrapper: &str) -> Result<f64> {
    let value = d
        .style(&format!("#{wrapper} {TRIGGER}"), "border-top-left-radius")
        .await?;
    Ok(value.trim_end_matches("px").parse()?)
}

async fn size_and_radius_reach_the_trigger<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "the triggers to mount", async |d| {
        d.exists("#icon-round [aria-haspopup=menu]").await
    })
    .await?;
    for kind in ["labelled", "icon"] {
        let normal = trigger_height(d, &format!("{kind}-default")).await?;
        let big = trigger_height(d, &format!("{kind}-big")).await?;
        ensure!(
            big > normal,
            "{kind}: xl is {big}px, the default {normal}px"
        );
        let tight = radius(d, &format!("{kind}-tight")).await?;
        let round = radius(d, &format!("{kind}-round")).await?;
        ensure!(tight < round, "{kind}: xs radius {tight}px, xl {round}px");
    }
    Ok(())
}

async fn the_trigger_keeps_its_own_width<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "the triggers to mount", async |d| {
        d.exists("#icon [aria-haspopup=menu]").await
    })
    .await?;
    for kind in ["labelled", "icon"] {
        let width = d.rect(&format!("#{kind} {TRIGGER}")).await?.width;
        ensure!(
            width < 150.0,
            "{kind}: the trigger spans {width}px of its column"
        );
    }
    Ok(())
}

e2e::scenario!(
    a_trigger_in_a_column_keeps_its_own_width,
    "/tldr/column",
    the_trigger_keeps_its_own_width
);
e2e::scenario!(
    size_and_radius_reach_both_triggers,
    "/tldr/sized",
    size_and_radius_reach_the_trigger
);
e2e::scenario!(
    every_provider_links_to_its_chat_in_a_new_tab,
    "/tldr",
    every_provider_links_to_its_chat
);
e2e::scenario!(
    escape_closes_the_menu_and_returns_focus,
    "/tldr",
    escape_closes_and_returns_focus
);
e2e::scenario!(
    the_icon_only_trigger_is_named_summarize_with_ai,
    "/tldr/icon",
    icon_only_is_named
);
e2e::scenario!(
    german_names_the_trigger_the_group_and_the_prompt,
    "/tldr/german",
    german_words
);
e2e::scenario!(
    apps_drop_and_add_providers_and_pass_a_prompt,
    "/tldr/custom",
    custom_providers_and_prompt
);

/// Space and Enter follow the link (the menu does not cancel the click), close, return focus.
#[test]
fn space_and_enter_follow_the_link_and_return_focus() {
    block_on(async {
        let fixture = Fixture::open("/tldr", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, TRIGGER).await.unwrap();
        page.evaluate(format!("document.querySelector('{TRIGGER}').focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "[...document.querySelectorAll('{ITEMS}')].map((a) => a.textContent).join() \
                 === 'ChatGPT,Google AI,Claude,Perplexity' \
                 && [...document.querySelectorAll('{ITEMS}')].every((a) => a.rel === 'noopener noreferrer')"
            ),
            "the four providers",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.textContent === 'Google AI'",
            "ArrowDown moving on",
        )
        .await
        .unwrap();

        // Records whether the menu cancelled the click, then cancels it so no tab opens.
        page.evaluate(
            "window.__clicks = []; document.addEventListener('click', (e) => { \
             if (e.target.closest('a')) { window.__clicks.push(e.defaultPrevented); e.preventDefault(); } })",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "window.__clicks.length === 1 && window.__clicks[0] === false \
                 && !document.querySelector('[role=menu]') && document.activeElement.matches('{TRIGGER}')"
            ),
            "Space following the link, closing the menu and returning focus",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.textContent === 'ChatGPT'",
            "the menu reopened",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "window.__clicks.length === 2 && window.__clicks[1] === false \
                 && document.activeElement.matches('{TRIGGER}')"
            ),
            "Enter following the link",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the tldr fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `menu_parts` reaches the labels of the portaled menu.
#[test]
fn menu_parts_style_the_portaled_menu() {
    block_on(async {
        let fixture = Fixture::open("/tldr/custom", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_visible(page, "[role=menu] [data-slot=label]").await?;
            let style: String = page
                .evaluate(
                    "getComputedStyle(document.querySelector('[role=menu] [data-slot=label]')).fontStyle",
                )
                .await?
                .into_value()?;
            ensure!(style == "italic", "menu_parts missed the label: {style}");
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("tldr menu_parts");

        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}
