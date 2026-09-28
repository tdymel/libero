use std::sync::LazyLock;

use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Code, Flex, Pagination, Pictogram, Select, Text, TextField},
    hooks::use_debounced_value,
    sx::{StaticSx, sx},
};
use pictogram_core::{Icon, Library};

/// Cells per page: enough to browse, few enough inline svgs for Blitz.
const PER_PAGE: usize = 120;

static GRID_SX: StaticSx = StaticSx::new(|| {
    sx().display("grid")
        .grid_template_columns("repeat(auto-fill, minmax(112px, 1fr))")
        .gap("xs")
});

/// One icon set as the catalogue lists it. Only how the list is filled may change (1358).
#[derive(Clone, PartialEq)]
pub struct CatalogueSet {
    /// The crate's set name, e.g. `lucide`.
    pub name: &'static str,
    pub title: &'static str,
    pub license: &'static str,
    pub repository: &'static str,
    pub upstream_version: &'static str,
    pub variants: Vec<&'static str>,
    pub icons: Vec<Icon>,
}

impl CatalogueSet {
    /// A set from its crate's index, colour variants left out.
    fn from_library(library: &Library) -> Self {
        Self {
            name: library.name,
            title: library.title,
            license: library.license,
            repository: library.repository,
            upstream_version: library.upstream_version,
            variants: library
                .variants
                .iter()
                .copied()
                .filter(|v| shown(v))
                .collect(),
            icons: library
                .icons
                .iter()
                .copied()
                .filter(|i| shown(i.variant))
                .collect(),
        }
    }

    /// The icons of `variant` that match `query`, in index order.
    fn filtered(&self, variant: &str, query: &str) -> Vec<Icon> {
        self.icons
            .iter()
            .filter(|icon| icon.variant == variant && icon.matches(query))
            .copied()
            .collect()
    }

    /// The path of the icon's const, as an app writes it.
    fn rust_path(&self, icon: &Icon) -> String {
        format!(
            "pictogram_icons_{}::{}::{}",
            self.name.replace('-', "_"),
            icon.module,
            icon.variant
        )
    }
}

/// Lobe's colour variants hard-code fills and share gradient ids across copies.
fn shown(variant: &str) -> bool {
    variant != "color" && !variant.ends_with("_color")
}

/// Every set the catalogue offers.
fn catalogue_sets() -> &'static [CatalogueSet] {
    static SETS: LazyLock<Vec<CatalogueSet>> =
        LazyLock::new(|| vec![CatalogueSet::from_library(&pictogram_icons_lucide::LIBRARY)]);
    &SETS
}

/// Page count for `len` icons; at least one, so an empty search keeps its page.
fn page_count(len: usize) -> u32 {
    len.div_ceil(PER_PAGE).max(1) as u32
}

/// The icons of 1-based `page`, clamped into range.
fn page_slice(icons: &[Icon], page: u32) -> &[Icon] {
    let page = page.clamp(1, page_count(icons.len())) as usize;
    let start = ((page - 1) * PER_PAGE).min(icons.len());
    &icons[start..(start + PER_PAGE).min(icons.len())]
}

