use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    stage::{Stage, image_id, lightbox_slide, lightbox_thumbnails, use_lightbox_drag},
    styles::{
        LIGHTBOX_BODY_SX, LIGHTBOX_CAPTION_SX, LIGHTBOX_DIALOG_SX, LIGHTBOX_STAGE_SX,
        LIGHTBOX_THUMBNAILS_SHOWN, LIGHTBOX_THUMBNAILS_SX, LIGHTBOX_TOOLBAR_SX,
    },
    use_lightbox::{LightboxOpening, LightboxOptions},
    zoom::{Fit, Gesture, Slide, Zoom, Zooming, refit, reopened, wheel_step, zoom_about},
};
use crate::{
    components::{
        accessibility::use_announcer,
        buttons::ActionIcon,
        common::{
            Glyph, Input, Part, Variables, parts_enum, recast_parts, use_name_warning, variables,
        },
        data_display::{Carousel, CarouselJump, CarouselQuietWhenFits},
        layout::Box,
        overlay::{Dialog, use_modal::use_modal_close},
    },
    context::IconSlot,
    hooks::{ElementHandle, id_selector, use_element, use_id, use_localization, use_theme},
    platform::ElementApi,
    sx::{StaticSx, Sx},
};

parts_enum! {
    /// The viewer's inner parts, for [`LightboxOptions::parts`]. A carousel sits
    /// between the stage and its frames, so those are descendants.
    pub enum LightboxPart {
        /// The row of zoom and close buttons.
        Toolbar = "toolbar" => "& > [data-slot='toolbar']",
        ZoomOut = "zoom-out" => "& > [data-slot='toolbar'] > [data-slot='zoom-out']",
        ZoomIn = "zoom-in" => "& > [data-slot='toolbar'] > [data-slot='zoom-in']",
        Close = "close" => "& > [data-slot='toolbar'] > [data-slot='close']",
        /// Everything under the toolbar: stage, caption, strip.
        Body = "body" => "& > [data-slot='body']",
        Stage = "stage" => "& > [data-slot='body'] > [data-slot='stage']",
        /// One picture's box, which clips the zoom and draws the focus ring.
        Frame = "frame" => "& > [data-slot='body'] > [data-slot='stage'] [data-slot='frame']",
        Image = "image" => "& > [data-slot='body'] > [data-slot='stage'] [data-slot='frame'] > [data-slot='image']",
        Caption = "caption" => "& > [data-slot='body'] > [data-slot='caption']",
        /// The thumbnail strip.
        Thumbnails = "thumbnails" => "& > [data-slot='body'] > [data-slot='thumbnails']",
        /// One thumbnail button; the current one has `aria-current="true"`.
        Thumbnail = "thumbnail" => "& > [data-slot='body'] > [data-slot='thumbnails'] [data-slot='thumbnail']",
    }
}

/// `base` with the caller's `sx` over it.
fn sx_over(base: &'static StaticSx, sx: &Input<Sx>) -> Input<Sx> {
    match sx.as_ref() {
        None => base.into(),
        Some(sx) => Input::Value((**base).clone().and(sx.clone())),
    }
}

