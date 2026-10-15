//! File tab (Backstage): Home/New, Open, Info, Save As, Print, Export, Close, Options, About.

use egui::{Align, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use crate::rtl::HoverText as _;
use crate::theme::{APP_COLOR, Tokens, medium, regular, semibold};
use crate::{WordApp, icons, rtl};

const PAGES: [(&str, &str); 10] = [
    ("home", "Home"),
    ("new", "New"),
    ("open", "Open"),
    ("info", "Info"),
    ("save", "Save"),
    ("saveAs", "Save As"),
    ("print", "Print"),
    ("export", "Export"),
    ("close", "Close"),
    ("options", "Options"),
];

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // No document open (#1129): only the pages that open or create one, and Options.
    let has_document = app.has_document();
    if !has_document && !works_without_document(&app.ui.backstage_page) {
        app.ui.backstage_page = "home".into();
    }
    let mac = app.integrated_titlebar;
    // The navigation column is at the window's start edge: the right under RTL.
    let nav = if rtl::is_rtl() { egui::Panel::right("backstage_nav") } else { egui::Panel::left("backstage_nav") };
    // The back arrow points towards the start edge.
    let back_icon = if rtl::is_rtl() { "chevronRight" } else { "chevronLeft" };
    nav.exact_size(200.0)
        // Fixed width, as in Word: without this egui shows a resize handle that does nothing (#502).
        .resizable(false)
        .frame(egui::Frame::NONE.fill(APP_COLOR).inner_margin(egui::Margin { left: 0, right: 0, top: if mac { 0 } else { 12 }, bottom: 12 }))
        .show(ui, |ui| {
            let back = if mac {
                // macOS (#222): the Backstage covers the title bar, so its top band is the title
                // bar: the back button sits beside the traffic lights, centred with them and as far
                // from them as the title bar's own controls, and the rest of the band drags the window.
                let (band, drag) = ui.allocate_exact_size(vec2(200.0, crate::chrome::MAC_TITLE_BAR), Sense::click_and_drag());
                if drag.drag_started() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                // The chevron's stroke starts at the content edge.
                // RTL: the column is at the right, away from the lights; the arrow keeps the same
                // distance from the column's right edge.
                let c = pos2(rtl::x(band, band.min.x + crate::chrome::MAC_CONTENT_LEFT + 3.0), band.center().y);
                let resp = ui.interact(Rect::from_center_size(c, vec2(22.0, band.height())), ui.id().with("backstage_back"), Sense::click());
                icons::paint(ui.painter(), Rect::from_center_size(c, vec2(18.0, 18.0)), back_icon, egui::Color32::WHITE, egui::Color32::WHITE);
                resp
            } else {
                let (r, resp) = ui.allocate_exact_size(vec2(200.0, 40.0), Sense::click());
                icons::paint(
                    ui.painter(),
                    Rect::from_center_size(pos2(rtl::x(r, r.min.x + 26.0), r.center().y), vec2(18.0, 18.0)),
                    back_icon,
                    egui::Color32::WHITE,
                    egui::Color32::WHITE,
                );
                resp
            };
            // Escape in the built-in file browser closes only the browser.
            if back.clicked() || (!app.file_dialog_open() && ui.input(|i| i.key_pressed(egui::Key::Escape))) {
                app.ui.backstage = false;
                app.canvas.want_focus = true;
            }
            ui.add_space(if mac { 12.0 } else { 6.0 });
            for (id, label) in PAGES {
                // No document open (#1129): Info, Save, Save As, Print and Export are greyed out.
                let on = has_document || works_without_document(id);
                let (r, resp) = ui.add_enabled_ui(on, |ui| ui.allocate_exact_size(vec2(200.0, 38.0), Sense::click())).inner;
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, on, tl!(label)));
                let active = app.ui.backstage_page == id;
                if active {
                    ui.painter().rect_filled(r, 0.0, egui::Color32::from_white_alpha(46));
                } else if on && resp.hovered() {
                    ui.painter().rect_filled(r, 0.0, egui::Color32::from_white_alpha(26));
                }
                ui.painter().text(
                    pos2(rtl::x(r, r.min.x + 22.0), r.center().y),
                    rtl::start_align2(Align::Center),
                    tl!(label),
                    if active { semibold(14.0) } else { medium(14.0) },
                    if on { egui::Color32::WHITE } else { egui::Color32::from_white_alpha(110) },
                );
                if resp.clicked() {
                    match id {
                        // A document without a file goes to the Save As page (as in Word) instead
                        // of `file.save`, which would also open the native Save As dialog.
                        // In the browser a save is a download, so it saves right away.
                        "save" if app.session.path.is_none() && app.services.download.is_none() => {
                            app.ui.backstage_page = "saveAs".into();
                        }
                        "save" => {
                            let r = app.run("file.save", json!({}));
                            if r.is_ok_and(|v| v.get("saved").and_then(serde_json::Value::as_bool) == Some(true) || v.get("downloaded").is_some()) {
                                app.ui.backstage = false;
                                app.canvas.want_focus = true;
                            } else {
                                app.ui.backstage_page = "saveAs".into();
                            }
                        }
                        "saveAs" => {
                            app.ui.backstage_page = "saveAs".into();
                        }
                        // Not a page: `file.close` asks Save / Don't Save / Cancel first when there
                        // are unsaved changes, and Cancel leaves the Backstage where it was.
                        "close" => {
                            let _ = app.run("file.close", json!({}));
                        }
                        _ => app.ui.backstage_page = id.into(),
                    }
                }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                for (label, id) in [("About", "about"), ("Discord community", "discord")] {
                    let (r, resp) = ui.allocate_exact_size(vec2(200.0, 32.0), Sense::click());
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tl!(label)));
                    if resp.hovered() {
                        ui.painter().rect_filled(r, 0.0, egui::Color32::from_white_alpha(26));
                    }
                    ui.painter().text(
                        pos2(rtl::x(r, r.min.x + 22.0), r.center().y),
                        rtl::start_align2(Align::Center),
                        tl!(label),
                        regular(13.0),
                        egui::Color32::WHITE,
                    );
                    if resp.clicked() {
                        if id == "discord" {
                            let _ = app.run("ui.discord", json!({}));
                        } else {
                            app.dialog = crate::dialogs::Dialog::open("about", app);
                        }
                    }
                }
            });
        });
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(40, 30))).show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let page = |app: &mut WordApp, ui: &mut Ui| match app.ui.backstage_page.as_str() {
                "home" => home_page(app, ui),
                "new" => new_page(app, ui),
                "open" => open_page(app, ui),
                "saveAs" => save_as_page(app, ui),
                "info" => info_page(app, ui),
                "print" | "export" => export_page(app, ui),
                "options" => crate::options::page(app, ui),
                _ => home_page(app, ui),
            };
            if rtl::is_rtl() {
                // Headings, lists and buttons start at the right.
                ui.set_width(ui.available_width());
                rtl::column(ui, |ui| page(app, ui));
            } else {
                page(app, ui);
            }
        });
    });
}

