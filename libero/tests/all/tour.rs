//! `use_tour`'s markup and step arithmetic: nothing until started, a named modal card
//! per step, Back/Next/Skip by position, controlled `current`, the card callback and
//! German words. Placement, focus and keys need a browser (todo 2210, dev 2).

use std::cell::{Cell, RefCell};

use crate::common::body;
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{TourOptions, TourStep, TourView, use_tour},
    localization::Localization,
    theme::Direction,
};

#[derive(Clone, Copy)]
enum Move {
    Start,
    Next,
    Prev,
    GoTo(usize),
    Close,
    /// Takes every step away.
    Empty,
}

thread_local! {
    static MOVES: RefCell<Vec<Move>> = const { RefCell::new(Vec::new()) };
    static CURRENT: Cell<Option<usize>> = const { Cell::new(None) };
    static UNTITLED: Cell<bool> = const { Cell::new(false) };
    static CUSTOM: Cell<bool> = const { Cell::new(false) };
    static GERMAN: Cell<bool> = const { Cell::new(false) };
    static CHANGES: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    static FINISHED: Cell<u32> = const { Cell::new(0) };
    static CLOSED: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    /// Moves made after the first render, once the layer is up.
    static LATER: RefCell<Vec<Move>> = const { RefCell::new(Vec::new()) };
    static KEYBOARD_OFF: Cell<bool> = const { Cell::new(false) };
    static RTL: Cell<bool> = const { Cell::new(false) };
    static OPEN: Cell<bool> = const { Cell::new(false) };
}

fn steps() -> Vec<TourStep> {
    let first = match UNTITLED.get() {
        true => TourStep::new("welcome").description("A quick look around."),
        false => TourStep::new("welcome")
            .title("Welcome")
            .description("A quick look around."),
    };
    vec![
        first,
        TourStep::new("search")
            .title("Search")
            .description("Find anything."),
        TourStep::new("save")
            .title("Save")
            .content(rsx! { strong { "Saves" } " your work." }),
    ]
}

fn custom_card(view: TourView) -> Element {
    rsx! {
        p { id: "custom", "Step {view.index} of {view.total}: {view.step.key}" }
    }
}

#[component]
fn Tour() -> Element {
    let mut empty = use_signal(|| false);
    let tour = use_tour(TourOptions {
        steps: if empty() { Vec::new() } else { steps() },
        keyboard: !KEYBOARD_OFF.get(),
        current: CURRENT.get(),
        onchange: Some(Callback::new(|index| {
            CHANGES.with(|c| c.borrow_mut().push(index))
        })),
        onfinish: Some(Callback::new(|()| FINISHED.set(FINISHED.get() + 1))),
        onclose: Some(Callback::new(|index| {
            CLOSED.with(|c| c.borrow_mut().push(index))
        })),
        card: CUSTOM.get().then(|| Callback::new(custom_card)),
        ..Default::default()
    });
    let mut make = move |step: Move| match step {
        Move::Start => tour.start(),
        Move::Next => tour.next(),
        Move::Prev => tour.prev(),
        Move::GoTo(index) => tour.go_to(index),
        Move::Close => tour.close(),
        Move::Empty => empty.set(true),
    };
    use_hook(|| {
        MOVES
            .with(|moves| moves.borrow().clone())
            .into_iter()
            .for_each(&mut make)
    });
    use_effect(move || {
        LATER
            .with(|later| later.take())
            .into_iter()
            .for_each(&mut make)
    });
    OPEN.set(tour.is_open());
    rsx! {}
}

fn app() -> Element {
    let localization = match GERMAN.get() {
        true => &Localization::GERMAN,
        false => &Localization::ENGLISH,
    };
    let direction = match RTL.get() {
        true => Direction::Rtl,
        false => Direction::Ltr,
    };
    rsx! {
        LiberoProvider { localization, direction, Tour {} }
    }
}

fn reset() {
    MOVES.with(|moves| moves.borrow_mut().clear());
    CURRENT.set(None);
    UNTITLED.set(false);
    CUSTOM.set(false);
    GERMAN.set(false);
    CHANGES.with(|c| c.borrow_mut().clear());
    FINISHED.set(0);
    CLOSED.with(|c| c.borrow_mut().clear());
    LATER.with(|later| later.borrow_mut().clear());
    KEYBOARD_OFF.set(false);
    RTL.set(false);
    OPEN.set(false);
}

fn rendered(moves: &[Move]) -> String {
    MOVES.with(|m| *m.borrow_mut() = moves.to_vec());
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    // The start lands after the slot was published empty.
    for _ in 0..4 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    body(&dioxus_ssr::render(&dom))
}

