//! `e2e/skips.md` lists every `scenario!` skip with its platform and reason, the one place a
//! revisit greps (todo 3362); this test rewrites it when a skip changed and fails.

use std::path::{Path, PathBuf};

/// One skip: the test path, the platform the token names, the reason.
#[derive(Debug, PartialEq)]
struct Skip {
    test: String,
    platform: &'static str,
    reason: String,
}

/// The platform a skip token names; `None` for an unknown token.
fn platform(token: &str) -> Option<&'static str> {
    Some(match token {
        "web" => "web",
        "native" => "native",
        "android" => "android",
        "desktop" => "desktop",
        "ios" => "ios",
        "macos" => "macos desktop",
        "windows" => "windows desktop",
        "macos_web" => "macos web",
        "macos_native" => "macos native",
        "windows_web" => "windows web",
        "windows_native" => "windows native",
        "android_only" => "all but android",
        "ios_only" => "all but ios",
        _ => return None,
    })
}

/// The value of the string literal `literal` starts with, and the rest after it.
fn string(literal: &str) -> Option<(String, &str)> {
    let mut chars = literal.strip_prefix('"')?.char_indices();
    let mut value = String::new();
    while let Some((at, c)) = chars.next() {
        match c {
            '"' => return Some((value, &literal[at + 2..])),
            '\\' => match chars.next()?.1 {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                // A line continuation drops the newline and the next line's indent.
                '\n' => {
                    while chars.clone().next().is_some_and(|(_, c)| c.is_whitespace()) {
                        chars.next();
                    }
                }
                escaped => value.push(escaped),
            },
            c => value.push(c),
        }
    }
    None
}

/// The length of the source item at the start of `rest` the scan steps over whole: a string,
/// a char literal or a comment, so their braces and parens do not count.
fn opaque(rest: &str) -> Option<usize> {
    if rest.starts_with('"') {
        let (_, after) = string(rest)?;
        return Some(rest.len() - after.len());
    }
    if rest.starts_with("//") {
        return Some(rest.find('\n').unwrap_or(rest.len()));
    }
    if rest.starts_with("/*") {
        return Some(rest.find("*/")? + 2);
    }
    // `'{'` or `'\''`, not a lifetime.
    let bytes = rest.as_bytes();
    if bytes[0] == b'\'' {
        if bytes.get(1) == Some(&b'\\') {
            return Some(rest[2..].find('\'')? + 3);
        }
        if bytes.get(2) == Some(&b'\'') {
            return Some(3);
        }
    }
    None
}

/// The scenario's arguments split at top-level commas, from just after `scenario!(` to its `)`.
fn arguments(source: &str) -> Vec<&str> {
    let (mut depth, mut start, mut at) = (0usize, 0, 0);
    let mut parts = Vec::new();
    while at < source.len() {
        if let Some(len) = opaque(&source[at..]) {
            at += len;
            continue;
        }
        let c = source[at..].chars().next().unwrap_or(' ');
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => break,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(source[start..at].trim());
                start = at + 1;
            }
            _ => {}
        }
        at += c.len_utf8();
    }
    parts.push(source[start..at].trim());
    parts.retain(|part| !part.is_empty());
    parts
}

