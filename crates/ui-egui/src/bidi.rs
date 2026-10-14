//! Bidirectional text for the interface (L10N-AR). Currently only the spike that decides how.
//!
//! **Verdict: Strategy A.** epaint 0.36.2 shapes and joins an Arabic run correctly, but it has no
//! bidi algorithm, so *runs* must be reordered by us and each run's characters left in logical
//! order. Evidence, from `epaint-0.36.2/src/text/`:
//! - `text_layout.rs::shape_text` pushes the run into a `harfrust::UnicodeBuffer` and calls
//!   `guess_segment_properties()`: direction and script come from the run's first strong
//!   character. An Arabic run is therefore shaped right-to-left with joining forms and lam-alef
//!   ligatures, and harfrust returns its glyphs in visual order. `layout_shaped_run` advances the
//!   cursor along that order, so `x` grows to the right and the first character sits rightmost.
//! - Runs are cut by *font face*, not by script (`segment_into_runs`; its doc says "would need
//!   script-aware splitting once RTL/bidi support is added", and `font.rs` has "TODO: heed bidi
//!   characters"). Runs are placed left to right in logical order, with no reordering.
//!   Consequences, observed by the probes below with Inter first and an Arabic face as fallback:
//!   * Neutrals (space, hyphen, brackets) are Inter glyphs, so they form their own runs rather
//!     than joining the neighbouring Arabic run. They keep their logical position.
//!   * Brackets are never mirrored (no `Bidi_Mirrored` handling anywhere in epaint).
//!   * Within an Arabic run, `Row::glyphs` holds the visual glyphs followed by zero-width
//!     "continuation" glyphs (`emit_continuation_glyphs`) at the run's end, so the `chr` order of
//!     a row is not the logical order. Caret and selection inside Arabic text are therefore
//!     approximate (already a non-goal for `TextEdit`).
//!   * A mixed string such as `حفظ PDF` draws the Arabic first (left) and `PDF` after it (right):
//!     plain logical order, which is wrong for an RTL base direction.
//!
//!   So Task 2 splits a string into level runs with `unicode-bidi`, mirrors paired brackets and
//!   reorders the runs (visual order), leaving the characters inside every run as they are. The
//!   probe tests pin this so an egui upgrade that changes it fails loudly. This assumes Arabic
//!   and Latin resolve to different faces; if one face covered both, a mixed run would be one
//!   buffer with a single guessed direction, so the reordering must also split runs at script
//!   changes (it does, being driven by bidi levels).

