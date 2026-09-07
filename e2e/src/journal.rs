//! What a red run leaves behind, so nobody has to have been watching.
//!
//! Todo 364: twice now the suite has failed every test at once, and both times
//! the only evidence was a wall-clock number and a failure list. Nothing said
//! **which** wait gave up. There are four of them, across two processes, and
//! they mean four different things:
//!
//! * `served-body` - the runner waiting for `dx` to serve the fixture app
//!   rather than its 404 placeholder (`main.rs::wait_for_app`). Expiring here
//!   means the *server* never arrived.
//! * `navigation` - `Fixture::open` creating a page, overriding its viewport
//!   and navigating. Expiring here means the *browser connection* stopped
//!   answering.
//! * `fixture-ready` - the same function waiting for `[data-fixture-ready]`.
//!   The page loaded and the app never mounted.
//! * `read` - every per-assertion poll in `wait.rs`, bounded by
//!   `E2E_TIMEOUT_MS` (15 s by default). Expiring here means one page's own
//!   state never became what the test wanted.
//!
//! Each of those appends one line to `waits.log` in the run's artifacts
//! directory when it gives up, with a UTC timestamp, the offset into the run,
//! the thread (which under libtest is the test's name), and how long the
//! slowest single poll took. That last number is the one that separates the
//! two mechanisms in todo 364: a condition that never came true polls hundreds
//! of times quickly, whereas a frozen renderer or a dead CDP connection sits
//! in **one** call for as long as chromiumoxide's 120 s request timeout
//! allows. A wait whose budget was 15 s and whose slowest poll was 118 s was
//! not slow, it was blocked.
//!
//! A green run's `waits.log` is not empty, and that was a surprise worth
//! writing down: `negative.rs` and `planted.rs` exist to watch a check fail,
//! and each spends a full `E2E_TIMEOUT_MS` doing it - twelve expiries in a
//! measured green run, every one a control. `waits_summary` sets those aside
//! by thread name and says how many it set aside, rather than hiding them.
//! Nothing here prints anything on a passing run.

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// `E2E_RUN_STARTED_MS`: the runner's own start, in epoch milliseconds, so
/// every line in `waits.log` - the runner's and the test process's alike - can
/// carry an offset into the same run.
pub const RUN_STARTED: &str = "E2E_RUN_STARTED_MS";

/// The run's artifacts directory, or `None` when the suite is running outside
/// the runner (a bare `cargo test`), in which case there is nothing to keep.
pub fn dir() -> Option<PathBuf> {
    std::env::var_os("E2E_ARTIFACTS").map(PathBuf::from)
}

/// The journal file itself, for the runner to read back at the end.
pub fn path() -> Option<PathBuf> {
    dir().map(|dir| dir.join("waits.log"))
}

/// One line about a wait that gave up: which kind, what it wanted, and the
/// timings that say whether it was slow or blocked.
pub struct GaveUp<'a> {
    /// `served-body`, `navigation`, `fixture-ready` or `read`.
    pub kind: &'a str,
    /// `expired` (the deadline passed) or `failed` (the poll itself errored).
    pub how: &'a str,
    /// What the wait was for, in the same words the failure message uses.
    pub what: &'a str,
    /// The budget it was given.
    pub budget: Duration,
    /// How long it actually took before giving up.
    pub elapsed: Duration,
    /// The longest single poll. Comparable to `budget`: much larger means the
    /// call was blocked, not that the condition was slow.
    pub slowest_poll: Duration,
    /// How many polls it managed. One poll over a 15 s budget is a block.
    pub polls: u32,
}

/// Append a line to the run's `waits.log`. Silent when there is no artifacts
/// directory, and silent on a write error: instrumentation must never be the
/// reason a run fails.
pub fn gave_up(event: &GaveUp<'_>) {
    if let Some(dir) = dir() {
        gave_up_in(&dir, event);
    }
}

/// `gave_up`, into a directory named outright. The tests use it, because the
/// env var the run reads is process-wide and the suite is threaded.
pub fn gave_up_in(dir: &std::path::Path, event: &GaveUp<'_>) {
    let thread = std::thread::current()
        .name()
        .unwrap_or("unnamed")
        .to_string();
    let line = format!(
        "{} +{:.1}s {:<13} {:<7} budget {:.3}s elapsed {:.3}s slowest-poll {:.3}s polls {} \
         thread {} :: {}\n",
        utc_now(),
        offset_into_run().as_secs_f64(),
        event.kind,
        event.how,
        event.budget.as_secs_f64(),
        event.elapsed.as_secs_f64(),
        event.slowest_poll.as_secs_f64(),
        event.polls,
        thread,
        event.what.replace('\n', " "),
    );
    append(dir, &line);
}

/// A free-form note in the same file, for the runner's own observations (what
/// it reaped, what it could not kill).
pub fn note(text: &str) {
    let Some(dir) = dir() else { return };
    append(
        &dir,
        &format!(
            "{} +{:.1}s note          {}\n",
            utc_now(),
            offset_into_run().as_secs_f64(),
            text.replace('\n', " ")
        ),
    );
}

