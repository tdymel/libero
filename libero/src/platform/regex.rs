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

    /// Both engines match a suffix and so report offsets into it; callers
    /// want them in the whole span.
    fn shift(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
        for group in self.groups.iter_mut().flatten() {
            group.0 += offset;
            group.1 += offset;
        }
    }
}

/// A span prepared for repeated matching.
///
/// On wasm the text is marshalled into a JS string once here instead of on
/// every [`RegexApi::find`]: `highlight.rs` re-searches the same span once
/// per match, and a `&str` argument costs a full UTF-8 -> UTF-16 copy per
/// call, which made tokenizing quadratic in span length. The per-call
/// haystack is then a JS-side `substring`, which is a view, not a copy.
pub(crate) struct PreparedText<'a> {
    text: &'a str,
    #[cfg(target_arch = "wasm32")]
    js: js_sys::JsString,
    /// Last resolved (byte, UTF-16) offset pair. `find`'s `start` only ever
    /// moves forward within one span, so resuming the walk here keeps
    /// offset conversion linear over the span instead of quadratic.
    #[cfg(target_arch = "wasm32")]
    cursor: std::cell::Cell<(usize, usize)>,
}

impl<'a> PreparedText<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            text,
            #[cfg(target_arch = "wasm32")]
            js: js_sys::JsString::from(text),
            #[cfg(target_arch = "wasm32")]
            cursor: std::cell::Cell::new((0, 0)),
        }
    }

    /// The still-unsearched tail. Matching this rather than the whole span
    /// keeps `^`/`\b` seeing a fresh string start, as slicing always did.
    fn suffix(&self, start: usize) -> &'a str {
        &self.text[start..]
    }

    #[cfg(target_arch = "wasm32")]
    fn utf16_offset(&self, byte: usize) -> u32 {
        let (checkpoint, utf16) = self.cursor.get();
        let (from, mut utf16) = if byte < checkpoint {
            (0, 0)
        } else {
            (checkpoint, utf16)
        };

        utf16 += self.text[from..byte]
            .chars()
            .map(char::len_utf16)
            .sum::<usize>();
        self.cursor.set((byte, utf16));

        utf16 as u32
    }
}

/// Finds the leftmost match of `pattern` in `text` at or after `start`,
/// reporting offsets into the whole span. Patterns must use syntax both
/// engines understand - no lookaround or backreferences; `PatternDef`'s
/// group-index flags stand in, the same convention Prism uses.
///
/// `pattern` is `&'static str` so it can key the compile cache by identity -
/// see [`CompiledKey`].
pub(crate) trait RegexApi {
    fn find(
        &self,
        pattern: &'static str,
        case_insensitive: bool,
        text: &PreparedText<'_>,
        start: usize,
    ) -> Option<RegexMatch>;
}

struct PlatformRegexApi;

impl RegexApi for PlatformRegexApi {
    fn find(
        &self,
        pattern: &'static str,
        case_insensitive: bool,
        text: &PreparedText<'_>,
        start: usize,
    ) -> Option<RegexMatch> {
        find_impl(pattern, case_insensitive, text, start)
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

/// `exec` is reached through `Reflect` rather than `js_sys::RegExp::exec`,
/// which takes a `&str` and so would re-copy the haystack on every call -
/// the copy [`PreparedText`] exists to avoid.
#[cfg(target_arch = "wasm32")]
fn find_impl(
    pattern: &'static str,
    case_insensitive: bool,
    text: &PreparedText<'_>,
    start: usize,
) -> Option<RegexMatch> {
    use wasm_bindgen::JsCast;

    let regexp = cached_regexp(pattern, case_insensitive);
    let haystack = text
        .js
        .substring(text.utf16_offset(start), text.js.length());

    let exec: js_sys::Function = js_sys::Reflect::get(&regexp, &"exec".into())
        .ok()?
        .dyn_into()
        .ok()?;
    let result = exec.call1(&regexp, &haystack).ok()?;
    if result.is_null() || result.is_undefined() {
        return None;
    }

    let indices = js_sys::Reflect::get(&result, &"indices".into()).ok()?;
    let indices: js_sys::Array = indices.dyn_into().ok()?;
    let spans: Vec<Option<(usize, usize)>> = (0..indices.length())
        .map(|group| group_span(&indices, group))
        .collect();

    let mut matched = resolve_spans(text.suffix(start), &spans)?;
    matched.shift(start);

    Some(matched)
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
fn find_impl(
    pattern: &'static str,
    case_insensitive: bool,
    text: &PreparedText<'_>,
    start: usize,
) -> Option<RegexMatch> {
    REGEX_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let regex = cache
            .entry(CompiledKey::new(pattern, case_insensitive))
            .or_insert_with(|| {
                let source = ascii_semantics(pattern);
                regex::RegexBuilder::new(&source)
                    .case_insensitive(case_insensitive)
                    .build()
                    .unwrap_or_else(|err| panic!("invalid regex pattern {pattern:?}: {err}"))
            });

        let captures = regex.captures(text.suffix(start))?;
        let whole = captures.get(0).expect("group 0 always matches");

        let groups = (1..captures.len())
            .map(|group| captures.get(group).map(|m| (m.start(), m.end())))
            .collect();

        let mut matched = RegexMatch {
            start: whole.start(),
            end: whole.end(),
            groups,
        };
        matched.shift(start);

        Some(matched)
    })
}

/// Rewrites the shorthands the `regex` crate reads as Unicode into the ASCII
/// ones JS's `RegExp` has, so both arms tokenize the same source the same way.
///
/// The web arm is the one that cannot move. JS has no flag that makes `\b`,
/// `\w` or `\d` Unicode-aware - `u` changes `.`, escapes and case folding,
/// not those - and the grammars are ports of Prism's, which were written
/// against exactly these ASCII semantics. So `regex` comes to JS, not the
/// other way round.
///
/// Without this, `Code { language: "rust", source: "éif x" }` highlights the
/// `if` inside `éif` as a keyword on the web and not natively: `é` is a word
/// character to `regex` and not to `RegExp`, so only one of them sees a
/// boundary before the `i`. Under `fullstack` that is a server render the
/// client then contradicts.
///
/// `\s` and `\S` are deliberately left alone: JS's `\s` is already Unicode
/// whitespace without `u`, so the two only disagree on `\u{feff}`, and a
/// negated ASCII class cannot be written inside a character class anyway.
///
/// Rewriting happens once per pattern, inside the compile cache's miss arm.
#[cfg(not(target_arch = "wasm32"))]
fn ascii_semantics(pattern: &str) -> std::borrow::Cow<'_, str> {
    if !pattern.contains('\\') {
        return std::borrow::Cow::Borrowed(pattern);
    }

