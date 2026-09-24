//! The fixture server's port and readiness, over a hand-rolled HTTP GET.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Child;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

/// Generous: a cold cargo build happens inside this wait.
const BUILD_TIMEOUT: Duration = Duration::from_secs(45 * 60);

/// A free port from the OS (racy until `dx` binds it).
pub(crate) fn free_port() -> Result<u16> {
    // `E2E_PORT` pins it, checked free: a held one ran the suite against another app (todo 364).
    if let Ok(port) = std::env::var("E2E_PORT") {
        let port: u16 = port.parse().context("E2E_PORT is not a port number")?;
        TcpListener::bind(("127.0.0.1", port)).with_context(|| {
            format!("E2E_PORT={port} is already in use; that port belongs to something else")
        })?;
        return Ok(port);
    }
    let listener = TcpListener::bind("127.0.0.1:0").context("bind a free port")?;
    Ok(listener.local_addr()?.port())
}

/// Waits until the fixture app, not dx's placeholder, is served. Fails early when `dx`
/// exits, which is how a fixture compile error looks.
pub(crate) fn wait_for_app(
    base_url: &str,
    title: &str,
    server: &mut Child,
    dx_log: &std::path::Path,
) -> Result<()> {
    let started = Instant::now();
    let deadline = started + BUILD_TIMEOUT;
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
                budget: BUILD_TIMEOUT,
                elapsed: started.elapsed(),
                slowest_poll: slowest,
                polls,
            });
            bail!(
                "timed out after {:.1?} (budget {BUILD_TIMEOUT:?}) waiting for the fixture \
                 app to be servable; got as far as {reached:?} [wait=served-body, {polls} \
                 poll(s), slowest {slowest:.2?}]; see {}",
                started.elapsed(),
                dx_log.display()
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }
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
    let Some(index) = http_get(base_url) else {
        return Readiness::NoServer;
    };
    if !index.contains(title) {
        return Readiness::NotOurApp;
    }
    // `.js`, not the first `src=`: that may be an icon.
    let Some(script) = quoted_ending_in(&index, ".js") else {
        return Readiness::NoScript;
    };
    let Some(js) = http_get(&join(base_url, "", &script)) else {
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
    match http_get(&join(base_url, dir, &bundle)) {
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

/// A one-shot HTTP GET, hand-rolled so no `curl` is needed.
fn http_get(url: &str) -> Option<String> {
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
    stream.read_to_end(&mut body).ok()?;
    Some(String::from_utf8_lossy(&body).into_owned())
}
