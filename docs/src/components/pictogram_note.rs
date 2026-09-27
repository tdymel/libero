use dioxus::prelude::*;
use libero::components::{Anchor, Text};

pub const PICTOGRAM_REPO: &str = "https://github.com/tdymel/pictogram";

/// The pictogram promo at the top of the Icon, ActionIcon, Pictogram and IconProvider pages.
#[component]
pub fn PictogramNote() -> Element {
    rsx! {
        Text { size: "sm", color: "text-dimmed",
            "Need icons? "
            Anchor { to: PICTOGRAM_REPO, target: "_blank", "pictogram" }
            " ships lucide, Tabler, Material and more."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libero::context::LiberoProvider;

    #[test]
    fn the_note_uses_the_dimmed_text_role() {
        fn app() -> Element {
            rsx! {
                LiberoProvider { PictogramNote {} }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        // `muted` drew the 14px note at 3.32:1; the dimmed text role holds 4.5:1.
        assert!(
            html.contains("--lsx-text-color:var(--lsx-text-dimmed)"),
            "{html}"
        );
    }
}