fn append(dir: &std::path::Path, line: &str) {
    // One `write` of a short line to an O_APPEND file is atomic on Linux, so
    // the runner and the several test threads can share the file without a
    // lock.
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("waits.log"))
        .and_then(|mut file| file.write_all(line.as_bytes()));
}

fn offset_into_run() -> Duration {
    let started_ms: u128 = std::env::var(RUN_STARTED)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Duration::from_millis((now_ms.saturating_sub(started_ms)) as u64)
}

/// `2026-09-20T11:22:33.123Z`, hand-rolled because this crate has no date
/// dependency and a timestamp is the whole point of the file. UTC, said out
/// loud in the `Z`, so two machines' logs compare.
pub fn utc_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let (secs, millis) = (now.as_secs(), now.subsec_millis());
    let (days, rest) = (secs / 86_400, secs % 86_400);
    let (hour, minute, second) = (rest / 3600, (rest % 3600) / 60, rest % 60);
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

/// Days since the Unix epoch to a civil date. Howard Hinnant's `civil_from_days`,
/// which is exact for every date the proleptic Gregorian calendar has.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

/// What a libtest summary line counted.
#[derive(Clone, Copy, Default)]
pub struct Tally {
    pub passed: u64,
    pub failed: u64,
}

impl Tally {
    pub fn ran(self) -> u64 {
        self.passed + self.failed
    }

    /// `test result: ok. 3 passed; 1 failed; 0 ignored; ..`, and zeroes for
    /// any other line.
    pub fn from_summary(line: &str) -> Self {
        let Some((_, counts)) = line
            .strip_prefix("test result: ")
            .and_then(|rest| rest.split_once(". "))
        else {
            return Self::default();
        };
        let mut tally = Self::default();
        for (n, what) in counts.split("; ").filter_map(|count| count.split_once(' ')) {
            let Ok(n) = n.parse::<u64>() else { continue };
            match what {
                "passed" => tally.passed += n,
                "failed" => tally.failed += n,
                _ => {}
            }
        }
        tally
    }
}

impl std::ops::AddAssign for Tally {
    fn add_assign(&mut self, other: Self) {
        self.passed += other.passed;
        self.failed += other.failed;
    }
}

/// Units in `tests/all` that drive no browser.
///
/// Measured 2026-09-20, and the reason this list exists: a real 364 event ran
/// 92 tests, failed all 87 that open a page, and the verdict said **PARTIAL**,
/// because the five `journal::` tests passed. They are this crate's own
/// positive control and they drive no browser on purpose, so they cannot say
/// anything about whether sibling *pages* stayed up. Counted as siblings, they
/// turned the todo's own signature into "ordinary test failure".
pub const NON_BROWSER: &[&str] = &["journal::"];

/// Whether a test opens a fixture page, and so whether its result says
/// anything about the browser.
pub fn drives_a_browser(test: &str) -> bool {
    !NON_BROWSER.iter().any(|unit| test.starts_with(unit))
}

/// `test <name> ... ok` or `... FAILED`, as libtest prints one per test.
///
/// `None` for everything else, which includes the `test result:` summary and
/// the `test <name> has been running for over 60 seconds` progress lines
/// libtest prints when a run is slow - and a 364 run is nothing but those.
pub fn test_outcome(line: &str) -> Option<(&str, bool)> {
    let (name, result) = line.strip_prefix("test ")?.rsplit_once(" ... ")?;
    match result {
        "ok" => Some((name, true)),
        "FAILED" => Some((name, false)),
        _ => None,
    }
}

