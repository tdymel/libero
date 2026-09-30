use dioxus::prelude::*;
use pictogram_core::Svg as SvgData;

use super::value::{fill, number_text, sane, stepped, value_at, zone_value};
use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, States, Variables, base_color, draw_svg, has_shortcut_modifier,
            names_itself, use_name_warning, variables,
        },
        form::{field_parts_enum, field_props, use_bound, use_field},
        layout::use_box,
    },
    context::IconSlot,
    hooks::{
        Drag, DragMove, DragOptions, DragStart, ElementHandle, sideways_drag_sx, use_element,
        use_formats, use_icon, use_localization, use_sideways_drag, use_theme,
    },
    localization::fill as fill_template,
    platform::{ElementApi, logical_key},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CssVar, RATING_GAP, RATING_GLYPH, RatingDefaults},
    utils::warn,
};

/// The filled symbols' colour, resolved on the root.
const RATING_COLOR: CssVar = CssVar::new("--lsx-rating-color");

static RATING_SX: StaticSx = StaticSx::new(|| {
    let glyph = RATING_GLYPH.value();
    RatingDefaults::theme_vars()
        // A vertical swipe scrolls the page, a sideways one scrubs (1058).
        .and(sideways_drag_sx())
        .display("inline-flex")
        .align_items("center")
        // A field's column would stretch it, and a scrub maps over its width.
        .width("max-content")
        .user_select("none")
        // Half the gap each side: the hit area spans it, so zones meet.
        .selector(
            format!("& > [data-slot='{}']", RatingPart::Symbol.slot()),
            sx().position("relative")
                .display("block")
                .padding(format!("0 calc({} / 2)", RATING_GAP.value())),
        )
        // Empty glyph and fill share one cell; `start` follows the direction.
        .selector(
            format!("& [data-slot='{}']", RatingPart::Glyph.slot()),
            sx().display("grid")
                .width(glyph.clone())
                .height(glyph.clone())
                .color("muted"),
        )
        .selector(
            format!("& [data-slot='{}'] > *", RatingPart::Glyph.slot()),
            sx().grid_row("1").grid_column("1"),
        )
        .selector(
            format!("& [data-slot='{}'] svg", RatingPart::Glyph.slot()),
            sx().display("block").width(glyph.clone()).height(glyph),
        )
        .selector(
            format!("& [data-slot='{}']", RatingPart::Fill.slot()),
            sx().justify_self("start")
                .height("100%")
                .overflow("hidden")
                .color(RATING_COLOR.value()),
        )
        .selector(
            "& [data-slot='zones']",
            sx().position("absolute")
                .top("0")
                .bottom("0")
                .left("0")
                .right("0")
                .display("flex"),
        )
        .selector("& [data-slot='zone']", sx().flex("1 1 0"))
        .when("editable", sx().cursor("pointer"))
        .when("disabled", sx().opacity("0.5").cursor("not-allowed"))
});

field_parts_enum! {
    /// [`Rating`]'s inner parts, for its `parts` prop: a field's, and the symbols.
    pub enum RatingPart {
        /// The row of symbols, the slider or image.
        Control = "control" => "& > [data-slot='control']",
        /// One symbol with its hit area.
        Symbol = "symbol" => "& > [data-slot='control'] > [data-slot='symbol']",
        /// A symbol's empty glyph, under its fill.
        Glyph = "glyph" => "& > [data-slot='control'] > [data-slot='symbol'] > [data-slot='glyph']",
        /// A symbol's filled share, in the rating's colour.
        Fill = "fill" => "& > [data-slot='control'] > [data-slot='symbol'] > [data-slot='glyph'] > [data-slot='fill']",
    }
}

field_props! {
    parts(RatingPart);
    without(radius);
    pub struct RatingProps {
        /// Controlled: pair it with `onchange`, or bind a path `name` in a `Form`.
        /// Unset is 0, unrated; a value between steps draws as it is (an average).
        #[props(default)]
        value: Option<f64>,
        /// Called with the picked value; with 0 when `clearable` clears it.
        #[props(default)]
        onchange: Option<EventHandler<f64>>,
        /// The value under a mouse or pen, `None` once it leaves or presses.
        /// Drawn in place of `value` meanwhile.
        #[props(default)]
        onhover: Option<EventHandler<Option<f64>>>,
        /// What the value posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<f64>,
        /// Rules over the value, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<f64>,
        /// Number of symbols. Default 5.
        #[props(default)]
        count: Option<u8>,
        /// Steps per symbol: 1 whole symbols, 2 halves. Default 1.
        #[props(default)]
        fractions: Option<u8>,
        /// The filled symbols' colour; `theme.rating.color` by default.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// The symbol, drawn empty and filled in `currentColor`; the
        /// `IconSlot::Star` glyph by default.
        #[props(default)]
        icon: Option<SvgData>,
        /// Picking the current value again resets it to 0, as do Home and the
        /// arrows below the first step.
        #[props(default)]
        clearable: Option<bool>,
        /// `aria-valuetext`; runs during render, so it can translate.
        /// `Localization.rating` by default: "3.5 of 5".
        #[props(default)]
        format: Option<Callback<f64, String>>,
        /// Names it without a `label`.
        #[props(default)]
        aria_label: Option<String>,
        /// `false` only shows a value: an image named by it, no tab stop, no input.
        #[props(default)]
        focusable: Option<bool>,
    }
}