/// The Backstage pages that work while no document is open (#1129).
fn works_without_document(page: &str) -> bool {
    matches!(page, "home" | "new" | "open" | "options")
}

fn heading(ui: &mut Ui, s: &str) {
    ui.label(egui::RichText::new(tl!(s)).font(semibold(26.0)));
    ui.add_space(16.0);
}

fn template_tile(ui: &mut Ui, app: &mut WordApp, label: &str, template: &str) {
    let t = Tokens::get(ui.ctx());
    rtl::column(ui, |ui| {
        // The label sits right under the thumbnail whatever row gap the gallery uses.
        ui.spacing_mut().item_spacing.y = 0.0;
        let (r, resp) = ui.allocate_exact_size(vec2(150.0, 194.0), Sense::click());
        ui.painter().rect(
            r,
            2.0,
            egui::Color32::WHITE,
            Stroke::new(if resp.hovered() { 2.0 } else { 1.0 }, if resp.hovered() { t.accent } else { t.border_strong }),
            egui::StrokeKind::Inside,
        );
        let ppp = ui.ctx().pixels_per_point();
        let key = format!("tpl:{template}:{ppp}");
        let region = app.session.region();
        let tex = ui.ctx().data(|d| d.get_temp::<egui::TextureHandle>(egui::Id::new(&key))).or_else(|| {
            let mut s = wordcraft_engine::Session::new(wordcraft_doc::Document::new());
            // The blank tile shows the region's paper (#838).
            s.system_region = region;
            let _ = s.run("file.new", &json!({"template": template}));
            let l = s.export_layout();
            let page = l.pages.first()?;
            let img = wordcraft_render::render_page(&s.doc, page, 150.0 / page.w * ppp, &crate::canvas::screen_render_options());
            let px = img.to_straight();
            let h = ui.ctx().load_texture(
                &key,
                egui::ColorImage::from_rgba_unmultiplied([img.width as usize, img.height as usize], &px),
                egui::TextureOptions::LINEAR,
            );
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(&key), h.clone()));
            Some(h)
        });
        if let Some(h) = tex {
            ui.painter().image(h.id(), r.shrink(1.0), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
        }
        // The caption opens the template too, as in Word.
        let caption = ui.add(egui::Label::new(egui::RichText::new(tl!(label)).font(regular(12.5))).sense(Sense::click()));
        if resp.clicked() || caption.clicked() {
            let _ = app.run("file.new", json!({"template": template}));
            app.ui.backstage = false;
            app.canvas.want_focus = true;
        }
    });
}