fn tags_with<'a>(html: &'a str, needle: &str) -> Vec<&'a str> {
    html.match_indices('<')
        .map(|(at, _)| &html[at..at + html[at..].find('>').unwrap()])
        .filter(|tag| tag.contains(needle))
        .collect()
}

fn slot(html: &str, name: &str) -> Option<String> {
    tags_with(html, &format!(r#"data-slot="{name}""#))
        .first()
        .map(|tag| tag.to_string())
}

/// The first text after the tag carrying `data-slot="name"`, past any nested tags.
fn slot_text(html: &str, name: &str) -> String {
    let at = html.find(&format!(r#"data-slot="{name}""#)).expect(name);
    let from = at + html[at..].find('>').unwrap() + 1;
    html[from..]
        .split('<')
        .map(|chunk| chunk.rsplit('>').next().unwrap_or(""))
        .find(|chunk| !chunk.trim().is_empty())
        .unwrap_or("")
        .to_string()
}

#[test]
fn nothing_renders_until_started() {
    reset();
    let html = rendered(&[]);
    assert!(!html.contains("data-lsx-tour"), "{html}");
    assert!(!html.contains(r#"role="dialog""#));
}

#[test]
fn a_started_tour_shows_a_named_modal_card_on_the_first_step() {
    reset();
    let html = rendered(&[Move::Start]);
    assert!(html.contains("data-lsx-tour"));
    let dialog = tags_with(&html, r#"role="dialog""#);
    assert_eq!(dialog.len(), 1, "{html}");
    let card = dialog[0];
    assert!(card.contains(r#"aria-modal="true""#), "{card}");
    assert!(card.contains(r#"data-autofocus="true""#), "{card}");
    assert!(card.contains(r#"tabindex="-1""#), "{card}");
    assert!(card.contains(r#"data-slot="card""#), "{card}");

    // Named by its title, described by its body.
    let at = card.find(r#"aria-labelledby=""#).expect("a labelled card") + 17;
    let title_id = &card[at..at + card[at..].find('"').unwrap()];
    assert!(tags_with(&html, &format!(r#"id="{title_id}""#))[0].contains("title"));
    assert!(html.contains(">Welcome<"));
    let at = card
        .find(r#"aria-describedby=""#)
        .expect("a described card")
        + 18;
    let body_id = &card[at..at + card[at..].find('"').unwrap()];
    assert!(tags_with(&html, &format!(r#"id="{body_id}""#))[0].contains(r#"data-slot="body""#));
    assert!(html.contains("A quick look around."));

    assert_eq!(slot_text(&html, "progress"), "1 of 3");
    assert!(slot(&html, "skip").is_some());
    assert!(
        slot(&html, "previous").is_none(),
        "no Back on the first step"
    );
    assert!(html.contains(">Next<") && !html.contains(">Done<"));
    // A step without a target: the dimming is a zero-size box in the middle.
    assert!(
        slot(&html, "highlight")
            .unwrap()
            .contains("left:50%;top:50%;width:0px")
    );
}

#[test]
fn next_and_prev_move_one_step_and_the_last_offers_done() {
    reset();
    let html = rendered(&[Move::Start, Move::Next, Move::Next]);
    assert_eq!(slot_text(&html, "progress"), "3 of 3");
    assert!(html.contains(">Done<") && !html.contains(">Next<"));
    assert!(slot(&html, "previous").is_some());
    assert!(slot(&html, "skip").is_none(), "no Skip on the last step");
    // Rich content takes the description's place.
    assert!(html.contains("<strong>Saves</strong>"));

    reset();
    let html = rendered(&[Move::Start, Move::Next, Move::Prev, Move::Prev]);
    assert_eq!(slot_text(&html, "progress"), "1 of 3");
    assert_eq!(CHANGES.with(|c| c.borrow().clone()), [1, 0]);
}

#[test]
fn go_to_clamps_to_the_last_step() {
    reset();
    let html = rendered(&[Move::Start, Move::GoTo(99)]);
    assert_eq!(slot_text(&html, "progress"), "3 of 3");
    assert_eq!(CHANGES.with(|c| c.borrow().clone()), [2]);
}

#[test]
fn next_on_the_last_step_finishes_once_and_closes() {
    reset();
    let html = rendered(&[Move::Start, Move::GoTo(2), Move::Next, Move::Next]);
    assert!(!html.contains("data-lsx-tour"), "{html}");
    assert_eq!(FINISHED.get(), 1);
    assert!(CLOSED.with(|c| c.borrow().is_empty()));
}

#[test]
fn closing_reports_the_step_it_left() {
    reset();
    let html = rendered(&[Move::Start, Move::Next, Move::Close, Move::Close]);
    assert!(!html.contains("data-lsx-tour"));
    assert_eq!(CLOSED.with(|c| c.borrow().clone()), [1]);
    assert_eq!(FINISHED.get(), 0);
}

#[test]
fn a_restart_begins_at_the_first_step() {
    reset();
    let html = rendered(&[Move::Start, Move::Next, Move::Close, Move::Start]);
    assert_eq!(slot_text(&html, "progress"), "1 of 3");
}

#[test]
fn a_controlled_tour_only_asks_to_move() {
    reset();
    CURRENT.set(Some(1));
    let html = rendered(&[Move::Start, Move::Next]);
    assert_eq!(slot_text(&html, "progress"), "2 of 3");
    assert_eq!(CHANGES.with(|c| c.borrow().clone()), [2]);
}

#[test]
fn an_untitled_step_is_named_by_the_localization() {
    reset();
    UNTITLED.set(true);
    let html = rendered(&[Move::Start]);
    let card = tags_with(&html, r#"role="dialog""#)[0].to_string();
    assert!(card.contains(r#"aria-label="Tour""#), "{card}");
    assert!(!card.contains("aria-labelledby"), "{card}");
}

#[test]
fn the_card_callback_draws_inside_the_named_card() {
    reset();
    CUSTOM.set(true);
    let html = rendered(&[Move::Start, Move::Next]);
    let card = tags_with(&html, r#"role="dialog""#);
    assert_eq!(card.len(), 1, "{html}");
    assert!(card[0].contains(r#"aria-label="Search""#), "{}", card[0]);
    assert!(card[0].contains(r#"data-slot="card""#));
    assert!(html.contains("Step 1 of 3: search"));
    assert!(
        slot(&html, "footer").is_none(),
        "the default card's footer is gone"
    );
}

#[test]
fn german_words_reach_the_card() {
    reset();
    GERMAN.set(true);
    let html = rendered(&[Move::Start, Move::Next]);
    assert_eq!(slot_text(&html, "progress"), "2 von 3");
    assert!(html.contains(">Weiter<") && html.contains(">Zurück<"));
    assert!(html.contains(">Überspringen<"));
    assert!(html.contains(r#"aria-label="Rundgang schließen""#));
}

#[test]
fn the_card_names_its_arrow_keys_next_first() {
    reset();
    let html = rendered(&[Move::Start]);
    let card = tags_with(&html, r#"role="dialog""#)[0].to_string();
    assert!(
        card.contains(r#"aria-keyshortcuts="ArrowRight ArrowLeft""#),
        "{card}"
    );

    reset();
    RTL.set(true);
    let html = rendered(&[Move::Start]);
    let card = tags_with(&html, r#"role="dialog""#)[0].to_string();
    assert!(
        card.contains(r#"aria-keyshortcuts="ArrowLeft ArrowRight""#),
        "{card}"
    );

    reset();
    KEYBOARD_OFF.set(true);
    let html = rendered(&[Move::Start]);
    assert!(!html.contains("aria-keyshortcuts"), "{html}");

    reset();
    CUSTOM.set(true);
    let html = rendered(&[Move::Start]);
    let card = tags_with(&html, r#"role="dialog""#)[0].to_string();
    assert!(card.contains("aria-keyshortcuts"), "{card}");
}

#[test]
fn a_hole_without_size_has_no_ring() {
    reset();
    let html = rendered(&[Move::Start]);
    assert!(
        !slot(&html, "highlight").unwrap().contains("data-ringed"),
        "{html}"
    );
}

#[test]
fn only_a_step_change_lets_the_hole_glide() {
    reset();
    let html = rendered(&[Move::Start]);
    assert!(
        !slot(&html, "highlight").unwrap().contains("data-moving"),
        "{html}"
    );

    reset();
    LATER.with(|later| *later.borrow_mut() = vec![Move::Next]);
    let html = rendered(&[Move::Start]);
    assert_eq!(slot_text(&html, "progress"), "2 of 3");
    assert!(
        slot(&html, "highlight")
            .unwrap()
            .contains(r#"data-moving="true""#),
        "{html}"
    );
}

#[test]
fn steps_going_empty_close_the_tour() {
    reset();
    LATER.with(|later| *later.borrow_mut() = vec![Move::Next, Move::Empty]);
    let html = rendered(&[Move::Start]);
    assert!(!html.contains("data-lsx-tour"), "{html}");
    assert!(!OPEN.get(), "the handle still says open");
    assert_eq!(CLOSED.with(|c| c.borrow().clone()), [0]);
}
