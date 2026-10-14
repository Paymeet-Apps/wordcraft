//! Direction-aware replacements for the LTR-only egui idioms (`ui.horizontal`,
//! `Layout::right_to_left`, `Align2::LEFT_*`, `Grid` rows, absolute painter x).
//! The direction is `i18n::current().is_rtl()`, which is per thread and set every frame.

// Used by the mirroring tasks (7-13).
#![cfg_attr(not(test), allow(dead_code))]

use egui::{Align, Align2, InnerResponse, Key, Layout, Rect, Ui, vec2};

/// Is the interface drawn right to left?
pub fn is_rtl() -> bool {
    crate::i18n::current().is_rtl()
}

/// `ui.horizontal` that starts at the visual start edge (right under RTL). Allocates exactly
/// as `Ui::horizontal` does.
pub fn row<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let size = vec2(ui.available_size_before_wrap().x, ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(size, row_layout(Align::Center), add)
}

/// `ui.vertical` whose items line up on the start edge (the right under RTL). Inside a
/// right-to-left row a plain `ui.vertical` would land at the far left of the free space.
pub fn column<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    if is_rtl() { ui.scope_builder(egui::UiBuilder::new().layout(Layout::top_down(Align::Max)), add) } else { ui.vertical(add) }
}

/// Content that always lays out from the left of the free space (an egui `Grid` or `ScrollArea`),
/// placed at the cursor of a right-to-left row: it gets a box exactly as wide as it measured last
/// frame (a changed width discards the frame and lays it out again). English: `add(ui)` as is.
pub fn fitted<R>(ui: &mut Ui, id_salt: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    if !is_rtl() {
        return add(ui);
    }
    let id = ui.id().with(("rtl_fitted", id_salt));
    let known = ui.data(|d| d.get_temp::<f32>(id));
    let width = known.unwrap_or_else(|| ui.available_width());
    let r = ui.allocate_ui_with_layout(vec2(width, ui.available_height()), Layout::top_down(Align::Min), add);
    let got = r.response.rect.width();
    if known.is_none_or(|k| (k - got).abs() > 0.5) {
        ui.data_mut(|d| d.insert_temp(id, got));
        ui.ctx().request_discard("rtl: fitted width changed");
    }
    r.inner
}

/// The start-to-end row layout: left to right, or right to left under RTL.
/// Note: with_layout with Align::Center takes the full available height unless the caller caps it.
pub fn row_layout(align: Align) -> Layout {
    if is_rtl() { Layout::right_to_left(align) } else { Layout::left_to_right(align) }
}

/// The end-to-start row layout (pushes content to the end edge): the opposite of [`row_layout`].
pub fn end_layout(align: Align) -> Layout {
    if is_rtl() { Layout::left_to_right(align) } else { Layout::right_to_left(align) }
}

/// Painter text anchor at the start edge (`LEFT_*`, or `RIGHT_*` under RTL).
pub fn start_align2(v: Align) -> Align2 {
    Align2([if is_rtl() { Align::RIGHT } else { Align::LEFT }, v])
}

/// Painter text anchor at the end edge (`RIGHT_*`, or `LEFT_*` under RTL).
pub fn end_align2(v: Align) -> Align2 {
    Align2([if is_rtl() { Align::LEFT } else { Align::RIGHT }, v])
}

/// Mirror an absolute x inside `outer` under RTL.
pub fn x(outer: Rect, x: f32) -> f32 {
    if is_rtl() { outer.right() - (x - outer.left()) } else { x }
}

/// Mirror a rect inside `outer` under RTL.
pub fn rect(outer: Rect, r: Rect) -> Rect {
    if is_rtl() { Rect::from_min_max(egui::pos2(x(outer, r.max.x), r.min.y), egui::pos2(x(outer, r.min.x), r.max.y)) } else { r }
}

/// One `Grid` row: cells in reading order (reversed under RTL), each aligned to its column's
/// start edge, then `end_row`.
pub fn grid_row<const N: usize>(ui: &mut Ui, cells: [&mut dyn FnMut(&mut Ui); N]) {
    if is_rtl() {
        for cell in cells.into_iter().rev() {
            // Height-capped like `row`, or the cell would take the grid's whole remaining height.
            let size = vec2(ui.available_width(), ui.spacing().interact_size.y);
            ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| cell(ui));
        }
    } else {
        // English: exactly the plain `label; field; end_row()` it replaces.
        for cell in cells {
            cell(ui);
        }
    }
    ui.end_row();
}

/// Layout for menu items: text aligned to the start edge.
pub fn menu_layout() -> Layout {
    Layout::top_down_justified(if is_rtl() { Align::Max } else { Align::Min })
}

