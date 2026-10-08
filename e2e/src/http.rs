//! The fixture server's port and readiness, over a hand-rolled HTTP GET.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Child;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

/// Generous: a cold cargo build happens inside this wait.
const BUILD_TIMEOUT: Duration = Duration::from_secs(45 * 60);

/// The first poll waits this long; each next one twice as long, up to [`POLL_MAX`].
const POLL_MIN: Duration = Duration::from_millis(50);
const POLL_MAX: Duration = Duration::from_secs(2);

/// Enough of a response for its status line: the bundle itself is never needed.
const STATUS_ONLY: u64 = 512;
const BODY_LIMIT: u64 = 16 * 1024 * 1024;

/// A free port from the OS (racy until `dx` binds it).
pub(crate) fn free_port() -> Result<u16> {
    // `E2E_PORT` pins it, checked free: a held one ran the suite against another app (todo 364).
    if let Ok(port) = std::env::var("E2E_PORT") {
        return pinned_port(&port);
    }
    let listener = TcpListener::bind("127.0.0.1:0").context("bind a free port")?;
    Ok(listener.local_addr()?.port())
}

fn pinned_port(value: &str) -> Result<u16> {
    let port: u16 = value.parse().context("E2E_PORT is not a port number")?;
    TcpListener::bind(("127.0.0.1", port)).with_context(|| {
        format!("E2E_PORT={port} is already in use; that port belongs to something else")
    })?;
    Ok(port)
}

/// Waits until the fixture app, not dx's placeholder, is served. Fails early when `dx`
/// exits, which is how a fixture compile error looks.
pub(crate) fn wait_for_app(
    base_url: &str,
    title: &str,
    server: &mut Child,
    dx_log: &std::path::Path,
) -> Result<()> {
    wait_for_app_within(BUILD_TIMEOUT, base_url, title, server, dx_log)
}

fn wait_for_app_within(
    budget: Duration,
    base_url: &str,
    title: &str,
    server: &mut Child,
    dx_log: &std::path::Path,
) -> Result<()> {
    let started = Instant::now();
    let deadline = started + budget;
    let (mut polls, mut slowest) = (0u32, Duration::ZERO);
    // Named in the timeout: which link of the chain never appeared.
    let mut reached;
    loop {
        if let Ok(Some(status)) = server.try_wait() {
            let tail = std::fs::read_to_string(dx_log)
                .map(|log| log.lines().rev().take(30).collect::<Vec<_>>().join("\n"))
                .unwrap_or_default();
            bail!(
                "dx exited before the app was served ({status}). Last of {}:\n{tail}",
                dx_log.display()
            );
        }

        let at = Instant::now();
        let stage = app_readiness(base_url, title);
        polls += 1;
        slowest = slowest.max(at.elapsed());
        if stage == Readiness::Ready {
            return Ok(());
        }
        reached = stage;

        if Instant::now() >= deadline {
            // The first of the three waits todo 364 could not tell apart.
            e2e::journal::gave_up(&e2e::journal::GaveUp {
                kind: "served-body",
                how: "expired",
                what: &format!("the fixture app to be servable (got as far as {reached:?})"),
                budget,
                elapsed: started.elapsed(),
                slowest_poll: slowest,
                polls,
            });
            bail!(
                "timed out after {:.1?} (budget {budget:?}) waiting for the fixture \
                 app to be servable; got as far as {reached:?} [wait=served-body, {polls} \
                 poll(s), slowest {slowest:.2?}]; see {}",
                started.elapsed(),
                dx_log.display()
            );
        }
        std::thread::sleep(poll_delay(polls));
    }
}

/// Waits until `dx`'s log holds `line`. A release build serves its bundle while it still
/// optimises and pre-compresses (minutes), and a page loaded then times out (todo 2163).
pub(crate) fn wait_for_log_line(
    line: &str,
    server: &mut Child,
    dx_log: &std::path::Path,
) -> Result<()> {
    wait_for_log_line_within(BUILD_TIMEOUT, line, server, dx_log)
}

