use dioxus::prelude::*;
use libero::components::Pictogram;
use pictogram_icons_lucide::{
    accessibility, book_open, calendar_check, check, chevron_right, clipboard_check, code,
    credit_card, file, folder, palette, search, sparkles, text_align_center, text_align_end,
    text_align_start, truck, utensils, x,
};

#[component]
pub fn ChevronIcon() -> Element {
    rsx! {
        Pictogram { icon: chevron_right::outlined }
    }
}

#[component]
pub fn CheckmarkIcon() -> Element {
    rsx! {
        Pictogram { icon: check::outlined }
    }
}

#[component]
pub fn FolderIcon() -> Element {
    rsx! {
        Pictogram { icon: folder::outlined }
    }
}

#[component]
pub fn FileIcon() -> Element {
    rsx! {
        Pictogram { icon: file::outlined }
    }
}

#[component]
pub fn CodeIcon() -> Element {
    rsx! {
        Pictogram { icon: code::outlined }
    }
}

#[component]
pub fn AccessibilityIcon() -> Element {
    rsx! {
        Pictogram { icon: accessibility::outlined }
    }
}

#[component]
pub fn GitHubIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            fill: "currentColor",
            path {
                d: "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82a7.4 7.4 0 0 1 2-.27c.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"
            }
        }
    }
}

#[component]
pub fn MarkdownIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { x: "2", y: "5", width: "20", height: "14", rx: "2" }
            polyline { points: "6 15 6 9 9 12 12 9 12 15" }
            polyline { points: "16 9 16 15 19 15" }
        }
    }
}

#[component]
pub fn AlignLeftIcon() -> Element {
    rsx! {
        Pictogram { icon: text_align_start::outlined }
    }
}

#[component]
pub fn AlignCenterIcon() -> Element {
    rsx! {
        Pictogram { icon: text_align_center::outlined }
    }
}

#[component]
pub fn AlignRightIcon() -> Element {
    rsx! {
        Pictogram { icon: text_align_end::outlined }
    }
}

#[component]
pub fn DismissIcon() -> Element {
    rsx! {
        Pictogram { icon: x::outlined }
    }
}

#[component]
pub fn TruckIcon() -> Element {
    rsx! {
        Pictogram { icon: truck::outlined }
    }
}

#[component]
pub fn CreditCardIcon() -> Element {
    rsx! {
        Pictogram { icon: credit_card::outlined }
    }
}

#[component]
pub fn ClipboardCheckIcon() -> Element {
    rsx! {
        Pictogram { icon: clipboard_check::outlined }
    }
}

#[component]
pub fn SearchIcon() -> Element {
    rsx! {
        Pictogram { icon: search::outlined }
    }
}

#[component]
pub fn UtensilsIcon() -> Element {
    rsx! {
        Pictogram { icon: utensils::outlined }
    }
}

#[component]
pub fn CalendarCheckIcon() -> Element {
    rsx! {
        Pictogram { icon: calendar_check::outlined }
    }
}

#[component]
pub fn BookOpenIcon() -> Element {
    rsx! {
        Pictogram { icon: book_open::outlined }
    }
}

#[component]
pub fn PaletteIcon() -> Element {
    rsx! {
        Pictogram { icon: palette::outlined }
    }
}

#[component]
pub fn SparklesIcon() -> Element {
    rsx! {
        Pictogram { icon: sparkles::outlined }
    }
}