    let mut out = String::with_capacity(pattern.len());
    // A character class needs the bare ranges, not a nested class: `[\da-f]`
    // becomes `[0-9a-f]`. Only the outermost `[`/`]` toggle it, since a `[`
    // inside a class is a literal to `RegExp` - the dialect these are written
    // in.
    let mut in_class = false;
    let mut chars = pattern.chars();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.next() {
                Some('b') if !in_class => out.push_str("(?-u:\\b)"),
                Some('B') if !in_class => out.push_str("(?-u:\\B)"),
                Some('w') => out.push_str(match in_class {
                    true => "0-9A-Za-z_",
                    false => "[0-9A-Za-z_]",
                }),
                Some('d') => out.push_str(match in_class {
                    true => "0-9",
                    false => "[0-9]",
                }),
                // Negated: expressible outside a class only. Both engines
                // then agree, since `[^...]` is Unicode-aware in each.
                Some('W') if !in_class => out.push_str("[^0-9A-Za-z_]"),
                Some('D') if !in_class => out.push_str("[^0-9]"),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            },
            '[' if !in_class => {
                in_class = true;
                out.push('[');
            }
            ']' if in_class => {
                in_class = false;
                out.push(']');
            }
            _ => out.push(ch),
        }
    }

    std::borrow::Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::{PreparedText, ascii_semantics, regex_api, resolve_spans, utf16_to_byte_offsets};

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

    /// The web arm has no such rewrite: `RegExp` already reads these as
    /// ASCII, which is what the rewrite is making `regex` agree with.
    #[test]
    fn the_unicode_shorthands_are_rewritten_to_their_ascii_sets() {
        assert_eq!(
            ascii_semantics(r"\b(?:true|false)\b"),
            r"(?-u:\b)(?:true|false)(?-u:\b)"
        );
        assert_eq!(ascii_semantics(r"\w+"), "[0-9A-Za-z_]+");
        assert_eq!(ascii_semantics(r"\d+"), "[0-9]+");
        assert_eq!(ascii_semantics(r"\W\D"), "[^0-9A-Za-z_][^0-9]");
    }

    /// `[\da-fA-F]` has to become `[0-9a-fA-F]`, not `[[0-9]a-fA-F]`.
    #[test]
    fn a_shorthand_inside_a_class_becomes_bare_ranges() {
        assert_eq!(ascii_semantics(r"0x[\da-fA-F]+"), "0x[0-9a-fA-F]+");
        assert_eq!(ascii_semantics(r"[-\w]+"), "[-0-9A-Za-z_]+");
        assert_eq!(ascii_semantics(r"[^\d]"), "[^0-9]");
    }

    /// An escaped backslash is not an escape sequence, and `\s` is left as it
    /// is because `RegExp` reads that one as Unicode too.
    #[test]
    fn nothing_else_is_touched() {
        assert_eq!(ascii_semantics(r"\\w"), r"\\w");
        assert_eq!(ascii_semantics(r"[\s\S]*?"), r"[\s\S]*?");
        assert_eq!(
            ascii_semantics(r#""(?:[^"\\]|\\.)*""#),
            r#""(?:[^"\\]|\\.)*""#
        );
    }

    /// Review 6 S1, todo 279. `é` is a word character to `regex` and not to
    /// `RegExp`, so before the rewrite only the web saw a boundary before the
    /// `i` and marked the `if` inside `éif` as a keyword. Probed in Node:
    /// `/\b(?:as|fn|let|if)\b/d` matches `"éif x"` at index 1.
    #[test]
    fn a_word_boundary_after_a_non_ascii_letter_matches_as_it_does_on_the_web() {
        let text = PreparedText::new("éif x; if y");

        let matched = regex_api()
            .find(r"\b(?:as|fn|let|if)\b", false, &text, 0)
            .expect("the keyword inside `éif` matches, as it does on the web");

        // `é` is two bytes, so the `if` inside `éif` starts at byte 2.
        assert_eq!((matched.start, matched.end), (2, 4));
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