#[cfg(test)]
mod tests {
    use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, FontId, RawInput};
    use std::sync::Arc;

    /// A glyph's rectangle in the font atlas: equal for equal glyph images, wherever drawn.
    type Id = [u16; 4];

    /// An installed Arabic-capable face, if any.
    fn arabic_face() -> Option<Arc<wordcraft_fonts::FontFace>> {
        let db = wordcraft_fonts::FontDb::global();
        db.load_system_fonts();
        for family in ["Noto Sans Arabic UI", "Noto Sans Arabic", "Segoe UI", "Tahoma", "Arial", "DejaVu Sans"] {
            let face = db.face(family, "Regular");
            if face.covers('م') && face.glyph_for('ف') != 0 {
                return Some(face);
            }
        }
        None
    }

    /// A context with the interface fonts (Inter first) and the Arabic face as a fallback, as the
    /// application installs fonts (`theme::add_fallback_faces`).
    fn context() -> Option<Context> {
        let Some(face) = arabic_face() else {
            eprintln!("skipped: no Arabic font");
            return None;
        };
        eprintln!("Arabic face: {} {}", face.family, face.style);
        let mut defs: FontDefinitions = crate::theme::font_definitions(false, false);
        let mut data = FontData::from_owned(face.data().to_vec());
        data.index = face.index();
        defs.font_data.insert("probe-arabic".into(), Arc::new(data));
        if let Some(v) = defs.families.get_mut(&FontFamily::Proportional) {
            v.push("probe-arabic".into());
        }
        let ctx = Context::default();
        ctx.set_fonts(defs);
        // Fonts are applied at the start of a pass.
        ctx.run_ui(RawInput::default(), |_| {}).textures_delta.clear();
        Some(ctx)
    }

    /// `(char, x, glyph id)` of the first row, in the order epaint stores them.
    fn glyphs(ctx: &Context, text: &str) -> Vec<(char, f32, Id)> {
        let galley = ctx.fonts_mut(|f| f.layout_no_wrap(text.into(), FontId::proportional(14.0), Color32::BLACK));
        let row = galley.rows.first().unwrap();
        row.glyphs.iter().map(|g| (g.chr, g.pos.x, [g.uv_rect.min[0], g.uv_rect.min[1], g.uv_rect.max[0], g.uv_rect.max[1]])).collect()
    }

    fn x_of(g: &[(char, f32, Id)], c: char) -> f32 {
        g.iter().find(|t| t.0 == c).unwrap_or_else(|| panic!("no glyph for {c}: {g:?}")).1
    }

    #[test]
    fn epaint_shapes_an_arabic_run_right_to_left() {
        let Some(ctx) = context() else { return };
        let g = glyphs(&ctx, "ملف");
        eprintln!("ملف: {g:?}");
        // The first letter is the rightmost: glyphs come out in visual order.
        assert!(x_of(&g, 'م') > x_of(&g, 'ف'), "{g:?}");
        assert!(x_of(&g, 'م') > x_of(&g, 'ل') && x_of(&g, 'ل') > x_of(&g, 'ف'), "{g:?}");
    }

    #[test]
    fn epaint_joins_arabic_letters() {
        let Some(ctx) = context() else { return };
        let joined = glyphs(&ctx, "ملف");
        let isolated: Vec<Id> = ["م", "ل", "ف"].iter().map(|s| glyphs(&ctx, s)[0].2).collect();
        let ids = |c: char| joined.iter().find(|t| t.0 == c).map(|t| t.2);
        eprintln!("joined {joined:?} isolated {isolated:?}");
        // Initial meem, medial lam and final feh are different glyphs from the isolated forms.
        assert_ne!(ids('م'), Some(isolated[0]));
        assert_ne!(ids('ل'), Some(isolated[1]));
        assert_ne!(ids('ف'), Some(isolated[2]));
    }

    #[test]
    fn epaint_keeps_mixed_runs_in_logical_order() {
        let Some(ctx) = context() else { return };
        let g = glyphs(&ctx, "حفظ PDF");
        eprintln!("حفظ PDF: {g:?}");
        let arabic_max = ['ح', 'ف', 'ظ'].iter().map(|&c| x_of(&g, c)).fold(f32::MIN, f32::max);
        // Logical order, left to right: no base-direction reordering, so `PDF` is to the right.
        assert!(x_of(&g, 'P') > arabic_max, "P sits right of the Arabic: {g:?}");
        assert!(x_of(&g, 'P') < x_of(&g, 'D') && x_of(&g, 'D') < x_of(&g, 'F'), "{g:?}");
    }

    #[test]
    fn epaint_mirrors_parentheses_in_rtl_runs() {
        let Some(ctx) = context() else { return };
        let g = glyphs(&ctx, "(نص)");
        eprintln!("(نص): {g:?}");
        // Brackets come from the Latin face as runs of their own and are never mirrored: the
        // opening bracket stays leftmost.
        assert!(x_of(&g, '(') < x_of(&g, 'ن') && x_of(&g, 'ن') != x_of(&g, ')'));
        assert!(x_of(&g, '(') < x_of(&g, ')'), "{g:?}");
        let leftmost = g.iter().min_by(|a, b| a.1.total_cmp(&b.1)).map(|t| t.0);
        assert_eq!(leftmost, Some('('));
    }

    #[test]
    fn epaint_groups_neutrals() {
        let Some(ctx) = context() else { return };
        let g = glyphs(&ctx, "أ - ب");
        eprintln!("أ - ب: {g:?}");
        // Logical order, left to right: أ, space, hyphen, space, ب. The neutrals are drawn by the
        // Latin face, so they are separate runs and do not join the Arabic ones.
        let xs: Vec<f32> = g.iter().map(|t| t.1).collect();
        assert!(xs.windows(2).all(|w| w[0] <= w[1]), "strictly logical order: {g:?}");
        assert_eq!(g.iter().map(|t| t.0).collect::<String>(), "أ - ب");
    }
}
