//! `Carousel`: a key pressed in a slide belongs to the slide (375); each check asserts the
//! control acted and the strip stayed. Desktop only: smooth scroll is unreliable in background.

use anyhow::{Result, bail, ensure};
use chromiumoxide::Page;
use e2e::archetypes::reset_tab_position;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;
use e2e::passes::pointer;
use e2e::passes::target_size::MINIMUM;
use e2e::suite::Step;
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, wait};

/// The slide pads by exactly the ring's reach, so a fractional slide width can
/// round the clip a layout unit (1/64px) inside it.
const SUBPIXEL: f64 = 0.5;

/// Todo 2577: a replaced child (`video controls`) in a slide paints its outset ring in the padding.
#[test]
fn a_focused_video_slide_keeps_its_ring() {
    block_on(async {
        let fixture = Fixture::open("/carousel/replaced", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#clip-1", 10).await.unwrap();
        let outline: Vec<String> = e2e::js(
            page,
            "(() => { const s = getComputedStyle(document.activeElement); \
               return [s.outlineStyle, s.outlineColor]; })()",
        )
        .await;
        // A bare `video` keeps the browser's own ring; it must be drawn, and not transparent.
        assert_ne!(outline[0], "none", "{outline:?}");
        assert_ne!(outline[1], "rgba(0, 0, 0, 0)", "{outline:?}");
        let clipped: f64 = e2e::js(
            page,
            crate::image_list::ring_clipped("[aria-roledescription=carousel]"),
        )
        .await;
        assert!(
            clipped < SUBPIXEL,
            "the ring runs {clipped}px past the clip"
        );
        fixture
            .console
            .assert_clean("a focused video slide")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 618, 619: the slide and the track clipped the outset ring of a slide's content.
/// Checks every visible slide's button, the first flush with the track's left edge.
#[test]
fn a_focused_slide_keeps_its_ring() {
    block_on(async {
        let fixture = Fixture::open("/carousel/loop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "[aria-roledescription=slide] button", 10)
            .await
            .unwrap();
        for stop in 0..3 {
            if stop > 0 {
                keyboard::press(page, keyboard::TAB).await.unwrap();
            }
            let clipped: f64 = page
                .evaluate_bounded(crate::image_list::ring_clipped(
                    "[aria-roledescription=carousel]",
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                clipped < SUBPIXEL,
                "slide button {stop}: the ring runs {clipped}px past the clip"
            );
        }
        fixture.close().await.unwrap();

        // Todo 619: nested, not a slide's direct child, and flush with its edge.
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        reach(page, BUTTON).await.unwrap();
        let clipped: f64 = page
            .evaluate_bounded(crate::image_list::ring_clipped(
                "[aria-roledescription=carousel]",
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            clipped < SUBPIXEL,
            "the nested slide button's ring runs {clipped}px past the clip"
        );
        fixture.close().await.unwrap();
    });
}

/// The scroll container, tab stop and key handler. Not `[role=group]` alone: every slide
/// is one too.
pub const TRACK: &str = "[aria-roledescription=carousel] [role=group][aria-describedby]";
pub(super) const TEXT: &str = "[aria-roledescription=slide] input[type=text]";
pub(super) const SLIDER: &str = "[aria-roledescription=slide] [role=slider]";
pub(super) const RANGE: &str = "#raw-range";
pub(super) const RADIO_A: &str = "#raw-radio-a";
pub(super) const RADIO_B: &str = "#raw-radio-b";
pub(super) const BUTTON: &str = "#slide-button";
/// The two chevrons. They are the only buttons in the component that point at
/// the track with `aria-controls`.
const CONTROLS: &str = "[aria-roledescription=carousel] button[aria-controls]";
/// The dots. Their ids are `<track id>-indicator-<n>`, and the track id is
/// generated, so the stable half of it is what this matches.
pub(super) const INDICATORS: &str = "[aria-roledescription=carousel] button[id*='-indicator-']";
pub(super) const NEXT: &str = "[aria-roledescription=carousel] button[aria-label='Next slide']";
/// The second slide once current. At rest this matches nothing, as `Suite` requires of a
/// state's settle selector.
const SECOND_CURRENT: &str = "[aria-roledescription=slide]:nth-of-type(2)[data-current]";

/// The track, the text field, the slider, the range, the radio and the button,
/// with room to spare.
const TAB_BUDGET: usize = 10;

/// The index of the slide carrying `data-current`. Not `scrollLeft`: a smooth scroll never
/// lands in a `background: true` page, so it stays 0 (`codebase/e2e-harness`).
pub(super) const INDEX: &str = "[...document.querySelectorAll('[aria-roledescription=slide]')]\
                     .findIndex(el => el.hasAttribute('data-current'))";

const FIRST_SLIDE: &str = "[aria-roledescription=slide]:nth-of-type(1)";

/// Tab to `selector` from the top of the document (`blur()` does not reset the Tab
/// position, see `archetypes::reset_tab_position`); asserts reachability on the way.
pub(super) async fn reach(page: &Page, selector: &str) -> Result<()> {
    reset_tab_position(page).await?;
    keyboard::tab_to(page, selector, TAB_BUDGET).await?;
    Ok(())
}

pub(super) async fn index(page: &Page) -> Result<i64> {
    let index: i64 = page.evaluate_bounded(INDEX).await?.into_value()?;
    if index < 0 {
        bail!("no slide carries data-current, so the carousel's index cannot be read");
    }
    Ok(index)
}

async fn an_arrow_on_the_track_moves<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRACK).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "ArrowRight on the track to reach slide 2", async |d| {
        d.exists(SECOND_CURRENT).await
    })
    .await
}

/// An arrow in a field inside a slide stays the field's (`typing_target`,
/// `arrow_target`): the first slide is still the current one.
async fn an_arrow_in_a_field_stays<D: Driver>(d: &mut D, field: &str) -> Result<()> {
    d.focus(field).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    // An arrow on the track, handled after it, reaches slide 2 only if the first moved nothing.
    d.focus(TRACK).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(
        d,
        &format!("the track's arrow to reach slide 2 after one in {field}"),
        async |d| d.exists(SECOND_CURRENT).await,
    )
    .await
}

/// Todo 1178: with the caret at the field's end, WebKitGTK scrolled the track on an
/// ArrowRight the caret could not follow.
async fn an_arrow_at_the_fields_end_stays<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TEXT).await?;
    d.press(keyboard::END).await?;
    let before = d.rect(FIRST_SLIDE).await?.x;
    d.press(keyboard::ARROW_RIGHT).await?;
    // A native scroll moves on a later frame and sends nothing to await before it.
    d.frame().await?;
    let after = d.rect(FIRST_SLIDE).await?.x;
    ensure!(
        (after - before).abs() < 1.0 && d.is_focused(TEXT).await?,
        "{:?}: an ArrowRight at the field's end scrolled the track from {before} to {after}",
        d.platform()
    );
    Ok(())
}

async fn an_arrow_in_the_text_field_stays<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    an_arrow_in_a_field_stays(d, TEXT).await
}

async fn an_arrow_on_the_range_stays<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    an_arrow_in_a_field_stays(d, RANGE).await
}