/// The paragraph a red run ends with: which of the two mechanisms in todo 364
/// this was, which wait gave up first, and where the evidence is.
///
/// The discriminator is the cheap half and it is why this exists: **a
/// 364-shaped run fails everything, and one frozen page leaves its siblings
/// green.** Both look identical from the wall-clock number alone, and the
/// second all-fail was filed against the wrong mechanism for a day because of
/// it.
/// `tally` counts **only the tests that drive a browser**; `set_aside` is how
/// many of the passes did not. See `NON_BROWSER` for why that distinction is
/// load-bearing rather than pedantic.
pub fn verdict(
    tally: Tally,
    set_aside: u64,
    artifacts: &std::path::Path,
    profile: &std::path::Path,
    survivors: usize,
    filter: Option<&str>,
) -> String {
    let shape = match (tally.passed, tally.failed) {
        (_, 0) => "red without a failed test - the tests did not run, or the runner itself failed"
            .to_string(),
        (0, failed) => {
            // A filtered run failing "everything" is two tests, not a suite,
            // and reading it as the 364 signature would be the third wrong
            // diagnosis on this todo. Say so where the claim is made.
            let caveat = match filter {
                Some(filter) => format!(
                    ", but this run was filtered to `{filter}`, so everything means only those \
                     tests and the shape says nothing"
                ),
                None => ". This is the todo 364 shape. A single frozen page takes down its own \
                         page and nothing else, so something every page shares stopped answering \
                         - the browser connection or the fixture server"
                    .to_string(),
            };
            format!("ALL-FAIL: 0 of {failed} browser tests passed{caveat}")
        }
        (passed, failed) => format!(
            "PARTIAL: {failed} failed, {passed} passed. Sibling pages stayed green, so this is \
             ordinary test failure (or one frozen page), not todo 364"
        ),
    };

    let mut lines = vec![format!("e2e: VERDICT {shape}.")];
    if set_aside > 0 {
        // Named rather than silently dropped: a reader who counts the libtest
        // summary against this line must be able to see where the difference
        // went.
        lines.push(format!(
            "e2e: {set_aside} passing test(s) drive no browser ({}) and are not counted as \
             siblings that stayed up.",
            NON_BROWSER.join(", ")
        ));
    }
    lines.push(waits_summary(artifacts));
    if survivors > 0 {
        lines.push(format!(
            "e2e: {survivors} browser process(es) outlived SIGKILL; see \
             {}/browser-processes.txt",
            artifacts.display()
        ));
    }
    lines.push(format!(
        "e2e: evidence kept in {} (test-output.log, waits.log, dx.log, screenshots) and in the \
         Chrome profile {}.",
        artifacts.display(),
        profile.display()
    ));
    lines.join("\n")
}

/// One field of a journal line: `<stamp> +<offset> <kind> <how> .. thread <t> :: <what>`.
fn field(line: &str, n: usize) -> &str {
    line.split_whitespace().nth(n).unwrap_or("?")
}

/// Whether this line is a wait that was **supposed** to give up.
///
/// `negative.rs` and `planted.rs` exist to watch a check fail, and each of
/// them spends a full `E2E_TIMEOUT_MS` doing it - twelve expiries in a green
/// run, measured. Counting those beside a real one would bury the signal in
/// its own positive controls. The thread name is what says so, because under
/// libtest a test's thread is named after it.
///
/// **The kind has to match as well, and that was missed** (2026-09-20). A real
/// 364 run wrote 87 `navigation` failures; 36 of them came from `negative::`
/// and `planted::` threads, which had never reached their deliberate wait
/// because their page never loaded. Filed on the thread name alone, the
/// summary said `navigation 51 (36 more were the controls)` about a run in
/// which there were no controls at all and 87 real failures. Only a `read`
/// expiry is ever deliberate: those units control an assertion, not a
/// navigation.
fn deliberate(line: &str) -> bool {
    if field(line, 2) != crate::wait::READ {
        return false;
    }
    let thread = line
        .split(" thread ")
        .nth(1)
        .and_then(|rest| rest.split(" :: ").next())
        .unwrap_or("");
    thread.starts_with("negative::") || thread.starts_with("planted::")
}

/// Which wait gave up first, how many of each kind, and when.
///
/// This is the line todo 364 asked for: the four waits - the runner's
/// `served-body`, `Fixture::open`'s `navigation` and `fixture-ready`, and the
/// per-read `E2E_TIMEOUT_MS` poll - were indistinguishable in a red run's
/// output, so nobody could say which expired.
pub fn waits_summary(artifacts: &std::path::Path) -> String {
    let nothing = "e2e: no wait gave up - every failure was an assertion, not a timeout.";
    let Ok(log) = std::fs::read_to_string(artifacts.join("waits.log")) else {
        return nothing.into();
    };
    let lines: Vec<&str> = log
        .lines()
        .filter(|line| field(line, 2) != "note")
        .collect();
    let (deliberate_count, gave_up): (usize, Vec<&str>) =
        lines
            .iter()
            .fold((0, Vec::new()), |(count, mut kept), line| {
                if deliberate(line) {
                    (count + 1, kept)
                } else {
                    kept.push(*line);
                    (count, kept)
                }
            });
    let aside = if deliberate_count > 0 {
        format!(
            " ({deliberate_count} more were the negative and planted controls, which give up on \
             purpose)"
        )
    } else {
        String::new()
    };
    if gave_up.is_empty() {
        return format!("{nothing}{aside}");
    }
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for line in &gave_up {
        let kind = field(line, 2);
        match counts.iter_mut().find(|(name, _)| *name == kind) {
            Some((_, n)) => *n += 1,
            None => counts.push((kind, 1)),
        }
    }
    let tally = counts
        .iter()
        .map(|(kind, n)| format!("{kind} {n}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "e2e: waits that gave up: {tally}{aside}. First:\n  {}\n  (all of them in {}/waits.log)",
        gave_up[0],
        artifacts.display()
    )
}

#[cfg(test)]
mod tests {
    use super::civil_from_days;

    #[test]
    fn the_epoch_and_a_leap_day_convert() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-02-29: the century leap year that the naive rule gets wrong.
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
        assert_eq!(civil_from_days(20_716), (2026, 9, 20));
    }
}