/// A row of symbols picking a value, whole or in fractions: click, drag
/// across, or use the arrow keys.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Rating;
/// # fn app() -> Element {
/// let mut stars = use_signal(|| 3.5);
/// rsx! {
///     Rating {
///         label: "Your rating",
///         value: stars(),
///         fractions: 2,
///         onchange: move |value| stars.set(value),
///     }
///     Rating { aria_label: "Average", value: 4.3, focusable: false }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/rating>
#[component]
pub fn Rating(props: RatingProps) -> Element {
    let theme = use_theme();
    let words = use_localization();
    let formats = use_formats();
    let root = use_element();
    let icon = use_icon(IconSlot::Star, pictogram_icons_lucide::star::outlined);
    let icon = props.icon.unwrap_or(icon);

    let count = props.count.unwrap_or(5).max(1);
    let fractions = props.fractions.unwrap_or(1).max(1);
    let clearable = props.clearable.unwrap_or(false);
    let readonly = props.readonly.unwrap_or(false);
    let display = props.focusable == Some(false);
    let size = props.size.copied_or(theme.rating.size);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let value = sane(bound.value().or(props.value).unwrap_or(0.0), count);
    let changeable = props.onchange.is_some() || bound.is_bound();
    let editable = changeable && !disabled && !readonly && !display;
    if !changeable && !disabled && !readonly && !display {
        warn("Rating: without `onchange` the value can never change.");
    }
    // Without `clearable` the first step is the floor.
    let lowest = if clearable {
        0.0
    } else {
        1.0 / f64::from(fractions)
    };

    let emit = bound.emit(props.onchange);
    let commit = use_callback(move |next: f64| {
        if let Some(emit) = &emit {
            emit(next);
        }
    });

    let mut hover = use_signal(|| None::<f64>);
    let onhover = props.onhover;
    let set_hover = use_callback(move |next: Option<f64>| {
        if *hover.peek() != next {
            hover.set(next);
            if let Some(onhover) = &onhover {
                onhover.call(next);
            }
        }
    });

    let (drag, pressed) = use_rating_drag(RatingDrag {
        root,
        editable,
        clearable,
        value,
        count,
        fractions,
        commit,
        set_hover,
    });

    let onkeydown = move |event: Event<KeyboardData>| {
        // Alt+ArrowLeft is Back, Ctrl+PageUp switches tabs: a chord is the browser's.
        if !editable || has_shortcut_modifier(&event) {
            return;
        }
        let whole = i32::from(fractions);
        let one = if event.modifiers().shift() { whole } else { 1 };
        // Under RTL ArrowLeft raises the value, as on a native range.
        let (next, down) = match logical_key(&event) {
            Key::ArrowRight | Key::ArrowUp => {
                (stepped(value, one, fractions, lowest, count), false)
            }
            Key::ArrowLeft | Key::ArrowDown => {
                (stepped(value, -one, fractions, lowest, count), true)
            }
            Key::PageUp => (stepped(value, whole, fractions, lowest, count), false),
            Key::PageDown => (stepped(value, -whole, fractions, lowest, count), true),
            Key::Home => (lowest, true),
            Key::End => (f64::from(count), false),
            _ => return,
        };
        event.prevent_default();
        // A key down never raises an unrated value to the floor.
        if next != value && !(down && next > value) {
            commit.call(next);
        }
    };

    let text = match &props.format {
        Some(format) => format.call(value),
        None => fill_template(
            words.rating.value,
            &[
                ("value", &number_text(value, formats.decimal_separator)),
                ("count", &count),
            ],
        ),
    };

    let field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&value))
        .bound(&bound)
        .required(props.required.unwrap_or(false))
        .required_in_name(words.rating.required)
        .disabled(disabled)
        .size(size)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .aria_label(props.aria_label.as_deref())
        .attributes(&props.attributes)
        .prepare();
    use_name_warning(
        field.label_id().is_some() || props.aria_label.is_some() || names_itself(&props.attributes),
        "Rating: no `label` or `aria_label`, so it is announced without a name.",
    );

    let dragging = (drag.dragging)();
    let states: Input<States> = States::default()
        .with(size.state_name(), true)
        .with("editable", editable)
        .with("dragging", dragging)
        .with("disabled", disabled)
        .with("readonly", readonly)
        .into();
    let color = match props.color.as_ref() {
        Some(color) => base_color(Some(color)),
        None => base_color(Some(&ThemeAwareValue::Color(theme.rating.color))),
    };
    let root_variables: Input<Variables> =
        variables().with(RATING_COLOR, color.resolve(None)).into();
    let style = use_box()
        .framework_sx(&RATING_SX)
        .states(&states)
        .variables(&root_variables)
        .prepare();

    // A hover preview only while nothing is pressed.
    let shown = match (editable && !dragging, hover()) {
        (true, Some(hovered)) => hovered,
        _ => value,
    };
    let mut children = render_symbols(Symbols {
        count,
        fractions,
        shown,
        icon,
        zones: editable,
        pressed,
        set_hover,
    });
    if let Some(name) = bound.name().filter(|_| !display) {
        // `Some(true)` or nothing: native writes `false` as a string.
        // Keyed past the symbols' 0..count: mixed siblings panic the diff (1169).
        children.push(rsx! {
            input {
                key: "{count}",
                r#type: "hidden",
                name: name.to_string(),
                value: number_text(value, "."),
                disabled: disabled.then_some(true),
            }
        });
    }

    let id = field.id().to_string();
    let style = style
        .element(&root)
        .attr("data-slot", RatingPart::Control.slot())
        .attr("id", id.clone());
    let control = if display {
        // The label, then the value: "Average 4.3 of 5".
        let (label, labelledby) = match (field.label_id(), &props.aria_label) {
            (Some(label), _) => (text, Some(format!("{label} {id}"))),
            (None, Some(name)) => (format!("{name}, {text}"), None),
            (None, None) => (text, None),
        };
        style
            .attr("role", "img")
            .attr("aria-label", label)
            .attr("aria-labelledby", labelledby)
            .attr("aria-describedby", field.describedby())
            .render(HtmlTag::Div, props.attributes, children)
    } else {
        style
            .attr("role", "slider")
            .attr("tabindex", if disabled { "-1" } else { "0" })
            .attr("aria-orientation", "horizontal")
            .attr("aria-valuemin", "0")
            .attr("aria-valuemax", count.to_string())
            .attr("aria-valuenow", number_text(value, "."))
            .attr("aria-valuetext", text)
            .attr("aria-label", props.aria_label)
            .attr("aria-labelledby", field.label_id())
            .attr("aria-describedby", field.describedby())
            .attr("aria-invalid", field.invalid().then_some("true"))
            // No `aria-required`: ARIA 1.2 does not allow it on `slider`.
            .attr("aria-disabled", disabled.then_some("true"))
            .attr("aria-readonly", readonly.then_some("true"))
            .event("onkeydown", onkeydown)
            .event("onpointerdown", drag.onpointerdown)
            .event("onpointermove", drag.onpointermove)
            .event("onpointerup", drag.onpointerup)
            .event("onpointercancel", drag.onpointercancel)
            .event("onmouseleave", move |_: MouseEvent| set_hover.call(None))
            .render(HtmlTag::Div, props.attributes, children)
    };

    field.render(control)
}