/// The built-in templates in gallery order: display label and the `file.new` template id.
const TEMPLATES: [(&str, &str); 5] =
    [("Blank document", "blank"), ("Studio handbook (sample)", "sample"), ("Letter", "letter"), ("Résumé", "resume"), ("Report", "report")];

/// How many templates Home shows before "More templates" (the rest are on New).
const HOME_TEMPLATES: usize = 4;

/// The templates whose English or translated label, or id, contains `query` (case-insensitive;
/// the id lets "resume" find "Résumé"); all of them for an empty query.
fn matching_templates(query: &str) -> Vec<(&'static str, &'static str)> {
    let q = query.trim().to_lowercase();
    TEMPLATES
        .into_iter()
        .filter(|(label, id)| q.is_empty() || id.contains(&q) || label.to_lowercase().contains(&q) || tl!(label).to_lowercase().contains(&q))
        .collect()
}

/// Home: a greeting, Blank plus a short row of templates with a link to New, then Recent.
fn home_page(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let hour = wordcraft_engine::cmd::now_iso().get(11..13).and_then(|h| h.parse::<u32>().ok()).unwrap_or(9);
    heading(
        ui,
        if hour < 12 {
            "Good morning"
        } else if hour < 18 {
            "Good afternoon"
        } else {
            "Good evening"
        },
    );
    rtl::row(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(18.0, 0.0);
        for (label, template) in TEMPLATES.into_iter().take(HOME_TEMPLATES) {
            template_tile(ui, app, label, template);
        }
    });
    ui.add_space(8.0);
    let more = ui.add(egui::Label::new(egui::RichText::new(tl!("More templates →")).font(medium(13.5)).color(t.accent)).sense(Sense::click()));
    if more.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        app.ui.backstage_page = "new".into();
    }
    ui.add_space(24.0);
    open_list(app, ui);
}

/// New: the whole template gallery with a search box; no Recent list (that is on Home and Open).
fn new_page(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    heading(ui, "New");
    let id = egui::Id::new("backstage_template_search");
    let mut query = ui.ctx().data(|d| d.get_temp::<String>(id)).unwrap_or_default();
    let search = ui.add(egui::TextEdit::singleline(&mut query).hint_text(tl!("Search for templates")).desired_width(360.0));
    if search.changed() {
        ui.ctx().data_mut(|d| d.insert_temp(id, query.clone()));
    }
    ui.add_space(18.0);
    let found = matching_templates(&query);
    if found.is_empty() {
        ui.label(egui::RichText::new(tl!("No templates match your search.")).color(t.text_dim));
        return;
    }
    let gallery = |ui: &mut Ui| {
        ui.spacing_mut().item_spacing = vec2(18.0, 18.0);
        for (label, template) in found {
            template_tile(ui, app, label, template);
        }
    };
    if rtl::is_rtl() {
        // Rows of tiles from the right, wrapping as in English.
        ui.with_layout(egui::Layout::right_to_left(Align::Min).with_main_wrap(true), gallery);
    } else {
        ui.horizontal_wrapped(gallery);
    }
}

