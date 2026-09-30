use super::*;

fn day(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 9, day).expect("a real day")
}

#[test]
fn a_waiting_range_previews_up_to_the_hovered_day() {
    let waiting = Selection::Range(Some(DateRange::new(day(10), None)));
    assert_eq!(waiting.marks(day(12), Some(day(14))), (false, true));
    assert_eq!(waiting.marks(day(14), Some(day(14))), (true, false));
    // Backwards works too.
    assert_eq!(waiting.marks(day(8), Some(day(6))), (false, true));
    assert_eq!(waiting.marks(day(12), None), (false, false));
    assert!(waiting.awaits_end());
}

#[test]
fn a_clipped_selection_marks_its_days_as_the_whole_one_does() {
    let hover = Some(day(24));
    let selections = [
        Selection::Single(Some(day(9))),
        Selection::Range(Some(DateRange::new(day(20), Some(day(3))))),
        Selection::Range(Some(DateRange::new(day(10), None))),
    ];
    for selection in selections {
        for first in (1..=22).map(day) {
            let last = add_days(first, 6);
            let clipped = selection.clip(hover, first, last);
            for offset in 0..7 {
                let shown = add_days(first, offset);
                assert_eq!(clipped.marks(shown, None), selection.marks(shown, hover));
            }
        }
    }
    // A range moving past a week leaves the week's marks equal.
    let range = |end| Selection::Range(Some(DateRange::new(day(1), Some(day(end)))));
    assert_eq!(
        range(20).clip(None, day(8), day(14)),
        range(27).clip(None, day(8), day(14))
    );
}

#[test]
fn a_strip_across_months_names_both() {
    use crate::localization::{DateLocale, Formats};
    let strip = |start, end| view::Strip {
        start,
        end,
        days: 7,
        stop: start,
    };
    let (formats, names) = (&Formats::AMERICAN, &DateLocale::ENGLISH);
    let october = NaiveDate::from_ymd_opt(2026, 10, 4).expect("a real day");
    assert_eq!(
        strip(day(28), october).name(formats, names),
        "September 2026 – October 2026"
    );
    assert_eq!(
        strip(day(7), day(13)).name(formats, names),
        "September 2026"
    );
}

#[test]
fn a_complete_range_ignores_the_mouse() {
    let complete = Selection::Range(Some(DateRange::new(day(10), Some(day(12)))));
    assert_eq!(complete.marks(day(11), Some(day(20))), (false, true));
    assert_eq!(complete.marks(day(15), Some(day(20))), (false, false));
    assert!(!complete.awaits_end());
}