/// What the drag hook reads off the render.
#[derive(Clone, Copy)]
struct RatingDrag {
    root: ElementHandle,
    editable: bool,
    clearable: bool,
    value: f64,
    count: u8,
    fractions: u8,
    commit: Callback<f64>,
    set_hover: Callback<Option<f64>>,
}

/// The row's box, measured at a press.
#[derive(Clone, Copy)]
struct Row {
    left: f64,
    width: f64,
    rtl: bool,
}

/// One press, from down to up.
#[derive(Clone, Copy, Default)]
struct Press {
    /// The value before it.
    from: f64,
    /// The value it last picked.
    last: f64,
    /// Its first pick: a press on the current value, never moved off, clears.
    first: Option<f64>,
    changed: bool,
    measuring: bool,
    /// A move or the release that came while measuring.
    moved_to: Option<f64>,
    released: bool,
}

/// Press, tap and sideways scrub. A zone names the pressed value at once; a
/// move maps the pointer over the row measured at the press.
fn use_rating_drag(context: RatingDrag) -> (Drag, CopyValue<Option<f64>>) {
    let RatingDrag {
        root,
        editable,
        clearable,
        value,
        count,
        fractions,
        commit,
        set_hover,
    } = context;
    let mut pressed = use_hook(|| CopyValue::new(None::<f64>));
    let mut press = use_hook(|| CopyValue::new(Press::default()));
    let mut row = use_hook(|| CopyValue::new(None::<Row>));

    let pick = use_callback(move |next: f64| {
        let mut current = *press.peek();
        current.first.get_or_insert(next);
        if next != current.last {
            current.last = next;
            current.changed = true;
            commit.call(next);
        }
        press.set(current);
    });
    let at = use_callback(move |x: f64| {
        (*row.peek()).map(|row| {
            let along = (x - row.left) / row.width;
            let along = if row.rtl { 1.0 - along } else { along };
            value_at(along, count, fractions)
        })
    });
    let finish = use_callback(move |()| {
        let done = *press.peek();
        press.set(Press::default());
        let tapped = !done.changed && done.first == Some(done.from);
        if clearable && tapped && done.from > 0.0 {
            commit.call(0.0);
        }
    });

    let options = DragOptions {
        capture: root,
        onstart: Callback::new(move |event: DragStart| {
            if !editable {
                event.cancel.call(());
                return;
            }
            set_hover.call(None);
            press.set(Press {
                from: value,
                last: value,
                measuring: true,
                ..Press::default()
            });
            let first = *pressed.peek();
            pressed.set(None);
            if let Some(first) = first {
                pick.call(first);
            }
            // Started here, awaited in the task: Blitz locks the document while tasks drain.
            let rtl = root.is_rtl();
            let size = root.dimensions();
            let offset = root.client_offset();
            let start = event.client.x;
            spawn(async move {
                let measured = match (size.await, offset.await) {
                    (Ok(size), Ok((left, _))) if size.width > 0.0 => Some(Row {
                        left,
                        width: size.width,
                        rtl,
                    }),
                    _ => None,
                };
                row.set(measured);
                let mut waited = *press.peek();
                waited.measuring = false;
                press.set(waited);
                // A press no zone took picks by where it landed.
                if waited.first.is_none()
                    && let Some(next) = at.call(start)
                {
                    pick.call(next);
                }
                if let Some(next) = waited.moved_to.and_then(|x| at.call(x)) {
                    pick.call(next);
                }
                if waited.released {
                    finish.call(());
                }
            });
        }),
        onmove: Callback::new(move |event: DragMove| {
            let mut waiting = *press.peek();
            if waiting.measuring {
                waiting.moved_to = Some(event.client.x);
                press.set(waiting);
                return;
            }
            if let Some(next) = at.call(event.client.x) {
                pick.call(next);
            }
        }),
        onend: Callback::new(move |()| {
            let mut waiting = *press.peek();
            if waiting.measuring {
                waiting.released = true;
                press.set(waiting);
                return;
            }
            finish.call(());
        }),
    };
    // The whole row is the thumb: any sideways touch on it scrubs.
    let drag = use_sideways_drag(options, Callback::new(|()| true));
    (drag, pressed)
}

