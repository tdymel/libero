use dioxus::prelude::*;
use libero::components::{Pictogram, Text, TimelineEvent, TimelineLine};
use pictogram_icons_lucide as lucide;

/// Four fixed events, with a bullet icon each or none, and one dashed connector or none.
pub fn demo_items(bullets: bool, dashed: bool) -> Vec<TimelineEvent> {
    let rows: [(&str, &str); 4] = [
        ("Pushed to main", "3 commits"),
        ("Review requested", "@tom"),
        ("Deployed", "v2.4.0 to production"),
        ("Rolled back", "reverted in 4 minutes"),
    ];

    rows.iter()
        .enumerate()
        .map(|(index, (title, content))| {
            let content = content.to_string();
            let mut event = TimelineEvent::new(*title).content(rsx! { Text { "{content}" } });
            if bullets {
                event = event.bullet(match index {
                    0 => rsx! { Pictogram { icon: lucide::code::outlined } },
                    1 => rsx! { Pictogram { icon: pictogram_icons_simple::github::regular } },
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
