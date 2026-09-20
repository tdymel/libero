/// The next enabled item in `step`'s direction, wrapping; `None` if there is none.
/// Works from a disabled `current` too.
pub(crate) fn neighbour(disabled: &[bool], current: usize, step: isize) -> Option<usize> {
    let count = disabled.len() as isize;
    if count == 0 {
        return None;
    }
    let mut index = current as isize;
    for _ in 0..count {
        index = ((index + step) % count + count) % count;
        if !disabled[index as usize] {
            return Some(index as usize);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::neighbour;

    #[test]
    fn stepping_wraps_at_both_ends() {
        let all_on = [false, false, false];
        assert_eq!(neighbour(&all_on, 0, 1), Some(1));
        assert_eq!(neighbour(&all_on, 2, 1), Some(0));
        assert_eq!(neighbour(&all_on, 0, -1), Some(2));
        assert_eq!(neighbour(&all_on, 1, -1), Some(0));
    }

    /// A disabled item renders but cannot be picked, so the arrows step
    /// over it rather than landing on it.
    #[test]
    fn stepping_skips_the_disabled_items() {
        let middle_off = [false, true, false];
        assert_eq!(neighbour(&middle_off, 0, 1), Some(2));
        assert_eq!(neighbour(&middle_off, 2, -1), Some(0));

        let only_last = [true, true, false];
        assert_eq!(neighbour(&only_last, 2, 1), Some(2));
    }

    /// Todo 403: from a selected disabled item the two arrows part ways.
    #[test]
    fn stepping_from_a_disabled_item_goes_both_ways() {
        let middle_off = [false, true, false];
        assert_eq!(neighbour(&middle_off, 1, -1), Some(0));
        assert_eq!(neighbour(&middle_off, 1, 1), Some(2));
    }

    #[test]
    fn stepping_nowhere_selects_nothing() {
        assert_eq!(neighbour(&[], 0, 1), None);
        assert_eq!(neighbour(&[true, true], 0, 1), None);
    }
}
