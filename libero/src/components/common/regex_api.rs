/// A single match, always as UTF-8 byte offsets - the browser engine's UTF-16
/// code-unit offsets are converted before this type is built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegexMatch {
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// 1-indexed group spans (`groups[0]` is group 1). Used to emulate
    /// lookaround, which neither engine supports - see `highlight.rs`.
    groups: Vec<Option<(usize, usize)>>,
}

impl RegexMatch {
    pub(crate) fn group(&self, index: usize) -> Option<(usize, usize)> {
        debug_assert!(index >= 1, "group 0 (the whole match) has no index here");
        self.groups.get(index - 1).copied().flatten()
    }
}

/// Finds the leftmost match of `pattern` in `text`. Patterns must use syntax
/// both engines understand - no lookaround or backreferences; `PatternDef`'s
/// group-index flags stand in, the same convention Prism uses.
///
/// `pattern` is `&'static str` so it can key the compile cache by identity -
/// see [`CompiledKey`].
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

/// Identifies a compiled regex without hashing its body: every pattern comes
/// from a `const` grammar, so address and length identify it. The flag is part
/// of the key because both engines bake it into the compiled form.
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

/// Not a hook, so it's callable from anywhere.
pub(crate) fn regex_api() -> &'static dyn RegexApi {
    &PLATFORM_REGEX_API
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// Reuse is safe: no `g`/`y` flag, so `exec` carries no `lastIndex`
    /// between calls. `thread_local` because the target is single-threaded.
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
                // "d" surfaces per-group spans via `.indices`, which is how
                // lookaround reference groups get located.
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
    let spans: Vec<Option<(usize, usize)>> = (0..indices.length())
        .map(|group| group_span(&indices, group))
        .collect();

    resolve_spans(text, &spans)
}

/// `spans` are the engine's UTF-16 spans, group 0 first. One walk of `text`
/// converts them all - doing it per offset re-walked the string 2 + 2*groups
/// times per match, which made highlighting quadratic in source length.
#[cfg(any(target_arch = "wasm32", test))]
fn resolve_spans(text: &str, spans: &[Option<(usize, usize)>]) -> Option<RegexMatch> {
    let offsets: Vec<usize> = spans
        .iter()
        .flatten()
        .flat_map(|&(start, end)| [start, end])
        .collect();
    let mut bytes = utf16_to_byte_offsets(text, &offsets).into_iter();

    let mut resolved = spans
        .iter()
        .map(|span| span.and_then(|_| Some((bytes.next()?, bytes.next()?))));
    let (start, end) = resolved.next().flatten()?;

    Some(RegexMatch {
        start,
        end,
        groups: resolved.collect(),
    })
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

/// JS offsets are UTF-16 code units; walk both encodings in lockstep,
/// resolving every offset in `offsets` in one pass. Returns them in the
/// order given.
#[cfg(any(target_arch = "wasm32", test))]
fn utf16_to_byte_offsets(text: &str, offsets: &[usize]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..offsets.len()).collect();
    order.sort_unstable_by_key(|&index| offsets[index]);

    let mut resolved = vec![text.len(); offsets.len()];
    let mut chars = text.chars();
    let mut utf16 = 0usize;
    let mut byte = 0usize;

    for index in order {
        while utf16 < offsets[index] {
            let Some(ch) = chars.next() else {
                byte = text.len();
                break;
            };
            utf16 += ch.len_utf16();
            byte += ch.len_utf8();
        }
        resolved[index] = byte;
    }

    resolved
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    /// Compilation dominates matching by orders of magnitude, and
    /// `highlight.rs` re-runs the same patterns over every untokenized span -
    /// without this, highlighting is O(patterns x spans) compiles.
    ///
    /// Unbounded on purpose: the pattern set is closed (`const` grammars), so
    /// the map reaches a fixed size.
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

#[cfg(test)]
mod tests {
    use super::{resolve_spans, utf16_to_byte_offsets};

    /// 'é' is 2 bytes / 1 code unit, '😀' 4 bytes / 2 code units.
    const MIXED: &str = "aé😀b";

    #[test]
    fn offsets_resolve_across_encodings() {
        assert_eq!(
            utf16_to_byte_offsets(MIXED, &[0, 1, 2, 4, 5]),
            vec![0, 1, 3, 7, 8]
        );
    }

    /// Group spans arrive in group order, which nesting makes non-monotonic.
    #[test]
    fn offsets_resolve_out_of_order() {
        assert_eq!(utf16_to_byte_offsets(MIXED, &[4, 1, 2]), vec![7, 1, 3]);
    }

    /// Landing inside a surrogate pair rounds past it, as walking one offset
    /// at a time did.
    #[test]
    fn an_offset_inside_a_surrogate_pair_lands_after_it() {
        assert_eq!(utf16_to_byte_offsets(MIXED, &[3]), vec![7]);
    }

    #[test]
    fn an_offset_past_the_end_is_the_length() {
        assert_eq!(utf16_to_byte_offsets(MIXED, &[99]), vec![MIXED.len()]);
    }

    #[test]
    fn spans_resolve_to_a_match_with_its_groups() {
        let spans = [Some((1, 4)), None, Some((2, 4))];

        let matched = resolve_spans(MIXED, &spans).expect("group 0 matched");

        assert_eq!((matched.start, matched.end), (1, 7));
        assert_eq!(matched.group(1), None);
        assert_eq!(matched.group(2), Some((3, 7)));
    }
}