fn wait_for_log_line_within(
    budget: Duration,
    line: &str,
    server: &mut Child,
    dx_log: &std::path::Path,
) -> Result<()> {
    let deadline = Instant::now() + budget;
    let mut polls = 0u32;
    loop {
        if std::fs::read_to_string(dx_log).is_ok_and(|log| log.contains(line)) {
            return Ok(());
        }
        if let Ok(Some(status)) = server.try_wait() {
            bail!(
                "dx exited ({status}) before logging {line:?}; see {}",
                dx_log.display()
            );
        }
        if Instant::now() >= deadline {
            bail!(
                "dx never logged {line:?} within {budget:?}; see {}",
                dx_log.display()
            );
        }
        polls += 1;
        std::thread::sleep(poll_delay(polls));
    }
}

/// Short at first, so an app that is up within a second is not found 2 s late; a cold build
/// polls at the cap.
fn poll_delay(polls: u32) -> Duration {
    POLL_MIN
        .saturating_mul(
            1u32.checked_shl(polls.saturating_sub(1))
                .unwrap_or(u32::MAX),
        )
        .min(POLL_MAX)
}

/// How far the served app has got. Ordered: each variant means every earlier
/// one already held.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Readiness {
    /// Nothing answered on the port.
    NoServer,
    /// Something else answered: dx's build splash or another app.
    NotOurApp,
    /// Our index.html, but its script is not being served yet.
    NoScript,
    /// The script, but the wasm bundle it names is not being served yet.
    NoBundle,
    /// The index, the script and the bundle are all servable.
    Ready,
}

/// Follows index, script, wasm bundle as served: dx serves the real `index.html` (title
/// included) ~44 s before the bundle exists (todo 364).
fn app_readiness(base_url: &str, title: &str) -> Readiness {
    let Some(index) = http_get(base_url, BODY_LIMIT) else {
        return Readiness::NoServer;
    };
    if !index.contains(title) {
        return Readiness::NotOurApp;
    }
    // `.js`, not the first `src=`: that may be an icon.
    let Some(script) = quoted_ending_in(&index, ".js") else {
        return Readiness::NoScript;
    };
    let Some(js) = http_get(&join(base_url, "", &script), BODY_LIMIT) else {
        return Readiness::NoScript;
    };
    if !is_ok(&js) {
        return Readiness::NoScript;
    }
    // The glue names the bundle bare or rooted; `join` resolves either.
    let Some(bundle) = quoted_ending_in(&js, ".wasm") else {
        return Readiness::NoBundle;
    };
    let dir = script
        .trim_start_matches(['.', '/'])
        .rsplit_once('/')
        .map(|(dir, _)| dir)
        .unwrap_or("");
    match http_get(&join(base_url, dir, &bundle), STATUS_ONLY) {
        Some(response) if is_ok(&response) => Readiness::Ready,
        _ => Readiness::NoBundle,
    }
}

/// A URL for `reference`, which is either rooted (`/./wasm/x.wasm`) or
/// relative to `dir`.
fn join(base_url: &str, dir: &str, reference: &str) -> String {
    let path = reference.trim_start_matches(['.', '/']);
    if reference.contains('/') || dir.is_empty() {
        format!("{base_url}/{path}")
    } else {
        format!("{base_url}/{dir}/{path}")
    }
}

/// The first single- or double-quoted run in `text` that ends with `suffix`.
fn quoted_ending_in(text: &str, suffix: &str) -> Option<String> {
    text.split(['"', '\''])
        .find(|part| part.ends_with(suffix) && !part.contains(['<', '>', ' ']))
        .map(str::to_string)
}

/// Whether a raw HTTP response carries a 2xx status.
fn is_ok(response: &str) -> bool {
    response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .is_some_and(|code| code.starts_with('2'))
}

