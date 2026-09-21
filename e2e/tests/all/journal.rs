//! Todo 364's run journal, proved on made-up runs with no browser. It lives in this binary
//! because `cargo run -p e2e` runs it; the runner's own unit tests nothing runs.

use std::time::Duration;

use e2e::journal::{self, GaveUp, Tally};

/// A directory of this test's own, removed when the test ends. `/tmp` and
/// `E2E_ARTIFACTS` both belong to the live run around us.
struct Scratch(std::path::PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("e2e-journal-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create the scratch directory");
        Self(dir)
    }

    fn waits(&self) -> String {
        std::fs::read_to_string(self.0.join("waits.log")).unwrap_or_default()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn gave_up(
    kind: &str,
    what: &str,
    budget_s: u64,
    elapsed_s: u64,
    slowest_s: f64,
    polls: u32,
) -> GaveUp<'static> {
    // Leaked on purpose: these live for the one assertion that follows and the
    // test process is about to end anyway. It keeps the call sites readable.
    GaveUp {
        kind: Box::leak(kind.to_string().into_boxed_str()),
        how: "expired",
        what: Box::leak(what.to_string().into_boxed_str()),
        budget: Duration::from_secs(budget_s),
        elapsed: Duration::from_secs(elapsed_s),
        slowest_poll: Duration::from_secs_f64(slowest_s),
        polls,
    }
}

#[test]
fn a_summary_line_is_counted_and_anything_else_is_not() {
    let all_fail = Tally::from_summary(
        "test result: FAILED. 0 passed; 77 failed; 1 ignored; 0 measured; 0 filtered out",
    );
    assert_eq!(
        (all_fail.passed, all_fail.failed, all_fail.ran()),
        (0, 77, 77)
    );

    let green = Tally::from_summary(
        "test result: ok. 82 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out",
    );
    assert_eq!((green.passed, green.failed), (82, 0));

    // The line a test's own failure prints must not be mistaken for a summary,
    // or the tally double-counts and the verdict names the wrong shape.
    let not_a_summary = Tally::from_summary("test select::the_list_opens ... FAILED");
    assert_eq!(not_a_summary.ran(), 0);
}

/// Todo 364's discriminator: a 364-shaped run fails everything, one frozen page leaves its
/// siblings green. The wall clock reads the same for both, so the verdict must say which.
#[test]
fn the_verdict_tells_an_all_fail_from_one_frozen_page() {
    let scratch = Scratch::new("verdict");
    let profile = std::path::Path::new("/tmp/e2e-chrome-nonexistent");

    let all_fail = journal::verdict(
        Tally {
            passed: 0,
            failed: 77,
        },
        0,
        &scratch.0,
        profile,
        0,
        None,
    );
    assert!(all_fail.contains("ALL-FAIL"), "{all_fail}");
    assert!(all_fail.contains("todo 364 shape"), "{all_fail}");

    // A `MutationObserver` freeze: 661 s and `4 passed; 1 failed`, an all-fail's wall
    // time without its shape.
    let one_page = journal::verdict(
        Tally {
            passed: 4,
            failed: 1,
        },
        0,
        &scratch.0,
        profile,
        0,
        None,
    );
    assert!(one_page.contains("PARTIAL"), "{one_page}");
    assert!(one_page.contains("not todo 364"), "{one_page}");

    // Red with nothing counted is its own thing: the runner failed before the
    // tests did, and reading it as an all-fail would invent 0-of-0.
    let no_tests = journal::verdict(Tally::default(), 0, &scratch.0, profile, 0, None);
    assert!(no_tests.contains("red without a failed test"), "{no_tests}");

    // Surviving browsers are named in the verdict: an all-fail once left eleven while
    // claiming it had reaped everything.
    let leaked = journal::verdict(
        Tally {
            passed: 0,
            failed: 77,
        },
        0,
        &scratch.0,
        profile,
        11,
        None,
    );
    assert!(
        leaked.contains("11 browser process(es) outlived SIGKILL"),
        "{leaked}"
    );

    // A filtered run failing all its tests is not the 364 shape, so the caveat rides on
    // the claim itself.
    let filtered = journal::verdict(
        Tally {
            passed: 0,
            failed: 2,
        },
        0,
        &scratch.0,
        profile,
        0,
        Some("tabs::"),
    );
    assert!(filtered.contains("filtered to `tabs::`"), "{filtered}");
    assert!(!filtered.contains("todo 364 shape"), "{filtered}");
}

/// Todo 364 (2026-09-20): `5 passed; 87 failed`, all `navigation`, read as PARTIAL because
/// the five were these browserless `journal::` tests. They must not flip the verdict.
#[test]
fn a_unit_that_drives_no_browser_is_not_a_sibling_that_stayed_up() {
    let scratch = Scratch::new("non-browser");
    let profile = std::path::Path::new("/tmp/e2e-chrome-nonexistent");

    assert!(journal::drives_a_browser("modal::it_meets_the_baseline"));
    assert!(!journal::drives_a_browser(
        "journal::the_verdict_names_the_wait"
    ));

    // libtest's per-test lines, plus the summary and a slow run's progress line, which
    // must not be read as one.
    assert_eq!(
        journal::test_outcome("test modal::it_meets_the_baseline ... ok"),
        Some(("modal::it_meets_the_baseline", true))
    );
    assert_eq!(
        journal::test_outcome("test modal::it_meets_the_baseline ... FAILED"),
        Some(("modal::it_meets_the_baseline", false))
    );
    assert_eq!(
        journal::test_outcome("test result: FAILED. 5 passed; 87 failed; 0 ignored"),
        None
    );
    assert_eq!(
        journal::test_outcome(
            "test modal::it_meets_the_baseline has been running for over 60 \
                               seconds"
        ),
        None
    );

    // The real run's numbers: 87 browser tests failed, 5 non-browser passes
    // set aside. It has to read ALL-FAIL.
    let real = journal::verdict(
        Tally {
            passed: 0,
            failed: 87,
        },
        5,
        &scratch.0,
        profile,
        0,
        None,
    );
    assert!(real.contains("ALL-FAIL"), "{real}");
    assert!(real.contains("todo 364 shape"), "{real}");
    assert!(
        real.contains("5 passing test(s) drive no browser"),
        "{real}"
    );
}

