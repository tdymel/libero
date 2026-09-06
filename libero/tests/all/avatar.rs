//! `Avatar`'s rendered contract: the fallback chain, the accessible name, and
//! the overflow chip an `AvatarGroup` collapses its extra people into.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Avatar, AvatarGroup, AvatarSpec},
};

fn people() -> Vec<AvatarSpec> {
    vec![
        AvatarSpec {
            name: "Ada Lovelace".into(),
            initials: Some("AL".into()),
            ..Default::default()
        },
        AvatarSpec::from("Grace Hopper"),
        AvatarSpec::from("Katherine Johnson"),
        AvatarSpec::from("Radia Perlman"),
        AvatarSpec::from("Barbara Liskov"),
    ]
}

fn initials_app() -> Element {
    rsx! {
        LiberoProvider { Avatar { name: "Ada Lovelace", initials: "AL" } }
    }
}

fn picture_app() -> Element {
    rsx! {
        LiberoProvider { Avatar { name: "Grace Hopper", src: "/grace.png", initials: "GH" } }
    }
}

fn glyph_app() -> Element {
    rsx! {
        LiberoProvider { Avatar { name: "Nobody" } }
    }
}

fn children_app() -> Element {
    rsx! {
        LiberoProvider {
            Avatar { name: "Ada Lovelace", initials: "AL", span { "\u{2605}" } }
        }
    }
}

fn decorative_app() -> Element {
    rsx! {
        LiberoProvider { Avatar { name: "Ada Lovelace", alt: "", initials: "AL" } }
    }
}

fn caller_role_app() -> Element {
    rsx! {
        LiberoProvider {
            Avatar { name: "Ada Lovelace", initials: "AL", role: "presentation" }
        }
    }
}

fn group_app() -> Element {
    rsx! {
        LiberoProvider { AvatarGroup { max: 3, people: people() } }
    }
}

fn whole_group_app() -> Element {
    rsx! {
        LiberoProvider { AvatarGroup { people: people() } }
    }
}

/// `role="img"` plus the name makes the subtree presentational, so a screen
/// reader says "Ada Lovelace" instead of spelling out the two letters.
#[test]
fn it_announces_the_name_rather_than_the_initials() {
    let html = body(&render(initials_app));
    let root = attributes_of(&html, "span");

    assert_eq!(root["role"], "img");
    assert_eq!(root["aria-label"], "Ada Lovelace");
    assert!(html.contains(">AL</span>"), "{html}");
}

/// The picture wins the chain, and it is `alt=""` because the root is the
/// image as far as the accessibility tree is concerned.
#[test]
fn a_picture_replaces_the_initials_and_carries_no_name_of_its_own() {
    let html = body(&render(picture_app));

    assert_eq!(attributes_of(&html, "img")["src"], "/grace.png");
    assert_eq!(attributes_of(&html, "img")["alt"], "");
    assert!(!html.contains("GH"), "{html}");
}

/// Nothing is derived from `name`: with no picture and no initials, the last
/// link in the chain is the glyph, never the first two letters.
#[test]
fn a_name_alone_falls_through_to_the_person_glyph() {
    let html = body(&render(glyph_app));

    assert!(html.contains("<svg"), "{html}");
    assert!(!html.contains(">No<"), "{html}");
}

#[test]
fn children_override_the_initials() {
    let html = body(&render(children_app));

    assert!(html.contains("\u{2605}"), "{html}");
    assert!(!html.contains(">AL<"), "{html}");
}

/// The image rule: an avatar beside the person's visible name is decorative,
/// and `alt: ""` is how a caller says so.
#[test]
fn an_empty_alt_marks_the_avatar_decorative() {
    let html = body(&render(decorative_app));
    let root = attributes_of(&html, "span");

    assert_eq!(root["role"], "presentation");
    assert!(!root.contains_key("aria-label"), "{root:?}");
    // `role="presentation"` drops the element's own semantics and leaves its
    // contents in the tree, so the initials would still be read as text.
    assert_eq!(root["aria-hidden"], "true");
}

/// Through `attr_default`, not `attr`: a component's own attributes render
/// after the caller's, so `attr` would silently win the duplicate.
#[test]
fn a_callers_own_role_wins() {
    let html = body(&render(caller_role_app));

    assert_eq!(attributes_of(&html, "span")["role"], "presentation");
    assert_eq!(html.matches("role=").count(), 1, "{html}");
}

