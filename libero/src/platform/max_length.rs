//! `maxlength` over a paste, which Blitz's editor ignores (todo 950). The web
//! applies it itself, so there every call returns the text unchanged.

/// The text a control's `input` should carry: natively, a paste cut to the
/// control's `maxlength`. Call it before emitting, so a caller never sees more.
pub(crate) fn fit_max_length(value: String) -> String {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return super::backend::fit_pasted(value);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    value
}

/// `new` with what was inserted into `old` cut to fit `limit` UTF-16 units, as
/// the web cuts a paste: the text around the insertion stays whole.
#[cfg_attr(
    not(all(not(target_arch = "wasm32"), feature = "native")),
    allow(dead_code)
)]
pub(crate) fn fit_insertion(old: &str, new: &str, limit: usize) -> String {
    let units = |text: &str| text.encode_utf16().count();
    if units(new) <= limit {
        return new.to_string();
    }
    let prefix = old
        .char_indices()
        .zip(new.chars())
        .find(|((_, a), b)| a != b)
        .map_or(old.len().min(new.len()), |((at, _), _)| at);
    let (old_rest, new_rest) = (&old[prefix..], &new[prefix..]);
    let suffix = old_rest
        .chars()
        .rev()
        .zip(new_rest.chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum::<usize>()
        .min(new_rest.len());
    let inserted = &new_rest[..new_rest.len() - suffix];
    let kept = units(&new[..prefix]) + units(&new_rest[new_rest.len() - suffix..]);
    let mut room = limit.saturating_sub(kept);
    let cut: String = inserted
        .chars()
        .take_while(|c| {
            let fits = c.len_utf16() <= room;
            if fits {
                room -= c.len_utf16();
            }
            fits
        })
        .collect();
    format!(
        "{}{cut}{}",
        &new[..prefix],
        &new_rest[new_rest.len() - suffix..]
    )
}

#[cfg(test)]
mod tests {
    use super::fit_insertion;

    #[test]
    fn a_paste_is_cut_where_it_was_inserted() {
        assert_eq!(fit_insertion("ab", "ab12345", 4), "ab12");
        assert_eq!(fit_insertion("ab", "a12345b", 4), "a12b");
        assert_eq!(fit_insertion("", "12345", 3), "123");
    }

    #[test]
    fn a_paste_over_a_selection_gets_the_selections_room() {
        assert_eq!(fit_insertion("abcd", "a123456d", 5), "a123d");
    }

    #[test]
    fn text_that_fits_and_wide_characters_are_left_whole() {
        assert_eq!(fit_insertion("ab", "abc", 4), "abc");
        // An emoji is two units: it never lands half.
        assert_eq!(fit_insertion("ab", "ab😀😀", 5), "ab😀");
    }

    #[test]
    fn a_repeated_character_is_not_double_counted() {
        assert_eq!(fit_insertion("aa", "aaaaaa", 3), "aaa");
    }
}