fn open_list(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new(tl!("Recent")).font(semibold(16.0)));
    ui.add_space(6.0);
    if app.ui.recent.is_empty() {
        ui.label(egui::RichText::new(tl!("Documents you open will show up here.")).color(t.text_dim));
    }
    for p in app.ui.recent.iter().take(app.ui.recent_shown()).cloned().collect::<Vec<_>>() {
        let name = std::path::Path::new(&p).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(p.clone());
        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width().min(700.0), 40.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r, 4.0, t.hover);
        }
        icons::paint(
            ui.painter(),
            Rect::from_center_size(pos2(rtl::x(r, r.min.x + 18.0), r.center().y), vec2(22.0, 22.0)),
            "printLayout",
            t.icon,
            t.accent,
        );
        let start = rtl::start_align2(Align::Center);
        ui.painter().text(pos2(rtl::x(r, r.min.x + 40.0), r.min.y + 13.0), start, &name, medium(13.0), t.text);
        ui.painter().text(pos2(rtl::x(r, r.min.x + 40.0), r.min.y + 29.0), start, &p, regular(11.0), t.text_dim);
        if resp.clicked() {
            let _ = app.run("file.open", json!({"path": p}));
            app.ui.backstage = false;
            app.canvas.want_focus = true;
        }
    }
}

fn open_page(app: &mut WordApp, ui: &mut Ui) {
    heading(ui, "Open");
    if ui.button(egui::RichText::new(tl!("Browse…")).font(medium(14.0))).clicked() {
        let _ = app.run("ui.openFileDialog", json!({}));
    }
    // Linux and the BSDs (#576): the system dialog needs a file chooser portal or zenity; when it
    // doesn't show up, WordCraft's own browser is always one click away.
    #[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
    if ui
        .button(egui::RichText::new(tl!("Browse in WordCraft…")).font(medium(14.0)))
        .rtl_hover_text(tl!("Open a file with WordCraft's own browser when the system file dialog doesn't appear."))
        .clicked()
    {
        let _ = app.run("ui.fileBrowser", json!({}));
    }
    ui.add_space(18.0);
    open_list(app, ui);
}

/// Save As: Browse… asks where to save. It has no recent-documents list: clicking one of those
/// opens it, which is not what Save As is for.
fn save_as_page(app: &mut WordApp, ui: &mut Ui) {
    heading(ui, "Save As");
    if ui.button(egui::RichText::new(tl!("Browse…")).font(medium(14.0))).clicked() {
        app.save_as_dialog();
    }
}

fn info_page(app: &mut WordApp, ui: &mut Ui) {
    heading(ui, "Info");
    let info = app.session.run("file.info", &json!({})).unwrap_or_default();
    // RTL: Properties in the right-hand column, every column's contents starting at the right.
    let (first, second) = if rtl::is_rtl() { (1, 0) } else { (0, 1) };
    ui.columns(2, |cols| {
        if let Some(ui) = cols.get_mut(first) {
            if rtl::is_rtl() {
                rtl::column(ui, |ui| properties_column(app, ui, &info));
            } else {
                properties_column(app, ui, &info);
            }
        }
        if let Some(ui) = cols.get_mut(second) {
            if rtl::is_rtl() {
                rtl::column(ui, |ui| statistics_column(ui, &info));
            } else {
                statistics_column(ui, &info);
            }
        }
    });
}