/// What the symbols draw from.
#[derive(Clone, Copy)]
struct Symbols {
    count: u8,
    fractions: u8,
    shown: f64,
    icon: SvgData,
    /// Hit zones per step, only on an editable rating.
    zones: bool,
    pressed: CopyValue<Option<f64>>,
    set_hover: Callback<Option<f64>>,
}

fn render_symbols(view: Symbols) -> Vec<Element> {
    let filled_attributes = || vec![Attribute::new("fill", "currentColor", None, false)];
    (0..view.count)
        .map(|index| {
            let filled = fill(view.shown, index);
            let fill_part = (filled > 0.0).then(|| {
                let width = filled * 100.0;
                rsx! {
                    span { "data-slot": RatingPart::Fill.slot(), style: "width: {width}%",
                        {draw_svg(&view.icon, filled_attributes())}
                    }
                }
            });
            let zones = view.zones.then(|| {
                let zones = (0..view.fractions).map(|zone| {
                    let at = zone_value(index, zone, view.fractions);
                    let (mut pressed, set_hover) = (view.pressed, view.set_hover);
                    rsx! {
                        span {
                            key: "{zone}",
                            "data-slot": "zone",
                            onpointerdown: move |_| pressed.set(Some(at)),
                            // Touch has no hover; its tap would leave one behind.
                            onpointermove: move |event: Event<PointerData>| {
                                if event.pointer_type() != "touch" {
                                    set_hover.call(Some(at));
                                }
                            },
                        }
                    }
                });
                rsx! {
                    span { "data-slot": "zones", {zones} }
                }
            });
            rsx! {
                span { key: "{index}", "data-slot": RatingPart::Symbol.slot(),
                    span { "data-slot": RatingPart::Glyph.slot(),
                        {draw_svg(&view.icon, Vec::new())}
                        {fill_part}
                    }
                    {zones}
                }
            }
        })
        .collect()
}
