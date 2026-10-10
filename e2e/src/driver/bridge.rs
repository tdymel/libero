//! The harness end of the fixture app's `E2E_BRIDGE` (`e2e/fixtures/src/lib.rs`): the app
//! dials a loopback listener and answers JSON lines, `{"ok": ..}` or `{"err": ..}`. Shared by
//! the WebViews with no DevTools socket the harness reaches: [`super::desktop`], [`super::ios`].

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;

/// One connected fixture app.
pub struct Bridge {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    /// The WebView in error messages, e.g. `desktop`.
    name: &'static str,
    /// How long [`Self::wait_until`] waits, and the app gets to connect.
    limit: Duration,
}

impl Bridge {
    /// The app's connection on `listener`, which it opens from `Shell`'s first effect: the ready
    /// signal. `alive` fails the wait early, e.g. when the app exited.
    pub fn accept(
        listener: &TcpListener,
        name: &'static str,
        limit: Duration,
        mut alive: impl FnMut() -> Result<()>,
    ) -> Result<Self> {
        listener.set_nonblocking(true)?;
        let started = Instant::now();
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
            alive()?;
            if started.elapsed() > limit {
                bail!("the fixture app's bridge did not connect within {limit:?}");
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(20)))?;
        Ok(Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: stream,
            name,
            limit,
        })
    }

    /// Runs a JS body (`return` for a value) in the page; its JSON answer.
    pub fn eval(&mut self, body: &str) -> Result<serde_json::Value> {
        // The bridge's own error names no cause: catch and carry the message.
        let body = format!("try {{ {body} }} catch (e) {{ return {{ __e2eError: String(e) }}; }}");
        let mut answer = self.request(&serde_json::to_string(&body)?)?;
        if let Some(error) = answer.get("err") {
            let running: String = body.chars().skip(6).take(120).collect();
            bail!("{} eval failed: {error}, running {running}", self.name);
        }
        let value = answer["ok"].take();
        if let Some(error) = value.get("__e2eError") {
            bail!("{} eval threw: {error}", self.name);
        }
        Ok(value)
    }

    /// One request line out, its `{"ok": ..}` or `{"err": ..}` answer back.
    pub fn request(&mut self, line: &str) -> Result<serde_json::Value> {
        writeln!(self.writer, "{line}")?;
        let mut answer = String::new();
        self.reader
            .read_line(&mut answer)
            .context("read the bridge's answer")?;
        serde_json::from_str(&answer).context("the bridge closed")
    }

    pub fn run(&mut self, body: &str) -> Result<()> {
        self.eval(&format!("{body}; return null;")).map(drop)
    }

    /// A JS expression's value; `undefined` reads as `null`.
    pub fn json<T: DeserializeOwned>(&mut self, expression: &str) -> Result<T> {
        let text = self.eval(&format!("return JSON.stringify({expression}) ?? 'null';"))?;
        let text = text.as_str().context("JSON.stringify gave no string")?;
        Ok(serde_json::from_str(text)?)
    }

    pub fn wait_for(&mut self, selector: &str) -> Result<()> {
        self.wait_until(&format!("{} !== null", element(selector)))
    }

    pub fn wait_until(&mut self, expression: &str) -> Result<()> {
        let started = Instant::now();
        while !self.json::<bool>(expression)? {
            if started.elapsed() > self.limit {
                bail!("{expression} did not hold within {:?}", self.limit);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }
}

/// The first match of `selector`, as a JS expression.
pub fn element(selector: &str) -> String {
    format!("document.querySelector({selector:?})")
}