e2e::scenario!(
    an_arrow_on_the_track_moves_to_the_next_slide,
    "/carousel",
    an_arrow_on_the_track_moves
);
e2e::scenario!(
    an_arrow_in_a_text_field_stays_in_the_field,
    "/carousel",
    an_arrow_in_the_text_field_stays
);
e2e::scenario!(
    an_arrow_at_a_text_fields_end_leaves_the_track,
    "/carousel",
    an_arrow_at_the_fields_end_stays
);
e2e::scenario!(
    an_arrow_on_a_range_stays_on_the_range,
    "/carousel",
    an_arrow_on_the_range_stays
);

/// Todo 3137: the field-edge guard leaves Up/Down to Android's spatial navigation, the
/// only way out of a field on a D-pad-only device.
mod an_arrow_down_leaves_an_empty_text_field {
    #[cfg(feature = "android")]
    #[test]
    fn android() {
        use e2e::driver::{Android, Driver, eventually};

        e2e::android::block_on(async {
            let mut d = Android::open("/carousel").await.unwrap();
            d.focus(super::TEXT).await.unwrap();
            d.evaluate(&format!(
                "document.querySelector({:?}).value = ''",
                super::TEXT
            ))
            .await
            .unwrap();
            // The first D-pad key may only switch spatial navigation on.
            eventually(&mut d, "ArrowDown to leave the empty field", async |d| {
                if !d.is_focused(super::TEXT).await? {
                    return Ok(true);
                }
                d.press(e2e::passes::keyboard::ARROW_DOWN).await?;
                Ok(!d.is_focused(super::TEXT).await?)
            })
            .await
            .unwrap();
            d.finish("carousel").await.unwrap();
        });
    }
}