/// The skips of every `scenario!` in `source`, the test file of module path `module`, and
/// how many scenarios it declares.
fn skips_in(source: &str, module: &str) -> (Vec<Skip>, usize) {
    let (mut skips, mut scenarios) = (Vec::new(), 0);
    // Inline `mod name {` blocks, each with the brace depth it closes at.
    let mut modules: Vec<(String, usize)> = Vec::new();
    let (mut depth, mut at) = (0usize, 0);
    while at < source.len() {
        let rest = &source[at..];
        if let Some(len) = opaque(rest) {
            at += len;
            continue;
        }
        let word_start = at == 0 || !source.as_bytes()[at - 1].is_ascii_alphanumeric();
        if word_start && rest.starts_with("mod ") {
            let name: String = rest[4..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if rest[4 + name.len()..].trim_start().starts_with('{') {
                modules.push((name, depth + 1));
            }
        }
        if word_start && rest.starts_with("scenario!(") {
            scenarios += 1;
            let args = arguments(&rest["scenario!(".len()..]);
            let path: Vec<&str> = std::iter::once(module)
                .chain(modules.iter().map(|(name, _)| name.as_str()))
                .chain(args.first().copied())
                .collect();
            for arg in args.iter().skip(3) {
                let token_len = arg
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(arg.len());
                let (token, call) = arg.split_at(token_len);
                let call = call.trim_start();
                // `macos: skip("..")` or `ios_only("..")`.
                let literal = match call.strip_prefix(':') {
                    Some(call) => call.trim_start().strip_prefix("skip("),
                    None => call.strip_prefix('('),
                };
                let platform = platform(token)
                    .unwrap_or_else(|| panic!("{}: unknown skip token `{token}`", path.join("::")));
                let (reason, _) = literal
                    .and_then(|literal| string(literal.trim_start()))
                    .unwrap_or_else(|| panic!("{}: unreadable skip `{arg}`", path.join("::")));
                skips.push(Skip {
                    test: path.join("::"),
                    platform,
                    reason,
                });
            }
        }
        // A plain web test's host skip: `#[cfg_attr(windows, ignore = "..")]`.
        if let Some(attr) = rest.strip_prefix("#[cfg_attr(") {
            let args = arguments(attr);
            let platform = match args.first().copied() {
                Some("windows") => Some("windows web"),
                Some("target_os = \"macos\"") => Some("macos web"),
                _ => None,
            };
            let reason = args.get(1).and_then(|ignore| {
                let literal = ignore
                    .strip_prefix("ignore")?
                    .trim_start()
                    .strip_prefix('=')?;
                string(literal.trim_start())
            });
            if let (Some(platform), Some((reason, _))) = (platform, reason) {
                let after = &rest[rest.find("fn ").map_or(rest.len(), |at| at + 3)..];
                let name: String = after
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                let path: Vec<&str> = std::iter::once(module)
                    .chain(modules.iter().map(|(name, _)| name.as_str()))
                    .chain([name.as_str()])
                    .collect();
                skips.push(Skip {
                    test: path.join("::"),
                    platform,
                    reason,
                });
            }
        }
        match rest.as_bytes()[0] {
            b'{' => depth += 1,
            b'}' => {
                if modules.last().is_some_and(|(_, closes)| *closes == depth) {
                    modules.pop();
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
        at += rest.chars().next().map_or(1, char::len_utf8);
    }
    (skips, scenarios)
}

/// The `.rs` files under `dir`, sorted.
fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(sources(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// The module path of a test file under `root`: `video/mod.rs` is `video`.
fn module_of(root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(root).unwrap().with_extension("");
    let mut parts: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.last().is_some_and(|last| last == "mod") {
        parts.pop();
    }
    parts.join("::")
}

fn render(skips: &[Skip], scenarios: usize) -> String {
    let mut out = format!("# Skipped e2e scenarios\n\nScenarios: {scenarios}\n\n");
    out.push_str(
        "\
         Generated from the `scenario!` skips in `e2e/tests/all` by `cargo test -p e2e --lib skips`; \
         edit the skip, not this file. A reason starting `upstream:` waits on Dioxus, Blitz, wry or \
         the engine (revisit on their update, todo 3362), a number names the todo that removes it.\n\n\
         The performance scenarios (units `perf`, `table_perf`) are no skips but run on request only: \
         the runner leaves them out on CI and `shard.py` assigns them to no shard, unless `E2E_PERF` \
         is set (todo 3454).\n\n\
         | Test | Platform | Reason |\n| --- | --- | --- |\n",
    );
    for skip in skips {
        let reason = skip
            .reason
            .replace('|', "\\|")
            .replace('<', "\\<")
            .replace('\n', " ");
        out.push_str(&format!(
            "| `{}` | {} | {reason} |\n",
            skip.test, skip.platform
        ));
    }
    out
}

#[test]
fn skips_md_lists_every_scenario_skip() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join("tests/all");
    let (mut skips, mut scenarios) = (Vec::new(), 0);
    for file in sources(&root) {
        let source = std::fs::read_to_string(&file).unwrap();
        let (found, count) = skips_in(&source, &module_of(&root, &file));
        skips.extend(found);
        scenarios += count;
    }
    skips.sort_by(|a, b| (&a.test, a.platform).cmp(&(&b.test, b.platform)));
    let expected = render(&skips, scenarios);
    let file = manifest.join("skips.md");
    if std::fs::read_to_string(&file).ok().as_deref() != Some(&expected) {
        std::fs::write(&file, &expected).unwrap();
        panic!("e2e/skips.md was stale: rewritten, commit it");
    }
}

#[test]
fn a_skip_is_read_with_its_module_and_reason() {
    let source = r#"
        mod inner {
            e2e::scenario!(
                it_opens,
                "/x",
                body,
                macos: skip("upstream: a \"quoted\" \
                    reason"),
                ios_only("only there")
            );
        }
        // scenario!(commented, "/", body, web: skip("no"));
        e2e::scenario!(it_closes, "/y", body);
        #[test]
        #[cfg_attr(windows, ignore = "3423: fonts")]
        fn it_meets() {}
    "#;
    let (skips, scenarios) = skips_in(source, "unit");
    assert_eq!(scenarios, 2);
    assert_eq!(
        skips,
        vec![
            Skip {
                test: "unit::inner::it_opens".into(),
                platform: "macos desktop",
                reason: "upstream: a \"quoted\" reason".into(),
            },
            Skip {
                test: "unit::inner::it_opens".into(),
                platform: "all but ios",
                reason: "only there".into(),
            },
            Skip {
                test: "unit::it_meets".into(),
                platform: "windows web",
                reason: "3423: fonts".into(),
            },
        ]
    );
}
