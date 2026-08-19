/// A single match, as byte offsets into the UTF-8 text that was searched -
/// callers never need to think about the underlying engine's own string
/// representation (native `regex` uses UTF-8 byte offsets already; the
/// browser engine's UTF-16 code-unit offsets are converted before this type
/// is ever built).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegexMatch {
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// 1-indexed capture group spans - `groups[0]` is group 1, etc. Used to
    /// emulate lookbehind/lookahead (see `PatternDef` in `highlight.rs`):
    /// neither regex engine needs real lookaround support because of it.
    groups: Vec<Option<(usize, usize)>>,
}

impl RegexMatch {
    pub(crate) fn group(&self, index: usize) -> Option<(usize, usize)> {
        debug_assert!(index >= 1, "group 0 (the whole match) has no index here");
        self.groups.get(index - 1).copied().flatten()
    }
}

/// Finds the leftmost match of `pattern` in `text`. Patterns are restricted
/// to syntax both implementations understand - no lookaround assertions or
/// backreferences (native `regex` supports neither); lookbehind/lookahead-
/// style behavior goes through `PatternDef`'s group-index flags instead,
/// same convention Prism itself uses for lookbehind.
///
/// `pattern` is `&'static str` because every compiled pattern is cached
/// under its identity for the process' lifetime - see [`CompiledKey`].
pub(crate) trait RegexApi {
    fn find(&self, pattern: &'static str, case_insensitive: bool, text: &str)
    -> Option<RegexMatch>;
}

struct PlatformRegexApi;

impl RegexApi for PlatformRegexApi {
    fn find(
        &self,
        pattern: &'static str,
        case_insensitive: bool,
        text: &str,
    ) -> Option<RegexMatch> {
        find_impl(pattern, case_insensitive, text)
    }
}

/// Identifies a compiled regex without hashing the pattern body: every
/// pattern reaching here is a `&'static str` from a `const` grammar in
/// `languages/`, so its address and length already identify it uniquely.
/// The flag is part of the key because case-insensitivity is baked into the
/// compiled form on both engines.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct CompiledKey {
    pattern: *const u8,
    len: usize,
    case_insensitive: bool,
}

impl CompiledKey {
    fn new(pattern: &'static str, case_insensitive: bool) -> Self {
        Self {
            pattern: pattern.as_ptr(),
            len: pattern.len(),
            case_insensitive,
        }
    }
}

static PLATFORM_REGEX_API: PlatformRegexApi = PlatformRegexApi;

/// The current platform's [`RegexApi`] - a plain accessor, not a hook, so
/// it's callable from anywhere.
pub(crate) fn regex_api() -> &'static dyn RegexApi {
    &PLATFORM_REGEX_API
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// Compiled `RegExp` objects, keyed by pattern identity. Safe to reuse
    /// across calls: no `g`/`y` flag, so `exec` carries no `lastIndex` state
    /// from one call to the next. Single-threaded target, hence `thread_local`
    /// rather than a lock.
    static REGEXP_CACHE: std::cell::RefCell<
        std::collections::HashMap<CompiledKey, js_sys::RegExp>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
}

#[cfg(target_arch = "wasm32")]
fn cached_regexp(pattern: &'static str, case_insensitive: bool) -> js_sys::RegExp {
    REGEXP_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(CompiledKey::new(pattern, case_insensitive))
            .or_insert_with(|| {
                // "d" (hasIndices) surfaces per-group code-unit spans via
                // `.indices`, needed to locate lookbehind/lookahead
                // reference groups.
                let flags = if case_insensitive { "di" } else { "d" };
                js_sys::RegExp::new(pattern, flags)
            })
            .clone()
    })
}

#[cfg(target_arch = "wasm32")]
fn find_impl(pattern: &'static str, case_insensitive: bool, text: &str) -> Option<RegexMatch> {
    use wasm_bindgen::JsCast;

    let regexp = cached_regexp(pattern, case_insensitive);
    let result = regexp.exec(text)?;

    let indices = js_sys::Reflect::get(&result, &"indices".into()).ok()?;
    let indices: js_sys::Array = indices.dyn_into().ok()?;
    let (start_u16, end_u16) = group_span(&indices, 0)?;
    let start = utf16_to_byte_offset(text, start_u16);
    let end = utf16_to_byte_offset(text, end_u16);

    let group_count = indices.length();
    let groups = (1..group_count)
        .map(|group| {
            group_span(&indices, group).map(|(g_start, g_end)| {
                (
                    utf16_to_byte_offset(text, g_start),
                    utf16_to_byte_offset(text, g_end),
                )
            })
        })
        .collect();

    Some(RegexMatch { start, end, groups })
}

#[cfg(target_arch = "wasm32")]
fn group_span(indices: &js_sys::Array, group: u32) -> Option<(usize, usize)> {
    use wasm_bindgen::JsCast;

    let span = indices.get(group);
    if span.is_undefined() || span.is_null() {
        return None;
    }
    let span: js_sys::Array = span.dyn_into().ok()?;
    let start = span.get(0).as_f64()? as usize;
    let end = span.get(1).as_f64()? as usize;
    Some((start, end))
}

/// JS string indices are UTF-16 code-unit offsets - walk `text`'s chars,
/// accumulating both encodings in lockstep, until the UTF-16 count reaches
/// `offset`.
#[cfg(target_arch = "wasm32")]
fn utf16_to_byte_offset(text: &str, offset: usize) -> usize {
    let mut utf16_count = 0usize;
    for (byte_offset, ch) in text.char_indices() {
        if utf16_count >= offset {
            return byte_offset;
        }
        utf16_count += ch.len_utf16();
    }
    text.len()
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    /// Compiled regexes, keyed by pattern identity. Compilation dominates
    /// matching by orders of magnitude, and `highlight.rs` re-runs the same
    /// handful of patterns across every untokenized span of a code block, so
    /// this is what keeps highlighting off an O(patterns x spans) compile path.
    ///
    /// Unbounded on purpose: the pattern set is closed (the `const` grammars of
    /// whichever `code-lang-*` features are compiled in), so the map reaches a
    /// fixed size and stops growing.
    static REGEX_CACHE: std::cell::RefCell<
        std::collections::HashMap<CompiledKey, regex::Regex>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
}

#[cfg(not(target_arch = "wasm32"))]
fn find_impl(pattern: &'static str, case_insensitive: bool, text: &str) -> Option<RegexMatch> {
    REGEX_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let regex = cache
            .entry(CompiledKey::new(pattern, case_insensitive))
            .or_insert_with(|| {
                regex::RegexBuilder::new(pattern)
                    .case_insensitive(case_insensitive)
                    .build()
                    .unwrap_or_else(|err| panic!("invalid regex pattern {pattern:?}: {err}"))
            });

        let captures = regex.captures(text)?;
        let whole = captures.get(0).expect("group 0 always matches");

        let groups = (1..captures.len())
            .map(|group| captures.get(group).map(|m| (m.start(), m.end())))
            .collect();

        Some(RegexMatch {
            start: whole.start(),
            end: whole.end(),
            groups,
        })
    })
}