/// The viewer [`crate::hooks::use_lightbox`] opens. Only rendered inside that
/// hook's modal.
#[component]
pub(crate) fn Lightbox(opening: LightboxOpening, options: LightboxOptions) -> Element {
    let theme = use_theme();
    let localization = use_localization();
    let close = use_modal_close();
    let stage = use_element();
    let base_id = use_id();
    let caption_id = use_id();

    let items = opening.items.clone();
    let count = items.len();
    let last = count.saturating_sub(1);
    let mut index = use_signal(|| opening.index.min(last));
    let mut zoom = use_signal(|| Zoom::fitted(usize::MAX));
    // The opening `index` and `zoom` were last reset for, and whether a
    // picture held focus when a new one arrived.
    let settled = use_hook(|| Rc::new(RefCell::new((opening.clone(), false))));
    // A second `open_with` is drawn at its own index before the effect resets
    // `index`: an `eager` `<img>` round the old index would fetch (todo 227).
    let pending = settled.borrow().0 != opening;
    // The swap jumps, so a smooth scroll fetches nothing it passes. Counted once
    // per gallery, or the user's next move would jump too.
    let jump = use_context_provider(CarouselJump::default);
    // Thumbnails that all fit drop the strip's status and track tab stop (todo
    // 564). The one-up stage never fits: one picture renders no carousel.
    use_context_provider(|| CarouselQuietWhenFits);
    let counted = use_hook(|| Rc::new(RefCell::new(opening.clone())));
    if pending && *counted.borrow() != opening {
        *counted.borrow_mut() = opening.clone();
        jump.swapped();
    }
    if pending {
        // The old `<img>`s are about to go, so focus on one would drop to the page.
        let focused = stage
            .query_selector("[data-lightbox-frame] img:focus")
            .is_ok();
        settled.borrow_mut().1 = focused;
    }
    // A zoom is keyed by index only, so it would carry onto whichever new
    // picture lands at the same index.
    use_effect(use_reactive!(|opening| {
        let (target, fitted) = reopened(*zoom.peek(), opening.index, opening.items.len());
        if *index.peek() != target {
            index.set(target);
        }
        if *zoom.peek() != fitted {
            zoom.set(fitted);
        }
        let refocus = std::mem::replace(&mut *settled.borrow_mut(), (opening, false)).1;
        if refocus
            && let Ok(image) = stage.query_selector(&id_selector(&image_id(&base_id(), target)))
        {
            let _ = image.focus();
        }
    }));

    let zooming = Zooming {
        index,
        zoom,
        fit: use_signal(|| None::<Fit>),
        gesture: use_signal(|| None::<Gesture>),
        dragged: use_signal(|| false),
        touches: use_signal(Vec::new),
        max_zoom: options.max_zoom.unwrap_or(theme.lightbox.max_zoom).max(1.0),
        announcer: use_announcer(),
        labels: localization.lightbox,
    };
    let keys_id = use_id();

    // Read either way, so this render stays subscribed to the reset.
    let current = match index() {
        _ if pending => opening.index.min(last),
        settled => settled,
    };
    let active = match zoom() {
        held if held.index == current && !pending => held,
        _ => Zoom::fitted(current),
    };

    // Focused from an effect: until the carousels move, the target's slide is
    // still `inert` and `focus()` does nothing.
    let mut focus_next = use_signal(|| None::<String>);
    use_effect(move || {
        if let Some(id) = focus_next() {
            focus_next.set(None);
            if let Ok(element) = stage.query_selector(&id_selector(&id)) {
                let _ = element.focus();
            }
        }
    });

    // One frame and picture handle pair per picture, grown in render as a later
    // opening can bring more; a `RefCell`, so growing it re-renders nothing.
    let pictures = use_hook(|| Rc::new(RefCell::new(Vec::<Slide>::new())));
    {
        let mut pictures = pictures.borrow_mut();
        while pictures.len() < count {
            pictures.push(Slide {
                frame: ElementHandle::new(),
                picture: ElementHandle::new(),
            });
        }
    }
    let picture = |i: usize| pictures.borrow()[i];
    let resized_pictures = pictures.clone();

    let drag = use_lightbox_drag(
        zooming,
        if count > 0 {
            picture(current).picture
        } else {
            stage
        },
        close,
    );

    let gesture = zooming.gesture;
    let swiping = match gesture() {
        Some(Gesture::Swipe { delta }) => Some(delta.y.max(0.0)),
        _ => None,
    };
    let stage_parts = Stage {
        zooming,
        drag,
        base_id,
        focus_next,
        current,
        last,
        active,
        zoomable: options.zoom,
        swipe: options.close_on_swipe_down,
        preload: options.preload,
        swiping,
    };

    let caption = items
        .get(current)
        .and_then(|item| item.caption.clone())
        .filter(|_| options.captions);
    // The caption, then the keys: a picture that takes keys says which.
    let described = [
        caption.as_ref().map(|_| caption_id()),
        options.zoom.then(&*keys_id),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    let described = (!described.is_empty()).then_some(described);

    let slides: Vec<Element> = (0..count)
        .map(|i| lightbox_slide(stage_parts, i, &items[i], picture(i), described.clone()))
        .collect();
    use_name_warning(
        options.aria_label.is_some(),
        "Lightbox: no `aria_label`, falling back to the localization's. A dialog needs a name of its own to be told apart.",
    );
    use_name_warning(
        items.iter().all(|item| !item.alt.trim().is_empty()),
        "Lightbox: a `LightboxItem` has an empty `alt`. Its picture is focusable and would have no name.",
    );
    let label = options
        .aria_label
        .clone()
        .unwrap_or_else(|| localization.lightbox.label.to_string());

    let stage_body = match count {
        // One picture has nothing to page through, so no carousel: its region
        // and "1 of 1" would only be noise.
        0 | 1 => rsx! { {slides.into_iter()} },
        _ => rsx! {
            Carousel {
                aria_label: label.clone(),
                slides,
                index: Some(current),
                onindexchange: move |next: usize| {
                    if next != *index.peek() {
                        index.set(next);
                    }
                },
                controls: options.controls,
                indicators: false,
            }
        },
    };

    let shown = (count as f64).min(theme.lightbox.thumbnails_per_view.max(1.0));
    let thumbnails_variables: Input<Variables> = variables()
        .with(LIGHTBOX_THUMBNAILS_SHOWN, shown.to_string())
        .into();
    let thumbnails = lightbox_thumbnails(stage_parts, &items, localization.lightbox.thumbnail);
    let show_thumbnails = options.thumbnails && count > 1;

    let max_zoom = zooming.max_zoom;
    let zoom_buttons = options.zoom && count > 0 && max_zoom > 1.0;
    let (at_max, at_fit) = (active.scale >= max_zoom, !active.is_zoomed());
    // About the stage's centre, in the wheel's steps.
    let shown_picture = (count > 0 && !pending).then(|| picture(current));
    let zoom_by = move |closer: bool| {
        if let Some(Slide { frame, picture }) = shown_picture {
            zoom_about(frame, picture, zooming, current, None, move |scale| {
                wheel_step(scale, closer, max_zoom)
            });
        }
    };

    rsx! {
        Dialog {
            aria_label: label.clone(),
            close_button: false,
            sx: sx_over(&LIGHTBOX_DIALOG_SX, &options.sx),
            parts: recast_parts(options.parts.clone()),
            Box { framework_sx: &LIGHTBOX_TOOLBAR_SX, "data-slot": LightboxPart::Toolbar.slot(),
                if zoom_buttons {
                    ActionIcon {
                        "data-slot": LightboxPart::ZoomOut.slot(),
                        variant: "standard",
                        color: "muted",
                        size: "sm",
                        aria_label: localization.lightbox.zoom_out,
                        // At its limit it keeps its tab stop.
                        disabled: at_fit,
                        focusable_when_disabled: true,
                        onclick: move |_: MouseEvent| zoom_by(true),
                        Glyph { slot: IconSlot::Minus, icon: lucide::minus::outlined }
                    }
                    ActionIcon {
                        "data-slot": LightboxPart::ZoomIn.slot(),
                        variant: "standard",
                        color: "muted",
                        size: "sm",
                        aria_label: localization.lightbox.zoom_in,
                        disabled: at_max,
                        focusable_when_disabled: true,
                        onclick: move |_: MouseEvent| zoom_by(false),
                        Glyph { slot: IconSlot::Plus, icon: lucide::plus::outlined }
                    }
                }
                // Focused on open, as `Dialog`'s own close button was: a zoom
                // button first in the tab order would open disabled.
                ActionIcon {
                    "data-slot": LightboxPart::Close.slot(),
                    variant: "standard",
                    color: "muted",
                    size: "sm",
                    aria_label: localization.common.close,
                    "data-autofocus": "true",
                    onclick: move |_: MouseEvent| close.call(()),
                    Glyph { slot: IconSlot::Close, icon: lucide::x::outlined }
                }
            }
            Box {
                framework_sx: &LIGHTBOX_BODY_SX,
                "data-slot": LightboxPart::Body.slot(),
                onmounted: stage.mount(),
                Box {
                    framework_sx: &LIGHTBOX_STAGE_SX,
                    "data-slot": LightboxPart::Stage.slot(),
                    // A window resize or a phone turned moves the pan bounds.
                    onresize: move |_: Event<ResizeData>| {
                        if let Some(&Slide { frame, picture }) = resized_pictures.borrow().get(*index.peek()) {
                            refit(frame, picture, zooming);
                        }
                    },
                    {stage_body}
                }
                if let Some(caption) = caption {
                    Box {
                        component: "p",
                        id: caption_id(),
                        framework_sx: &LIGHTBOX_CAPTION_SX,
                        "data-slot": LightboxPart::Caption.slot(),
                        "{caption}"
                    }
                }
                if options.zoom {
                    span { id: keys_id(), hidden: true, "{localization.lightbox.keys}" }
                }
                {zooming.announcer.render()}
                if show_thumbnails {
                    Box {
                        framework_sx: &LIGHTBOX_THUMBNAILS_SX,
                        variables: thumbnails_variables,
                        "data-slot": LightboxPart::Thumbnails.slot(),
                        Carousel {
                            aria_label: localization.lightbox.thumbnails,
                            slides: thumbnails,
                            // Follows the picture, never leads it: a clamped
                            // index near the ends is the strip's business.
                            index: Some(current),
                            per_view: shown,
                            gap: theme.lightbox.thumbnails_gap,
                            align: "center",
                            controls: false,
                            indicators: false,
                        }
                    }
                }
            }
        }
    }
}