/// Swap left/right arrows under RTL, for code that reads them as previous/next.
pub fn arrow(key: Key) -> Key {
    if !is_rtl() {
        return key;
    }
    match key {
        Key::ArrowLeft => Key::ArrowRight,
        Key::ArrowRight => Key::ArrowLeft,
        other => other,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{self, Lang};
    use egui_kittest::kittest::Queryable;

    /// Run `f` with the interface language set, restoring English afterwards.
    fn with_lang<T>(code: &str, f: impl FnOnce() -> T) -> T {
        i18n::set_current(Lang::from_code(code).unwrap_or(Lang::EN));
        let out = f();
        i18n::set_current(Lang::EN);
        out
    }

    fn outer() -> Rect {
        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 10.0))
    }

    #[test]
    fn x_and_rect_mirror_inside_the_outer_rect() {
        let r = Rect::from_min_max(egui::pos2(10.0, 0.0), egui::pos2(30.0, 10.0));
        with_lang("ar", || {
            assert!(is_rtl());
            assert_eq!(x(outer(), 10.0), 90.0);
            assert_eq!(rect(outer(), r), Rect::from_min_max(egui::pos2(70.0, 0.0), egui::pos2(90.0, 10.0)));
        });
        assert!(!is_rtl());
        assert_eq!(x(outer(), 10.0), 10.0);
        assert_eq!(rect(outer(), r), r);
    }

    fn two_labels(code: &str) -> (f32, f32) {
        with_lang(code, || {
            let mut h = egui_kittest::Harness::new_ui(|ui| {
                row(ui, |ui| {
                    ui.label("A");
                    ui.label("B");
                });
            });
            h.run();
            (h.get_by_label("A").rect().center().x, h.get_by_label("B").rect().center().x)
        })
    }

    #[test]
    fn row_places_the_first_widget_at_the_right_in_rtl() {
        let (a, b) = two_labels("ar");
        assert!(a > b, "arabic: A ({a}) is right of B ({b})");
        let (a, b) = two_labels("en");
        assert!(a < b, "english: A ({a}) is left of B ({b})");
    }

    fn grid_cells(code: &str) -> (f32, f32) {
        with_lang(code, || {
            let mut h = egui_kittest::Harness::new_ui(|ui| {
                egui::Grid::new("g").show(ui, |ui| {
                    grid_row(
                        ui,
                        [
                            &mut |ui| {
                                ui.label("L");
                            },
                            &mut |ui| {
                                ui.label("F");
                            },
                        ],
                    );
                });
            });
            h.run();
            (h.get_by_label("L").rect().center().x, h.get_by_label("F").rect().center().x)
        })
    }

    #[test]
    fn grid_row_reverses_cells_in_rtl() {
        let (l, f) = grid_cells("ar");
        assert!(l > f, "arabic: L ({l}) is right of F ({f})");
        let (l, f) = grid_cells("en");
        assert!(l < f, "english: L ({l}) is left of F ({f})");
    }

    #[test]
    fn arrows_swap_only_in_rtl() {
        assert_eq!(arrow(Key::ArrowLeft), Key::ArrowLeft);
        assert_eq!(arrow(Key::ArrowRight), Key::ArrowRight);
        with_lang("ar", || {
            assert_eq!(arrow(Key::ArrowLeft), Key::ArrowRight);
            assert_eq!(arrow(Key::ArrowRight), Key::ArrowLeft);
            assert_eq!(arrow(Key::ArrowUp), Key::ArrowUp);
        });
    }

    #[test]
    fn layouts_and_alignments_follow_direction() {
        assert!(!row_layout(Align::Center).prefer_right_to_left());
        assert!(end_layout(Align::Center).prefer_right_to_left());
        assert_eq!(start_align2(Align::Min), Align2::LEFT_TOP);
        assert_eq!(end_align2(Align::Center), Align2::RIGHT_CENTER);
        assert_eq!(menu_layout().cross_align, Align::Min);
        with_lang("ar", || {
            assert!(row_layout(Align::Center).prefer_right_to_left());
            assert!(!end_layout(Align::Center).prefer_right_to_left());
            assert_eq!(start_align2(Align::Min), Align2::RIGHT_TOP);
            assert_eq!(end_align2(Align::Center), Align2::LEFT_CENTER);
            assert_eq!(menu_layout().cross_align, Align::Max);
        });
    }

    /// (label y, grid height) for a grid row built by `grid_row` or by plain cells.
    fn grid_metrics(code: &str, helper: bool) -> (f32, f32) {
        use std::{cell::Cell, rc::Rc};
        let height = Rc::new(Cell::new(0.0_f32));
        let h2 = height.clone();
        let y = with_lang(code, || {
            let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(300.0, 400.0)).build_ui(move |ui| {
                let r = egui::Grid::new("g").show(ui, |ui| {
                    if helper {
                        grid_row(
                            ui,
                            [
                                &mut |ui| {
                                    ui.label("L");
                                },
                                &mut |ui| {
                                    ui.label("F");
                                },
                            ],
                        );
                    } else {
                        ui.label("L");
                        ui.label("F");
                        ui.end_row();
                    }
                });
                h2.set(r.response.rect.height());
            });
            h.run();
            h.get_by_label("L").rect().center().y
        });
        (y, height.get())
    }

    #[test]
    fn grid_row_keeps_english_rows_unchanged_and_rtl_rows_the_same_height() {
        let plain = grid_metrics("en", false);
        assert_eq!(grid_metrics("en", true), plain, "english: same y and grid height");
        let (y, height) = grid_metrics("ar", true);
        assert!((y - plain.0).abs() <= 1.0, "arabic y {y} vs english {}", plain.0);
        assert!((height - plain.1).abs() <= 1.0, "arabic height {height} vs {}", plain.1);
    }
}