/// A one-shot HTTP GET, hand-rolled so no `curl` is needed. Reads at most `limit` bytes.
fn http_get(url: &str, limit: u64) -> Option<String> {
    let rest = url.strip_prefix("http://")?;
    let (address, path) = match rest.find('/') {
        Some(at) => (&rest[..at], &rest[at..]),
        None => (rest, "/"),
    };
    let mut stream = TcpStream::connect(address).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .ok()?;
    write!(
        stream,
        "GET {path} HTTP/1.0\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .ok()?;
    let mut body = Vec::new();
    stream.take(limit).read_to_end(&mut body).ok()?;
    Some(String::from_utf8_lossy(&body).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    /// Serves `(path, body)` pairs with a 200 and everything else with a 404, until the process ends.
    fn serve(routes: &'static [(&'static str, &'static str)]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut reader = BufReader::new(&stream);
                let mut request = String::new();
                let _ = reader.read_line(&mut request);
                // The headers are read too, or closing on them resets the client's read.
                let mut header = String::new();
                while reader.read_line(&mut header).is_ok_and(|n| n > 2) {
                    header.clear();
                }
                let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
                let path = path.as_str();
                let reply = match routes.iter().find(|(route, _)| *route == path) {
                    Some((_, body)) => format!("HTTP/1.0 200 OK\r\n\r\n{body}"),
                    None => "HTTP/1.0 404 Not Found\r\n\r\n".to_string(),
                };
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        base_url
    }

    const INDEX: &str = "<title>fixtures</title><script src=\"/assets/app.js\"></script>";

    fn closed_port_url() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}", listener.local_addr().unwrap())
    }

    #[test]
    fn a_poll_starts_short_and_stops_doubling_at_the_cap() {
        let delays: Vec<_> = (1..=8).map(poll_delay).collect();
        assert_eq!(delays[0], POLL_MIN);
        assert_eq!(delays[1], POLL_MIN * 2);
        assert!(delays.windows(2).all(|pair| pair[0] <= pair[1]));
        assert_eq!(delays[7], POLL_MAX);
        assert_eq!(poll_delay(u32::MAX), POLL_MAX);
    }

    #[test]
    fn a_pinned_port_must_be_a_free_number() {
        let held = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = held.local_addr().unwrap().port();
        assert!(pinned_port("nonsense").is_err());
        assert!(
            pinned_port(&port.to_string())
                .unwrap_err()
                .to_string()
                .contains("already in use")
        );
        drop(held);
        // The kernel may hand the freed port to a parallel test, so a few tries.
        let free = (0..5).any(|_| {
            let probe = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = probe.local_addr().unwrap().port();
            drop(probe);
            pinned_port(&port.to_string()).is_ok_and(|pinned| pinned == port)
        });
        assert!(free);
    }

    #[test]
    fn a_reference_joins_rooted_or_relative_to_the_script_directory() {
        assert_eq!(join("http://h", "", "/./a.js"), "http://h/a.js");
        assert_eq!(
            join("http://h", "assets", "x.wasm"),
            "http://h/assets/x.wasm"
        );
        assert_eq!(
            join("http://h", "assets", "/wasm/x.wasm"),
            "http://h/wasm/x.wasm"
        );
    }

    #[test]
    fn the_quoted_run_is_the_first_one_with_the_suffix() {
        let html = "<link href=\"/icon.png\"><script src='/a/app.js'></script> \"b.js\"";
        assert_eq!(quoted_ending_in(html, ".js").as_deref(), Some("/a/app.js"));
        assert_eq!(quoted_ending_in(html, ".wasm"), None);
    }

    #[test]
    fn only_a_2xx_status_is_ok() {
        assert!(is_ok("HTTP/1.0 200 OK\r\n\r\nbody"));
        assert!(is_ok("HTTP/1.1 204 No Content\r\n\r\n"));
        assert!(!is_ok("HTTP/1.0 404 Not Found\r\n\r\n"));
        assert!(!is_ok(""));
    }

    #[test]
    fn a_get_reads_no_more_than_its_limit() {
        let url = serve(&[("/", "0123456789")]);
        let all = http_get(&url, BODY_LIMIT).unwrap();
        assert!(all.ends_with("0123456789"));
        let head = http_get(&url, 8).unwrap();
        assert_eq!(head, "HTTP/1.0");
        assert_eq!(http_get(&closed_port_url(), 8), None);
    }

    #[test]
    fn readiness_names_the_first_link_that_is_missing() {
        assert_eq!(
            app_readiness(&closed_port_url(), "fixtures"),
            Readiness::NoServer
        );
        let splash = serve(&[("/", "<title>dx building</title>")]);
        assert_eq!(app_readiness(&splash, "fixtures"), Readiness::NotOurApp);
        let no_script = serve(&[("/", INDEX)]);
        assert_eq!(app_readiness(&no_script, "fixtures"), Readiness::NoScript);
        let no_wasm_name = serve(&[("/", INDEX), ("/assets/app.js", "export default 1;")]);
        assert_eq!(
            app_readiness(&no_wasm_name, "fixtures"),
            Readiness::NoBundle
        );
        let no_bundle = serve(&[("/", INDEX), ("/assets/app.js", "fetch('app_bg.wasm')")]);
        assert_eq!(app_readiness(&no_bundle, "fixtures"), Readiness::NoBundle);
        let ready = serve(&[
            ("/", INDEX),
            ("/assets/app.js", "fetch('app_bg.wasm')"),
            ("/assets/app_bg.wasm", "\0asm"),
        ]);
        assert_eq!(app_readiness(&ready, "fixtures"), Readiness::Ready);
    }

    fn idle_child() -> Child {
        Command::new("sleep")
            .arg("30")
            .stdout(Stdio::null())
            .spawn()
            .unwrap()
    }

    #[test]
    fn a_served_app_ends_the_wait() {
        let url = serve(&[
            ("/", INDEX),
            ("/assets/app.js", "fetch('app_bg.wasm')"),
            ("/assets/app_bg.wasm", "\0asm"),
        ]);
        let mut server = idle_child();
        let log = std::env::temp_dir().join("e2e-http-test-unused.log");
        let outcome =
            wait_for_app_within(Duration::from_secs(30), &url, "fixtures", &mut server, &log);
        let _ = server.kill();
        let _ = server.wait();
        outcome.unwrap();
    }

    #[test]
    fn the_wait_for_a_log_line_ends_when_it_appears_and_fails_with_a_dead_server() {
        let log = std::env::temp_dir().join(format!("e2e-http-line-{}.log", std::process::id()));
        std::fs::write(&log, "optimising\n").unwrap();
        let mut server = idle_child();
        let writer = {
            let log = log.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(200));
                std::fs::write(&log, "optimising\nBuild completed successfully in 9s\n").unwrap();
            })
        };
        let started = Instant::now();
        wait_for_log_line_within(
            Duration::from_secs(30),
            "Build completed successfully",
            &mut server,
            &log,
        )
        .unwrap();
        assert!(started.elapsed() < Duration::from_secs(10));
        writer.join().unwrap();
        let _ = server.kill();
        let _ = server.wait();

        std::fs::write(&log, "optimising\n").unwrap();
        let mut dead = Command::new("true").spawn().unwrap();
        let _ = dead.wait();
        let error = wait_for_log_line_within(Duration::from_secs(30), "done", &mut dead, &log)
            .unwrap_err()
            .to_string();
        let _ = std::fs::remove_file(&log);
        assert!(error.contains("dx exited"), "{error}");
    }

    #[test]
    fn a_dead_server_fails_the_wait_with_the_tail_of_its_log() {
        let log = std::env::temp_dir().join(format!("e2e-http-test-{}.log", std::process::id()));
        std::fs::write(&log, "compiling\nerror[E0425]: cannot find value\n").unwrap();
        let mut server = Command::new("true").spawn().unwrap();
        let _ = server.wait();
        let error = wait_for_app_within(
            Duration::from_secs(30),
            &closed_port_url(),
            "fixtures",
            &mut server,
            &log,
        )
        .unwrap_err()
        .to_string();
        let _ = std::fs::remove_file(&log);
        assert!(
            error.contains("dx exited before the app was served"),
            "{error}"
        );
        assert!(error.contains("cannot find value"), "{error}");
    }

    #[test]
    fn a_wait_past_its_budget_names_how_far_the_app_got() {
        let url = serve(&[("/", INDEX)]);
        let mut server = idle_child();
        let log = std::env::temp_dir().join("e2e-http-test-unused.log");
        let started = Instant::now();
        let error = wait_for_app_within(
            Duration::from_millis(300),
            &url,
            "fixtures",
            &mut server,
            &log,
        )
        .unwrap_err()
        .to_string();
        let _ = server.kill();
        let _ = server.wait();
        assert!(error.contains("got as far as NoScript"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(10));
    }
}