/// File › Info's left-hand column (the right-hand one under RTL): the document properties and
/// Protect Document.
fn properties_column(app: &mut WordApp, ui: &mut Ui, info: &serde_json::Value) {
    ui.label(egui::RichText::new(tl!("Properties")).font(semibold(15.0)));
    // The fields edit a copy of the document's properties that is read afresh every frame,
    // so each change goes into the document as it is typed (#260): waiting for the field to
    // lose focus dropped the keystrokes, and leaving File with Escape never loses it at all.
    // The keystrokes of one visit to a field are a single undo step.
    let mut props = app.session.doc.core.clone();
    let mut edited = None;
    let mut focused = None;
    // RTL: the grid in a box of its own width at the right (egui's Grid lays out from the left).
    rtl::fitted(ui, "props", |ui| {
        crate::rtl::grid("props", 2).spacing(vec2(12.0, 6.0)).show(ui, |ui| {
            for (l, v) in [
                ("Title", &mut props.title),
                ("Subject", &mut props.subject),
                ("Author", &mut props.creator),
                ("Keywords", &mut props.keywords),
                ("Category", &mut props.category),
            ] {
                let (mut label, mut field) = (None, None);
                rtl::grid_row(
                    ui,
                    [&mut |ui: &mut Ui| label = Some(ui.label(tl!(l))), &mut |ui: &mut Ui| {
                        let mut edit = egui::TextEdit::singleline(v).id_salt(("info-prop", l));
                        if rtl::is_rtl() {
                            // The right-to-left cell is only as wide as the column was last frame; give
                            // the field the width it has in English.
                            let w = ui.spacing().text_edit_width;
                            ui.set_min_width(w);
                            edit = edit.desired_width(w);
                        }
                        field = Some(ui.add(edit));
                    }],
                );
                let (Some(label), Some(r)) = (label, field) else { continue };
                let r = r.labelled_by(label.id);
                if r.changed() {
                    edited = Some(l);
                }
                if r.has_focus() {
                    focused = Some(l);
                }
            }
        })
    });
    if let Some(field) = edited {
        if app.info_editing == Some((field, app.session.rev())) {
            app.session.join_next_undo();
        }
        let ok = app.run("file.properties", json!({"title": props.title, "subject": props.subject, "author": props.creator, "keywords": props.keywords, "category": props.category})).is_ok();
        app.info_editing = ok.then(|| (field, app.session.rev()));
    }
    if focused != app.info_editing.map(|(f, _)| f) {
        app.info_editing = None;
    }
    // Protect Document › Encrypt with Password (#55).
    ui.add_space(18.0);
    ui.label(egui::RichText::new(tl!("Protect Document")).font(semibold(15.0)));
    let encrypted = info.get("encrypted").and_then(|v| v.as_bool()).unwrap_or(false);
    let note = if encrypted {
        tl!("A password is required to open this document.")
    } else {
        tl!("Anyone can open this document. Encrypt it with a password to keep it private.")
    };
    rtl::wrapped_label(ui, note, |s| egui::RichText::new(s));
    rtl::row(ui, |ui| {
        let label = if encrypted { tl!("Change Password…") } else { tl!("Encrypt with Password…") };
        if ui.button(egui::RichText::new(label).font(medium(13.5))).clicked() {
            let _ = app.run("file.encrypt", json!({}));
        }
        if encrypted && ui.button(egui::RichText::new(tl!("Remove Password")).font(medium(13.5))).clicked() {
            let _ = app.run("file.encrypt", json!({"password": null}));
        }
    });
}

/// "Pages: 2": a translated caption and its number, through the catalog's `{label}: {value}`, so
/// under RTL the caption reads first (drawn right of the number).
fn stat_line(label: &str, value: &str) -> String {
    crate::i18n::fmt_translated(tl!("{label}: {value}"), &[("label", label), ("value", value)])
}

/// File › Info's statistics column.
fn statistics_column(ui: &mut Ui, info: &serde_json::Value) {
    ui.label(egui::RichText::new(tl!("Statistics")).font(semibold(15.0)));
    for (l, k) in [("Pages", "pages"), ("Words", "words"), ("Paragraphs", "paragraphs"), ("Sections", "sections"), ("Comments", "comments")] {
        ui.label(stat_line(tl!(l), &info.get(k).map(|v| v.to_string()).unwrap_or_default()));
    }
    // A file path is converted as it is inserted; "Not saved yet" is already in draw order.
    ui.label(match info.get("path").and_then(|v| v.as_str()) {
        Some(path) => crate::i18n::fmt(tl!("Location: {path}"), &[("path", path)]),
        None => crate::i18n::fmt_translated(tl!("Location: {path}"), &[("path", tl!("Not saved yet"))]),
    });
}

