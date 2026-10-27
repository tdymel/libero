use dioxus::prelude::*;
use libero::components::{Pictogram, Text, TimelineEvent, TimelineLine};
use pictogram_icons_lucide as lucide;

/// Four fixed events, with a bullet icon each or none, one dashed connector or none,
/// and a time across the rail or none.
pub fn demo_items(bullets: bool, dashed: bool, opposite: bool) -> Vec<TimelineEvent> {
    let rows: [(&str, &str, &str); 4] = [
        ("Pushed to main", "3 commits", "09:12"),
        ("Review requested", "@tom", "09:40"),
        ("Deployed", "v2.4.0 to production", "10:05"),
        ("Rolled back", "reverted in 4 minutes", "10:09"),
    ];

    rows.iter()
        .enumerate()
        .map(|(index, (title, content, time))| {
            let content = content.to_string();
            let time = time.to_string();
            let mut event = TimelineEvent::new(*title).content(rsx! { Text { "{content}" } });
            if opposite {
                event = event.opposite(rsx! { Text { "{time}" } });
            }
            if bullets {
                event = event.bullet(match index {
                    0 => rsx! { Pictogram { icon: lucide::code::outlined } },
                    // Outlined: a solid glyph would cover the active fill.
                    1 => rsx! { Pictogram { icon: lucide::git_pull_request::outlined } },
                    2 => rsx! { Pictogram { icon: lucide::check::outlined } },
                    _ => rsx! { Pictogram { icon: lucide::file::outlined } },
                });
            }
            // On the second-to-last: its connector reaches the final bullet.
            if dashed && index == rows.len() - 2 {
                event = event.line(TimelineLine::Dashed);
            }
            event
        })
        .collect()
}