/// `max` is the number of circles, the chip included - so three circles over
/// five people are two avatars and a chip standing for the other three.
#[test]
fn the_overflow_chip_counts_the_people_it_hides() {
    let html = body(&render(group_app));

    assert_eq!(html.matches("role=\"img\"").count(), 3, "{html}");
    assert!(html.contains(">+3</span>"), "{html}");
    assert!(
        html.contains("aria-label=\"3 more: Katherine Johnson, Radia Perlman, Barbara Liskov\""),
        "{html}"
    );
}

/// The names are in the chip's own label as well as in its tooltip, so a
/// tooltip clipped by an `overflow: hidden` ancestor is cosmetic. The chip is
/// focusable, which is what makes the tooltip reachable without a pointer.
#[test]
fn the_hidden_names_are_reachable_by_keyboard() {
    let html = body(&render(group_app));

    assert!(html.contains("tabindex=\"0\""), "{html}");
    assert!(
        html.contains("role=\"tooltip\">Katherine Johnson, Radia Perlman, Barbara Liskov"),
        "{html}"
    );
}

/// Counting down from the number of circles, so the first avatar paints over
/// the second and the chip sits under all of them. DOM order stays visual
/// order - nothing is reordered in CSS.
#[test]
fn the_first_avatar_paints_over_the_second() {
    let html = body(&render(group_app));

    let indices: Vec<&str> = html
        .match_indices("--lsx-avatar-group-index:")
        .map(|(at, key)| {
            let rest = &html[at + key.len()..];
            &rest[..rest.find(';').expect("a terminated declaration")]
        })
        .collect();

    assert_eq!(indices, vec!["3", "2", "1"], "{html}");
}

/// The chip's vars reach it through `Box`'s `variables`, where a var dropped
/// on a later render is reverted, rather than a raw `style` attribute, where it
/// would stay. SSR sees one render, so this pins only that they still arrive.
#[test]
fn the_chip_carries_its_vars() {
    let html = body(&render(group_app));
    let chip = &html[..html.find(">+3</span>").expect("a chip")];
    let chip = &chip[chip.rfind("<span").expect("the chip's tag")..];

    assert!(chip.contains("--lsx-avatar-group-index:1;"), "{chip}");
    assert!(chip.contains("--lsx-avatar-color:"), "{chip}");
}

/// No `max`, so every person is a circle and there is nothing to collapse.
#[test]
fn a_group_under_its_max_grows_no_chip() {
    let html = body(&render(whole_group_app));

    assert_eq!(html.matches("role=\"img\"").count(), 5, "{html}");
    assert!(!html.contains("role=\"tooltip\""), "{html}");
}

/// The ring in the page colour is what makes two overlapping circles legible,
/// and the group sets it on its members directly - no context provider.
#[test]
fn a_member_is_told_it_is_in_a_group() {
    let html = body(&render(group_app));

    assert_eq!(html.matches("grouped").count(), 3, "{html}");
    assert!(!body(&render(initials_app)).contains("grouped"));
}

/// Measured in Chromium on `/data-display/avatar`, 2026-09-18 (todo 64): with a
/// plain `center`, a ten-character label in a 38px `md` circle was cut 25.7px at
/// *each* end and the first visible glyph was the fourth - the start, which is
/// what identifies the person, was gone, and the overflow ahead of the box was
/// not even scrollable (`scrollWidth` 64 against a 89px label). `safe` keeps the
/// centring while the label fits and falls back to the start edge once it does
/// not. Nothing in the layout is visible to SSR, so what this pins is the
/// declaration itself.
#[test]
fn an_over_long_label_is_cut_at_the_end_rather_than_at_both_ends() {
    let html = render(initials_app);

    assert!(html.contains("justify-content:safe center"), "{html}");
    assert!(!html.contains("justify-content:center"), "{html}");
}

/// A group's `radius` reaches every member through `Avatar`'s own radius
/// scale, whose `xxl` circle is the theme's default.
#[test]
fn a_group_radius_step_resolves_through_the_avatar_scale() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { AvatarGroup { people: people(), radius: "md" } }
        }
    }

    let html = render(app);
    assert!(html.contains("--lsx-avatar-radius-xxl:9999px;"), "{html}");
    assert!(
        html.contains("--lsx-avatar-radius:var(--lsx-avatar-radius-xxl);"),
        "{html}"
    );
    assert_eq!(
        html.matches("--lsx-avatar-radius-override:var(--lsx-avatar-radius-md);")
            .count(),
        people().len(),
        "{html}"
    );
}