fn export_page(app: &mut WordApp, ui: &mut Ui) {
    if app.ui.backstage_page == "print" {
        heading(ui, "Print");
        // Print Markup (a print setting, as in Word): the comments beside each page.
        let mut markup = app.session.prefs.print_markup;
        if rtl::checkbox(ui, &mut markup, tl!("Print Markup"))
            .rtl_hover_text(tl!(
                "Comments beside each page and tracked changes in PDFs, prints and images; a comment list at the end of web pages, Markdown and text."
            ))
            .changed()
        {
            let _ = app.run("file.printMarkup", json!({"value": markup}));
        }
        ui.add_space(12.0);
        crate::print_page::show(app, ui);
        return;
    }
    heading(ui, "Export");
    rtl::wrapped_label(ui, tl!("Save a copy in another format. Printing goes through a PDF you can print from any viewer."), |s| {
        egui::RichText::new(s)
    });
    ui.add_space(12.0);
    crate::print_page::export_formats(app, ui);
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;
    use serde_json::json;

    use super::matching_templates;
    use crate::{Services, WordApp};

    #[test]
    fn template_search_matches_labels_case_insensitively() {
        let ids = |q: &str| matching_templates(q).into_iter().map(|(_, id)| id).collect::<Vec<_>>();
        assert_eq!(ids(""), ["blank", "sample", "letter", "resume", "report"]);
        assert_eq!(ids("  LETTER "), ["letter"]);
        assert_eq!(ids("re"), ["resume", "report"]);
        assert_eq!(ids("résumé"), ["resume"]);
        assert!(ids("no such template").is_empty());
    }

    /// Home and New are different pages (#141): Home has the short template row, the link to New
    /// and Recent; New has every template and no Recent list.
    #[test]
    fn home_and_new_show_different_sections() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        assert!(h.query_by_label("Recent").is_some(), "Home lists recent documents");
        assert!(h.query_by_label("Letter").is_some(), "Home shows the first templates");
        assert!(h.query_by_label("Report").is_none(), "Home shows only a short row of templates");
        h.get_by_label("More templates →").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.backstage_page, "new", "the link opens New");
        assert!(h.query_by_label("Report").is_some(), "New shows every template");
        assert!(h.query_by_label("Recent").is_none(), "New has no Recent list");
    }

    /// #260: File › Info's Title and Author fields dropped every keystroke (they edited a copy
    /// re-read each frame and only wrote it back when the field lost focus), so a save had no
    /// title or author. Typing now lands in the document as it happens, one undo step per field,
    /// and the browser's Save downloads it with the save stamps advanced (#262).
    #[test]
    fn info_fields_keep_typed_properties_through_a_web_save() {
        use egui::accesskit::Role;
        let got: std::rc::Rc<std::cell::RefCell<Vec<(String, Vec<u8>)>>> = Default::default();
        let sink = got.clone();
        let services = Services {
            download: Some(Box::new(move |n: &str, b: &[u8]| {
                sink.borrow_mut().push((n.to_string(), b.to_vec()));
                Ok(())
            })),
            ..Default::default()
        };
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::from_text("XYZ")), services);
        a.session.author = "GAMMA".into();
        a.run("ui.backstage", json!({"value": true, "page": "info"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        for (field, text) in [("Title", "ALPHA"), ("Author", "BETA")] {
            h.get_by_role_and_label(Role::TextInput, field).click();
            h.run_steps(2);
            for c in text.chars() {
                h.get_by_role_and_label(Role::TextInput, field).type_text(&c.to_string());
                h.run_steps(2);
            }
            assert_eq!(h.get_by_role_and_label(Role::TextInput, field).value().as_deref(), Some(text), "{field} shows what was typed");
        }
        let core = h.state().session.doc.core.clone();
        assert_eq!((core.title.as_str(), core.creator.as_str()), ("ALPHA", "BETA"));
        assert!(h.state().session.dirty, "the property edits are unsaved changes");

        // Save (web: a download) writes them into docProps/core.xml, with fresh save stamps.
        let a = h.state_mut();
        let created = a.session.doc.core.created.clone();
        a.run("file.save", json!({})).unwrap();
        a.run("file.save", json!({})).unwrap();
        let (name, bytes) = got.borrow_mut().pop().unwrap();
        let back = wordcraft_engine::io::open_bytes(&name, &bytes).unwrap();
        assert_eq!((back.core.title.as_str(), back.core.creator.as_str()), ("ALPHA", "BETA"));
        assert_eq!(back.core.last_modified_by, "GAMMA");
        assert_eq!(back.core.revision, core.revision + 2, "each save advances the revision");
        assert!(created.is_empty() && !back.core.created.is_empty(), "the first save sets the creation time");
        assert!(!back.core.modified.is_empty());
        assert!(!a.session.dirty);

        // Each field's typing is one undo step.
        a.session.run("edit.undo", &json!({})).unwrap();
        assert_eq!((a.session.doc.core.title.as_str(), a.session.doc.core.creator.as_str()), ("ALPHA", ""));
        a.session.run("edit.undo", &json!({})).unwrap();
        assert_eq!(a.session.doc.core.title, "");
    }

    #[test]
    fn save_as_page_shows_browse_and_no_open_list() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "saveAs"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        assert!(h.query_by_label("Browse…").is_some(), "Save As has Browse button");
        assert!(h.query_by_label("Recent").is_none(), "Save As has no list that opens documents");
    }

    #[test]
    fn backstage_save_navigates_to_save_as_when_unnamed() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Save").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.backstage_page, "saveAs", "unnamed Save opens Save As page");
        assert!(h.state().ui.backstage, "backstage remains open");
    }

    /// Save on a document without a file shows the Save As page only: no native dialog as well.
    #[test]
    fn backstage_save_when_unnamed_opens_no_file_dialog() {
        let shown = std::rc::Rc::new(std::cell::Cell::new(0));
        let count = shown.clone();
        let services = Services {
            file_dialog: Some(Box::new(move |_| {
                count.set(count.get() + 1);
                std::sync::mpsc::channel::<crate::FileDialogAnswer>().1
            })),
            ..Default::default()
        };
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services);
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Save").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.backstage_page, "saveAs");
        assert_eq!(shown.get(), 0, "no native Save As dialog");
        assert!(!h.state().file_dialog_open());
    }

    #[test]
    fn backstage_save_closes_when_saved() {
        let got: std::rc::Rc<std::cell::RefCell<Vec<(String, Vec<u8>)>>> = Default::default();
        let sink = got.clone();
        let services = Services {
            download: Some(Box::new(move |n: &str, b: &[u8]| {
                sink.borrow_mut().push((n.to_string(), b.to_vec()));
                Ok(())
            })),
            ..Default::default()
        };
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services);
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Save").click();
        h.run_steps(4);
        assert!(!h.state().ui.backstage, "successful save closes backstage");
    }

    /// #1124: the side menu had no Close. It sits after Export and closes the document, asking
    /// first when there are unsaved changes; Cancel leaves the Backstage as it was.
    #[test]
    fn backstage_close_asks_then_closes() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("text.insert", json!({"text": "Draft"})).unwrap();
        a.run("ui.backstage", json!({"value": true, "page": "info"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        let export = h.get_by_label("Export").rect().min.y;
        assert!(h.get_by_label("Close").rect().min.y > export, "Close comes after Export");
        h.get_by_label("Close").click();
        h.run_steps(4);
        assert!(matches!(h.state().dialog, Some(crate::dialogs::Dialog::SaveChanges { .. })), "unsaved changes ask first");
        assert!(!h.state().quit_requested);

        h.state_mut().run("ui.saveChanges", json!({"answer": "cancel"})).unwrap();
        h.run_steps(4);
        assert!(!h.state().quit_requested, "Cancel keeps the document open");
        assert!(h.state().ui.backstage && h.state().ui.backstage_page == "info", "Cancel leaves the Backstage as it was");

        h.get_by_label("Close").click();
        h.run_steps(4);
        h.state_mut().run("ui.saveChanges", json!({"answer": "dontSave"})).unwrap();
        assert!(h.state().quit_requested, "Don't Save closes");
    }

    /// A document without unsaved changes closes without a prompt.
    #[test]
    fn backstage_close_on_a_clean_document_closes_at_once() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Close").click();
        h.run_steps(1);
        assert!(h.state().dialog.is_none(), "no prompt");
        assert!(h.state().quit_requested);
    }

    #[test]
    fn backstage_save_as_sidebar_opens_save_as_page() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.backstage", json!({"value": true, "page": "home"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h.get_by_label("Save As").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.backstage_page, "saveAs");
    }

    /// File › Info in `lang` at 1440×900: the rects of the nav's Export item, the page heading, the
    /// Properties and Statistics captions, and the Title field.
    fn info_rects(lang: &str) -> [egui::Rect; 5] {
        use egui::accesskit::Role;
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::from_text("XYZ")), Services::default());
        a.run("ui.language", json!({"value": lang})).unwrap();
        a.run("ui.backstage", json!({"value": true, "page": "info"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        let l = crate::i18n::Lang::from_code(lang).unwrap();
        let tr = |s| crate::i18n::tr(l, s);
        // "Info" is both the nav item and the heading: the heading is the larger one.
        let heading = h.query_all_by_label(tr("Info")).map(|n| n.rect()).max_by(|a, b| a.height().total_cmp(&b.height())).unwrap();
        let field = h.get_by_role_and_label(Role::TextInput, tr("Title")).rect();
        let out =
            [h.get_by_label(tr("Export")).rect(), heading, h.get_by_label(tr("Properties")).rect(), h.get_by_label(tr("Statistics")).rect(), field];
        crate::i18n::set_current(crate::i18n::Lang::EN);
        out
    }

    /// Info's statistics come from the catalog template: "Pages: 2" in English; in Arabic the
    /// caption reads first, so in draw order the number is at the left and the caption at the right.
    #[test]
    fn info_statistics_read_caption_first_in_both_directions() {
        assert_eq!(super::stat_line("Pages", "2"), "Pages: 2");
        crate::i18n::set_current(crate::i18n::Lang::from_code("ar").unwrap());
        let label = tl!("Pages");
        let line = super::stat_line(label, "2");
        crate::i18n::set_current(crate::i18n::Lang::EN);
        assert!(line.starts_with('2') && line.ends_with(label), "arabic draw order: {line:?}");
        assert!(line.contains(':'), "{line:?}");
    }

    #[test]
    fn backstage_navigation_is_on_the_right_in_arabic() {
        // The Title field's caption is in the grid's first column, before the field: the field
        // starts well after the Properties caption's start edge.
        let [export, heading, props, stats, field] = info_rects("en");
        assert!(export.right() <= 220.0, "english: the nav column ({export:?}) is at the left");
        assert!(heading.left() < 300.0, "english: the heading ({heading:?}) starts at the left");
        assert!(props.center().x < stats.center().x, "english: Properties ({props:?}) is left of Statistics ({stats:?})");
        assert!(field.left() > props.left() + 20.0, "english: the Title caption ({props:?}) comes left of its field ({field:?})");
        let [export, heading, props, stats, field] = info_rects("ar");
        assert!(export.left() >= 1440.0 - 220.0, "arabic: the nav column ({export:?}) is at the right");
        assert!(heading.right() > 1000.0, "arabic: the heading ({heading:?}) starts at the right");
        assert!(props.center().x > stats.center().x, "arabic: Properties ({props:?}) is right of Statistics ({stats:?})");
        assert!(field.right() < props.right() - 20.0, "arabic: the Title caption comes right of its field ({field:?}, Properties {props:?})");
    }
}
