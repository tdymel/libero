//! Below `sm`, an explicit `columns` shows only the months that fit side by side (todo 3958).

use dioxus::prelude::*;

use crate::{
    hooks::{ElementHandle, use_subscription_slot},
    platform::{ElementApi, ScrollSubscription, document, on_viewport_resize},
    utils::bump,
};

use super::styles::MONTHS_GAP;

/// How many `month` px wide months, `MONTHS_GAP` apart, fit in `room` px: at least 1, at most `columns`.
pub(super) fn months_that_fit(room: f64, month: f64, columns: i64) -> i64 {
    if month <= 0.0 {
        return columns;
    }
    // Half a px of slack for subpixel widths.
    let fit = ((room + MONTHS_GAP + 0.5) / (month + MONTHS_GAP)).floor() as i64;
    fit.clamp(1, columns.max(1))
}

/// The room for months: the root, capped by a constrained parent, and within the screen less
/// the root's start inset on both sides, for a parent that grows to its content.
pub(super) fn room_for(root: f64, left: f64, viewport: f64) -> f64 {
    root.min(viewport - 2.0 * left.max(0.0)).max(0.0)
}

/// The months a calendar shows, and whether this render only measures them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MonthFit {
    pub(super) columns: i64,
    /// All `columns` drawn, unseen, so the root fills its room and one month can be measured.
    pub(super) measuring: bool,
}

/// While `active`, measures the room ([`room_for`]) against one month, and again when the
/// viewport's width changes. Inactive, all `columns` show.
pub(super) fn use_month_fit(root: ElementHandle, columns: i64, active: bool) -> MonthFit {
    // The fit and the `columns` it was measured for.
    let mut fit = use_signal(|| None::<(i64, i64)>);
    let tick = use_signal(|| 0u64);
    let mut width = use_hook(|| CopyValue::new(None::<f64>));
    let resizing = use_subscription_slot::<dyn ScrollSubscription>();
    use_effect(use_reactive!(|active| {
        if !active {
            resizing.clear();
        } else if !resizing.is_some() {
            resizing.set(on_viewport_resize(Box::new(move || bump(tick))));
        }
    }));
    // A keyboard resizes only the height: a new width alone measures again.
    use_effect(move || {
        let _ = tick();
        let Some(document) = document() else {
            return;
        };
        let viewport = document.viewport();
        spawn(async move {
            let Ok(viewport) = viewport.await else {
                return;
            };
            if width.peek().is_some_and(|width| width != viewport.width) && fit.peek().is_some() {
                fit.set(None);
            }
            width.set(Some(viewport.width));
        });
    });
    let fitted = fit()
        .filter(|(of, _)| *of == columns)
        .map(|(_, shown)| shown);
    let measuring = active && fitted.is_none();
    use_effect(use_reactive!(|measuring, columns| {
        if !measuring {
            return;
        }
        let (size, offset) = (root.dimensions(), root.client_offset());
        let viewport = document().map(|document| document.viewport());
        let month = root
            .query_selector("[data-slot='months'] > [role='grid']")
            .map(|month| month.dimensions());
        spawn(async move {
            let room = async {
                let (size, (left, _)) = (size.await.ok()?, offset.await.ok()?);
                let viewport = viewport?.await.ok()?;
                Some(room_for(size.width, left, viewport.width))
            };
            let month = async { month.ok()?.await.ok() };
            // Unmeasurable, every month shows as before.
            let shown = match (room.await, month.await) {
                (Some(room), Some(month)) => months_that_fit(room, month.width, columns),
                _ => columns,
            };
            fit.set(Some((columns, shown)));
        });
    }));
    MonthFit {
        columns: match active {
            true => fitted.unwrap_or(columns),
            false => columns,
        },
        measuring,
    }
}

#[cfg(test)]
mod tests {
    use super::{months_that_fit, room_for};

    #[test]
    fn the_room_is_the_root_within_the_screen() {
        // A sheet's centred root, capped by the sheet.
        assert_eq!(room_for(308.0, 26.0, 360.0), 308.0);
        // Grown with its parent past the screen: the screen less the inset both sides.
        assert_eq!(room_for(632.0, 25.0, 360.0), 310.0);
        assert_eq!(room_for(632.0, -40.0, 360.0), 360.0);
    }

    #[test]
    fn as_many_months_show_as_fit_but_never_none() {
        // Two 308px months need 632px.
        assert_eq!(months_that_fit(632.0, 308.0, 2), 2);
        assert_eq!(months_that_fit(631.6, 308.0, 2), 2);
        assert_eq!(months_that_fit(631.0, 308.0, 2), 1);
        assert_eq!(months_that_fit(1000.0, 308.0, 3), 3);
        assert_eq!(months_that_fit(1000.0, 308.0, 2), 2);
        assert_eq!(months_that_fit(200.0, 308.0, 3), 1);
        assert_eq!(months_that_fit(200.0, 0.0, 3), 3);
    }
}