/// Every icon of the sets docs carries the index of, by set and variant, with search.
#[component]
pub fn IconCatalogue() -> Element {
    let sets = catalogue_sets();
    let mut set_name = use_signal(|| sets[0].name.to_string());
    let mut variant = use_signal(|| sets[0].variants[0].to_string());
    let mut query = use_signal(String::new);
    let settled = use_debounced_value(query.into(), 200);
    let mut page = use_signal(|| 1u32);
    use_effect(move || {
        settled.read();
        page.set(1);
    });

    let set = sets
        .iter()
        .find(|set| set.name == set_name())
        .unwrap_or(&sets[0]);
    let icons = set.filtered(&variant(), &settled());
    let count = icons.len();
    let shown = page_slice(&icons, page());

    rsx! {
        Flex { direction: "column", gap: "sm",
            Text {
                "{set.title} {set.upstream_version}, "
                Code { source: set.license }
                ", "
                Anchor { to: set.repository, target: "_blank", "{set.repository}" }
            }
            Flex { direction: "row", gap: "sm", wrap: "wrap", align: "flex-end",
                Select {
                    label: "Set",
                    options: sets.iter().map(|set| set.name.to_string()).collect::<Vec<_>>(),
                    value: Some(set_name()),
                    onchange: move |next: Option<String>| {
                        if let Some(next) = next {
                            let first = sets.iter().find(|set| set.name == next).map(|set| set.variants[0]);
                            variant.set(first.unwrap_or_default().to_string());
                            set_name.set(next);
                            page.set(1);
                        }
                    },
                }
                Select {
                    label: "Variant",
                    options: set.variants.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                    value: Some(variant()),
                    onchange: move |next: Option<String>| {
                        if let Some(next) = next {
                            variant.set(next);
                            page.set(1);
                        }
                    },
                }
                TextField {
                    label: "Search",
                    placeholder: "arrow left",
                    value: query(),
                    oninput: move |text| query.set(text),
                }
            }
            Text { size: "sm", role: "status", "{count} icons" }
            Box { framework_sx: &GRID_SX,
                for icon in shown.iter() {
                    Flex {
                        key: "{icon.module}-{icon.variant}",
                        direction: "column",
                        align: "center",
                        gap: "xs",
                        sx: sx().padding("xs").min_width("0"),
                        // The const's path on hover; no copy button per cell (todo 1384).
                        title: set.rust_path(icon),
                        Pictogram { icon: icon.svg, width: "24px", height: "24px" }
                        Text {
                            size: "xs",
                            sx: sx().max_width("100%").overflow("hidden").text_overflow("ellipsis").white_space("nowrap"),
                            title: icon.name,
                            "{icon.name}"
                        }
                    }
                }
            }
            Pagination {
                total: page_count(count),
                page: page(),
                onchange: move |next| page.set(next),
                aria_label: "Icon pages",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libero::context::LiberoProvider;

    fn lucide() -> &'static CatalogueSet {
        &catalogue_sets()[0]
    }

    #[test]
    fn lucide_is_indexed() {
        let set = lucide();
        assert_eq!(set.name, "lucide");
        assert_eq!(set.variants, ["outlined"]);
        assert!(set.icons.len() > 1500);
    }

    #[test]
    fn search_narrows_by_words() {
        let found = lucide().filtered("outlined", "arrow left");
        assert!(found.iter().any(|icon| icon.name == "arrow-left"));
        assert!(
            found
                .iter()
                .all(|icon| icon.name.contains("arrow") && icon.name.contains("left"))
        );
        assert!(lucide().filtered("filled", "").is_empty());
    }

    #[test]
    fn pages_hold_at_most_120_and_clamp() {
        let icons = lucide().filtered("outlined", "");
        let last = page_count(icons.len());
        assert_eq!(page_slice(&icons, 1).len(), PER_PAGE);
        assert_eq!(page_slice(&icons, 1)[0], icons[0]);
        assert_eq!(page_slice(&icons, last + 5), page_slice(&icons, last));
        assert_eq!(page_count(0), 1);
        assert!(page_slice(&[], 1).is_empty());
    }

    #[test]
    fn path_is_the_const() {
        let set = lucide();
        let icon = set
            .icons
            .iter()
            .find(|icon| icon.name == "arrow-up")
            .unwrap();
        assert_eq!(
            set.rust_path(icon),
            "pictogram_icons_lucide::arrow_up::outlined"
        );
    }

    #[test]
    fn colour_variants_are_hidden() {
        assert!(shown("mono") && shown("brand") && shown("outlined"));
        assert!(!shown("color") && !shown("brand_color") && !shown("text_color"));
    }

    #[test]
    fn renders_the_first_page() {
        fn app() -> Element {
            rsx! {
                LiberoProvider { IconCatalogue {} }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("ISC"), "{html}");
        assert!(html.contains(">a-arrow-down<"), "{html}");
        let total = lucide().icons.len();
        assert!(html.contains(&format!("{total} icons")), "{html}");
        assert!(html.matches("<svg").count() >= PER_PAGE);
    }
}