/// The generic battery (381), at rest and advanced: every state flips on the first advance.
/// No `targets(INDICATORS)`: the pass ignores the dots' 24px `::before` (see the hit test).
#[test]
fn it_meets_the_baseline() {
    Suite::new("carousel", "/carousel")
        .focusable(TRACK)
        .targets(CONTROLS)
        // `current` follows the scroll; a smooth scroll that never lands in a background
        // page puts the index back, and the snapshot flaked at 390px (375).
        .reduced_motion()
        .state("advanced", &[Step::Click(NEXT)], SECOND_CURRENT)
        .run();
}

/// WCAG 2.5.8 on the dots' 24px `::before` hit area (389): hit-tests 11.5px out, 13px as
/// the failing control, then clicks the third dot 11px above its drawn box.
#[test]
fn the_dots_take_the_pointer_over_24px() {
    block_on(async {
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        let inside = dot_corners(page, MINIMUM / 2.0 - 0.5).await.unwrap();
        assert!(
            !inside.is_empty() && inside.iter().all(|dot| dot.iter().all(|&hit| hit)),
            "a corner of a dot's 24px hit area missed the dot: {inside:?}"
        );
        let outside = dot_corners(page, MINIMUM / 2.0 + 1.0).await.unwrap();
        assert!(
            outside.iter().all(|dot| dot.iter().all(|&hit| !hit)),
            "a dot took the pointer outside its 24px hit area: {outside:?}"
        );

        let third = dot_centre(page, 2).await.unwrap();
        let at = pointer::Point {
            x: third.x,
            y: third.y - 11.0,
        };
        pointer::drag(page, at, at, 1).await.unwrap();
        wait::until(
            "a click on the third dot's hit area to go there",
            || async move { Ok(index(page).await? == 2) },
        )
        .await
        .unwrap_or_else(|e| panic!("a click 11px above the third dot did not select it: {e}"));

        fixture.console.assert_clean("clicks on the dots").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3053: Home on the first dot and End on the last stay the page's.
#[test]
fn home_and_end_at_the_ends_leave_the_page_to_scroll() {
    block_on(async {
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        // The dots follow the slide's inputs and the chevrons.
        keyboard::tab_to(
            page,
            "[aria-roledescription=carousel] button[id$='-indicator-0']",
            12,
        )
        .await
        .unwrap();
        crate::accordion::assert_keys_left_to_page(page, &[keyboard::HOME]).await;
        keyboard::press(page, keyboard::END).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.id.endsWith('-indicator-2')",
            "focus on the last dot",
        )
        .await
        .unwrap();
        crate::accordion::assert_keys_left_to_page(page, &[keyboard::END]).await;
        fixture.console.assert_clean("the ends").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Per dot, whether each corner of a square `offset` out from its centre
/// hit-tests to that dot.
async fn dot_corners(page: &Page, offset: f64) -> Result<Vec<Vec<bool>>> {
    Ok(page
        .evaluate_bounded(format!(
            "[...document.querySelectorAll({INDICATORS:?})].map(dot => {{ \
             const r = dot.getBoundingClientRect(); \
             const x = r.x + r.width / 2, y = r.y + r.height / 2; \
             return [[-1,-1],[1,-1],[-1,1],[1,1]].map(([dx, dy]) => \
             document.elementFromPoint(x + dx * {offset}, y + dy * {offset}) === dot); }})"
        ))
        .await?
        .into_value()?)
}

/// The centre of the `n`th dot, in viewport coordinates.
async fn dot_centre(page: &Page, n: usize) -> Result<pointer::Point> {
    let (x, y): (f64, f64) = page
        .evaluate_bounded(format!(
            "(() => {{ const r = document.querySelectorAll({INDICATORS:?})[{n}]\
             .getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; }})()"
        ))
        .await?
        .into_value()?;
    Ok(pointer::Point { x, y })
}

/// A clone in the viewport is its slide's only live copy, so it cannot be `aria-hidden`
/// (axe `aria-hidden-focus`: its button took Tab while hidden).
#[test]
fn a_visible_clone_is_not_hidden_from_assistive_tech() {
    block_on(async {
        let fixture = Fixture::open("/carousel/loop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelectorAll('[aria-roledescription=carousel] [inert]').length > 0",
            "the offscreen slides to go inert",
        )
        .await
        .unwrap();
        let hidden_focusable: Vec<String> = page
            .evaluate_bounded(
                "[...document.querySelectorAll('[aria-roledescription=carousel] [aria-hidden=true]')]\
                 .flatMap(h => [...h.querySelectorAll('button, a[href], input, [tabindex]')])\
                 .filter(el => !el.closest('[inert]'))\
                 .map(el => el.textContent)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            hidden_focusable.is_empty(),
            "focusable content under aria-hidden: {hidden_focusable:?}"
        );
        let names: Vec<String> = page
            .evaluate_bounded(
                "[...document.querySelectorAll('[aria-roledescription=slide]')]\
                 .filter(el => !el.closest('[inert]'))\
                 .map(el => el.getAttribute('aria-label'))",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            names,
            ["5 of 5", "1 of 5", "2 of 5"],
            "the live slide groups"
        );
        fixture
            .console
            .assert_clean("a looping carousel at rest")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A looping strip opens on slide 1, drawn where the component thinks it is. A smooth
/// opening scroll never landed in a background page: the drawn slides were `inert`.
#[test]
fn a_looping_strip_opens_on_its_first_slide() {
    block_on(async {
        let fixture = Fixture::open("/carousel/loop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_on_new_document_bounded(format!(
            "window.__carouselSamples = []; \
             requestAnimationFrame(function sample() {{ \
             const t = document.querySelector({TRACK:?}); \
             if (t) window.__carouselSamples.push(t.scrollLeft); \
             if (window.__carouselSamples.length < 120) requestAnimationFrame(sample); }});"
        ))
        .await
        .unwrap();
        page.reload_bounded().await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const t = document.querySelector({TRACK:?})?.getBoundingClientRect(); \
                 const s = document.querySelector('[aria-label=\"1 of 5\"]')?.getBoundingClientRect(); \
                 if (!t || !s) return false; const mid = s.x + s.width / 2; return mid > t.left && mid < t.right; }})()"
            ),
            "slide 1 to be drawn inside the track",
        )
        .await
        .unwrap();
        // Sampled from load: an instant placement has no frame between the two.
        let between: Vec<f64> = page
            .evaluate_bounded(format!(
                "window.__carouselSamples.filter(v => v > 0 && \
                 v < document.querySelector({TRACK:?}).scrollLeft - 1)"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            between.is_empty(),
            "the opening scroll swept through {between:?}"
        );
        fixture
            .console
            .assert_clean("a looping carousel's mount")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A smooth scroll's late settle in a background page reads back a "rotation" autoplay
/// never made. The autoplay units test the timer, so the strip jumps.
pub(super) async fn instant_scroll(page: &Page) {
    page.evaluate_bounded(format!(
        "(() => {{ const s = document.createElement('style'); \
         s.textContent = '{TRACK} {{ scroll-behavior: auto !important; }}'; \
         document.head.append(s); }})()"
    ))
    .await
    .unwrap();
}

/// Forced colours paint every dot `Canvas`: the strip vanished, and with it
/// which slide is current (todo 524).
#[test]
fn the_dots_show_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        // The dots fade their fill; read it once the fade has landed.
        wait::for_js_true(
            page,
            "document.getAnimations().length === 0",
            "the dots' fades to end",
        )
        .await
        .unwrap();
        // The page itself is `Canvas`; a fill the same colour is no fill.
        let [current, other, canvas]: [String; 3] = page
            .evaluate_bounded(format!(
                "(() => {{ const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 const dots = [...document.querySelectorAll({INDICATORS:?})]; \
                 const current = dots.find((d) => d.getAttribute('aria-current') === 'true'); \
                 const other = dots.find((d) => d !== current); \
                 return [getComputedStyle(current).backgroundColor, \
                 getComputedStyle(other).backgroundColor, canvas]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(other, canvas, "a dot's fill is the page's own");
        assert_ne!(current, canvas, "the current dot's fill is the page's own");
        assert_ne!(current, other, "the current dot looks like the others");
        fixture.close().await.unwrap();
    });
}

/// [`open_in`](crate::rtl_keys::open_in), with smooth scrolling off.
async fn open_in(route: &str, dir: &str) -> Fixture {
    let fixture = crate::rtl_keys::open_in(route, dir).await;
    instant_scroll(&fixture.page).await;
    fixture
}

/// Todo 708: a drag towards the start reveals the next slide, leftwards in
/// LTR and rightwards in RTL, and Next scrolls towards the end.
#[test]
fn a_drag_and_a_step_head_for_the_end_in_either_direction() {
    block_on(async {
        for (dir, sign) in [("ltr", -1.0), ("rtl", 1.0)] {
            let fixture = open_in("/carousel/text", dir).await;
            let page = &fixture.page;
            let from = pointer::centre_of(page, TRACK).await.unwrap();
            let to = pointer::Point {
                x: from.x + sign * 300.0,
                y: from.y,
            };
            pointer::drag(page, from, to, 12).await.unwrap();
            wait::until(
                &format!("{dir}: the drag to reach slide 2"),
                || async move { Ok(index(page).await? == 1) },
            )
            .await
            .unwrap();

            pointer::click(page, NEXT).await.unwrap();
            let moved = match dir {
                "rtl" => format!("document.querySelector({TRACK:?}).scrollLeft < -500"),
                _ => format!("document.querySelector({TRACK:?}).scrollLeft > 500"),
            };
            wait::for_js_true(page, &moved, &format!("{dir}: Next to scroll to slide 3"))
                .await
                .unwrap();
            wait::until(&format!("{dir}: Next to reach slide 3"), || async move {
                Ok(index(page).await? == 2)
            })
            .await
            .unwrap();
            fixture.console.assert_clean(dir).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// The status text, which reads the settled slide.
async fn status(page: &Page) -> String {
    e2e::js(
        page,
        format!(
            "(() => {{ const t = document.querySelector({TRACK:?}); \
             return document.getElementById(t.getAttribute('aria-describedby')).textContent; }})()"
        ),
    )
    .await
}

/// Drags the narrow `drag-end` strip far past its end; the browser clamps at the last snap point.
/// Small steps: the move that starts the drag has to land on the track, not on Previous.
async fn drag_past_the_end(page: &Page) {
    let from = pointer::centre_of(page, TRACK).await.unwrap();
    let to = pointer::Point { x: 20.0, y: from.y };
    pointer::drag(page, from, to, 30).await.unwrap();
}

/// Todo 3117: a drag released on the last snap point scrolls no more, so no `scrollend`
/// comes; the release itself settles the status.
#[test]
fn a_drag_released_at_the_end_settles() {
    block_on(async {
        let fixture = Fixture::open("/carousel/drag-end", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        drag_past_the_end(page).await;
        wait::until("the status to name the last slide", || async move {
            Ok(status(page).await == "Slide 4 of 4")
        })
        .await
        .unwrap();
        fixture.console.assert_clean("a drag to the end").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3117: a looping strip dragged onto its trailing clone crosses the seam to the
/// real slide on release, so the next step does not sweep back across every slide.
#[test]
fn a_drag_released_on_a_clone_crosses_the_seam() {
    block_on(async {
        let fixture = Fixture::open("/carousel/drag-loop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        let home = scroll_left(page).await;
        page.evaluate_bounded(format!(
            "(() => {{ const t = document.querySelector({TRACK:?}); window.__far = 0; \
             t.addEventListener('scroll', () => window.__far = Math.max(window.__far, t.scrollLeft)); }})()"
        ))
        .await
        .unwrap();
        drag_past_the_end(page).await;
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const t = document.querySelector({TRACK:?}); \
                 return window.__far >= t.scrollWidth - t.clientWidth - 1 \
                 && Math.abs(t.scrollLeft - {home}) < 1; }})()"
            ),
            "the drag to reach the clone, and its release to jump to slide 1",
        )
        .await
        .unwrap();
        assert_eq!(status(page).await, "Slide 1 of 4");
        fixture.console.assert_clean("a drag onto a clone").unwrap();
        fixture.close().await.unwrap();
    });
}

const PREVIOUS: &str = "[aria-roledescription=carousel] button[aria-label='Previous slide']";

async fn scroll_left(page: &Page) -> f64 {
    page.evaluate_bounded(format!("document.querySelector({TRACK:?}).scrollLeft"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Runs `act`, which wraps the strip from rest `from` to rest `to`, and returns every
/// `scrollLeft` sampled until it rests on `to`.
async fn wrap_samples(page: &Page, act: impl AsyncFnOnce(), to: f64, what: &str) -> Vec<f64> {
    page.evaluate_bounded(format!(
        "(() => {{ const t = document.querySelector({TRACK:?}); window.__wrap = []; \
         if (window.__wrapOn) return; window.__wrapOn = true; \
         t.addEventListener('scroll', () => window.__wrap.push(t.scrollLeft)); \
         (function sample() {{ window.__wrap.push(t.scrollLeft); requestAnimationFrame(sample); }})(); }})()"
    ))
    .await
    .unwrap();
    act().await;
    wait::for_js_true(
        page,
        &format!("Math.abs(document.querySelector({TRACK:?}).scrollLeft - {to}) < 1"),
        what,
    )
    .await
    .unwrap();
    page.evaluate_bounded("window.__wrap")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Todo 2357: a looping strip wraps one step onto the adjacent clone, then jumps the
/// seam; it never sweeps back across the slides between its two ends.
#[test]
fn a_looping_wrap_steps_onto_the_clone() {
    block_on(async {
        let fixture = Fixture::open("/carousel/loop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        let home = scroll_left(page).await;
        focus_track(page).await;
        keyboard::press(page, keyboard::END).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({TRACK:?}).scrollLeft > {home} + 1"),
            "End to reach slide 5",
        )
        .await
        .unwrap();
        let end = scroll_left(page).await;
        let between = |samples: &[f64]| -> Vec<f64> {
            samples
                .iter()
                .copied()
                .filter(|&v| v > home + 1.0 && v < end - 1.0)
                .collect()
        };

        let next = wrap_samples(
            page,
            async || pointer::click(page, NEXT).await.unwrap(),
            home,
            "Next on slide 5 to land on slide 1",
        )
        .await;
        assert!(
            next.iter().any(|&v| v > end + 1.0),
            "Next on slide 5 never stepped onto the clone: {next:?}"
        );
        assert!(between(&next).is_empty(), "Next swept back: {next:?}");

        let previous = wrap_samples(
            page,
            async || pointer::click(page, PREVIOUS).await.unwrap(),
            end,
            "Previous on slide 1 to land on slide 5",
        )
        .await;
        assert!(
            previous.iter().any(|&v| v < home - 1.0),
            "Previous on slide 1 never stepped onto the clone: {previous:?}"
        );
        assert!(
            between(&previous).is_empty(),
            "Previous swept: {previous:?}"
        );

        focus_track(page).await;
        let arrow = wrap_samples(
            page,
            async || keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap(),
            home,
            "ArrowRight on slide 5 to land on slide 1",
        )
        .await;
        assert!(
            arrow.iter().any(|&v| v > end + 1.0),
            "ArrowRight on slide 5 never stepped onto the clone: {arrow:?}"
        );
        assert!(
            between(&arrow).is_empty(),
            "ArrowRight swept back: {arrow:?}"
        );
        fixture.console.assert_clean("a looping wrap").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3816: a slide that held the focus goes `inert` on a settle that is no key press, and the
/// track takes the focus instead of `<body>`.
#[test]
fn a_scroll_away_from_the_focused_slide_keeps_the_focus_in_the_carousel() {
    block_on(async {
        let fixture = Fixture::open("/carousel/buttons", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        page.evaluate_bounded(
            "document.querySelector('[aria-roledescription=slide] button').focus()",
        )
        .await
        .unwrap();
        page.evaluate_bounded(format!(
            "(() => {{ const t = document.querySelector({TRACK:?}); t.scrollTo(t.scrollWidth, 0); }})()"
        ))
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector({TRACK:?})"),
            "the track to take the focus from the slide that scrolled away",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn focus_track(page: &Page) {
    page.evaluate_bounded(format!("document.querySelector({TRACK:?}).focus()"))
        .await
        .unwrap();
}

/// Todo 2358: a mouse click on a button in a draggable slide reaches the button; the
/// drag starts only once the pointer moves.
#[test]
fn a_click_in_a_draggable_slide_stays_a_click() {
    block_on(async {
        let fixture = Fixture::open("/carousel/buttons", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "[aria-roledescription=slide] button")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('clicks').textContent === '1'",
            "the click to reach the slide's button",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("a click in a draggable slide")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3034: slides that all fit have nowhere to move, so no status, dots or track tab stop.
#[test]
fn slides_that_all_fit_leave_no_dead_stop() {
    block_on(async {
        let fixture = Fixture::open("/carousel/fits", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelectorAll('[aria-roledescription=carousel]').length === 2",
            "both carousels to render",
        )
        .await
        .unwrap();
        let [status, dots, stops]: [usize; 3] = page
            .evaluate_bounded(
                "(() => { const q = (s) => document.querySelectorAll(`[aria-roledescription=carousel] ${s}`).length; \
                 return [q('[role=status]'), q('[data-slot=indicators]'), q('[tabindex=\"0\"]:not(button)')]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            (status, dots, stops),
            (0, 0, 0),
            "status, dot strips, track stops"
        );
        fixture.console.assert_clean("slides that fit").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4145: `slides_to_scroll: 2` moves Next and Previous two slides, stopping at the last.
#[test]
fn a_step_moves_slides_to_scroll_slides() {
    block_on(async {
        let fixture = Fixture::open("/carousel/scroll-by", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        for (button, expected) in [(NEXT, 2), (NEXT, 4), (PREVIOUS, 2)] {
            pointer::click(page, button).await.unwrap();
            wait::until(
                &format!("the step to reach slide {}", expected + 1),
                || async move { Ok(index(page).await? == expected) },
            )
            .await
            .unwrap();
        }
        fixture.console.assert_clean("a step of two").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4146: a short drag with `drag_free` is not pulled back to a slide on release.
#[test]
fn a_free_drag_rests_between_slides() {
    block_on(async {
        let fixture = Fixture::open("/carousel/drag-free", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        let from = pointer::centre_of(page, TRACK).await.unwrap();
        let to = pointer::Point {
            x: from.x - 120.0,
            y: from.y,
        };
        pointer::drag(page, from, to, 12).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const t = document.querySelector({TRACK:?}); \
                 return getComputedStyle(t).scrollSnapType === 'none' \
                 && t.scrollLeft > 60 && t.scrollLeft < t.clientWidth - 60; }})()"
            ),
            "the strip to rest between slides after the release",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a free drag").unwrap();
        fixture.close().await.unwrap();
    });
}