/// Which of the four waits expired, and when. Nothing recorded this before,
/// which is why both all-fail events are undiagnosable.
#[test]
fn the_verdict_names_the_wait_that_gave_up_first() {
    let scratch = Scratch::new("waits");

    // Before anything gave up, the verdict must not imply a timeout: a red run
    // whose failures were assertions is a different investigation.
    let quiet = journal::waits_summary(&scratch.0);
    assert!(quiet.contains("no wait gave up"), "{quiet}");

    journal::gave_up_in(
        &scratch.0,
        &gave_up("navigation", "navigating to /select", 120, 120, 120.0, 1),
    );
    journal::gave_up_in(
        &scratch.0,
        &gave_up("read", "[role=listbox] to be visible", 15, 15, 0.03, 480),
    );
    journal::gave_up_in(
        &scratch.0,
        &gave_up("read", "[role=menu] to be visible", 15, 15, 0.02, 490),
    );
    // Written by hand, not via `journal::note`, which would plant this in the live run's
    // own journal.
    std::fs::write(
        scratch.0.join("waits.log"),
        format!(
            "{}2026-09-20T00:00:00.000Z +1.0s note          reaped 88 browser process(es)\n",
            scratch.waits()
        ),
    )
    .expect("append a note");

    let summary = journal::waits_summary(&scratch.0);
    // Counted per kind, so "everything timed out on navigation" and "one page
    // never settled" do not read the same.
    assert!(summary.contains("navigation 1"), "{summary}");
    assert!(summary.contains("read 2"), "{summary}");
    // The first one is the diagnosis; the ones after it are consequences.
    assert!(summary.contains("navigating to /select"), "{summary}");
    // A note is not a wait, and must not be counted as one.
    assert!(!summary.contains("note"), "{summary}");
}

/// A green run's `waits.log` holds the controls' expiries (`negative.rs`, `planted.rs`).
/// They are set aside by thread name and reported, not counted as real ones.
#[test]
fn the_controls_that_give_up_on_purpose_are_set_aside() {
    let scratch = Scratch::new("controls");
    let control = "2026-09-20T09:58:28.600Z +110.5s read          expired budget 15.0s \
                   elapsed 15.0s slowest-poll 0.04s polls 495 thread \
                   negative::the_combobox_pass_catches_a_dangling_activedescendant :: \
                   aria-activedescendant to name an option\n";
    std::fs::write(scratch.0.join("waits.log"), control).expect("write the log");

    let only_controls = journal::waits_summary(&scratch.0);
    assert!(only_controls.contains("no wait gave up"), "{only_controls}");
    assert!(
        only_controls.contains("1 more were the negative and planted controls"),
        "{only_controls}"
    );

    // One real expiry beside them is the whole point: it must be the one the
    // summary names, and the controls must not be counted with it.
    journal::gave_up_in(
        &scratch.0,
        &gave_up("navigation", "navigating to /select", 120, 120, 120.0, 1),
    );
    let with_a_real_one = journal::waits_summary(&scratch.0);
    assert!(
        with_a_real_one.contains("waits that gave up: navigation 1"),
        "{with_a_real_one}"
    );
    assert!(
        with_a_real_one.contains("navigating to /select"),
        "{with_a_real_one}"
    );
    assert!(!with_a_real_one.contains("read 1"), "{with_a_real_one}");

    // A control unit failing at `navigation` is a real failure under a control's name;
    // the 364 run undercounted by 40% that way.
    let scratch = Scratch::new("controls-by-kind");
    let never_loaded = "2026-09-20T11:38:29.020Z +243.4s navigation    failed  budget 120.000s \
                        elapsed 30.000s slowest-poll 30.000s polls 1 thread \
                        negative::the_console_pass_catches_a_warning :: navigating for \
                        http://127.0.0.1:40815/broken/console-warning\n";
    std::fs::write(scratch.0.join("waits.log"), never_loaded).expect("write the log");
    let misfiled = journal::waits_summary(&scratch.0);
    assert!(
        misfiled.contains("waits that gave up: navigation 1"),
        "{misfiled}"
    );
    assert!(
        !misfiled.contains("were the negative and planted"),
        "{misfiled}"
    );
}

/// The line separates blocked from slow: one 118 s call is a page that stopped answering,
/// 480 quick polls a condition that never came true.
#[test]
fn a_blocked_wait_and_a_slow_one_are_distinguishable_in_the_log() {
    let scratch = Scratch::new("shape");
    journal::gave_up_in(
        &scratch.0,
        &gave_up("read", "the combobox to open", 15, 118, 118.0, 1),
    );
    let line = scratch.waits();

    assert!(line.contains("polls 1"), "{line}");
    assert!(line.contains("slowest-poll 118.000s"), "{line}");
    assert!(line.contains("budget 15.000s"), "{line}");
    // A UTC timestamp, because "when" is half of what the todo asked for.
    assert!(
        line.starts_with("20") && line[..24].ends_with('Z'),
        "no timestamp in {line}"
    );
    // libtest names each test's thread after the test, which is how a wait is
    // attributed without plumbing a context through every call.
    assert!(
        line.contains("a_blocked_wait_and_a_slow_one_are_distinguishable_in_the_log"),
        "{line}"
    );
}
