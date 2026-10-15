//! Dialogs: Font, Paragraph, Find & Replace, Go To, Insert Table, Page Setup, Link, Bookmark,
//! Word Count, Zoom, Watermark, New/Modify Style, Manage Styles, New/Modify Table Style, Table
//! Properties, Enclose Characters, Command search, Icons, Paste Special, About, Save Changes, the password to open a
//! document and Encrypt with Password, and the mail-merge Recipient List, Insert Merge Field, Find
//! Recipient, merge rules, Match Fields, Check for Errors and Select Table (a workbook's sheet) (Customize Keyboard is in
//! `dialogs_keyboard`, Customize Ribbon in `dialogs_ribbon`). Every dialog ends by running a
//! command (or shows one's result), so agents get the same result without the dialog.

use egui::{Sense, Ui, vec2};
use serde::Serialize;
use serde_json::{Value, json};

use wordcraft_engine::Password;

use crate::options::MeasureExt;
use crate::theme::{Tokens, semibold};
use crate::{WordApp, rtl};

/// The Find and Replace dialog's search options beyond the first row (#543).
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindExtra {
    pub wildcards: bool,
    pub prefix: bool,
    pub suffix: bool,
    pub ignore_punct: bool,
    pub ignore_space: bool,
    /// The search options are shown.
    pub more: bool,
    /// Special ▾ inserts into Replace with (it last had the focus), not Find what.
    pub to_replace: bool,
}

/// The tabs of the Find and Replace dialog (#712).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FindTab {
    #[default]
    Find,
    Replace,
    GoTo,
}

/// What Go To moves to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GoToWhat {
    #[default]
    Page,
    Bookmark,
}

/// The Special ▾ menu: (name, code) for Find what, without and with wildcards.
const SPECIAL_FIND: &[(&str, &str)] = &[
    ("Paragraph Mark", "^p"),
    ("Tab Character", "^t"),
    ("Any Character", "^?"),
    ("Any Digit", "^#"),
    ("Any Letter", "^$"),
    ("Caret Character", "^^"),
    ("Section Character", "^%"),
    ("Paragraph Character", "^v"),
    ("Column Break", "^n"),
    ("Em Dash", "^+"),
    ("En Dash", "^="),
    ("Graphic", "^g"),
    ("Manual Line Break", "^l"),
    ("Manual Page Break", "^m"),
    ("Nonbreaking Hyphen", "^~"),
    ("Nonbreaking Space", "^s"),
    ("Optional Hyphen", "^-"),
    ("Section Break", "^b"),
    ("White Space", "^w"),
];
const SPECIAL_FIND_WILD: &[(&str, &str)] = &[
    ("Any Character", "?"),
    ("Character in Range", "[-]"),
    ("Beginning of Word", "<"),
    ("End of Word", ">"),
    ("Expression", "()"),
    ("Not", "[!]"),
    ("Num Occurrences", "{,}"),
    ("Previous 1 or More", "@"),
    ("0 or More Characters", "*"),
    ("Paragraph Mark", "^13"),
    ("Tab Character", "^t"),
    ("Caret Character", "^^"),
    ("Section Character", "^%"),
    ("Paragraph Character", "^v"),
    ("Column Break", "^n"),
    ("Em Dash", "^+"),
    ("En Dash", "^="),
    ("Graphic", "^g"),
    ("Manual Line Break", "^l"),
    ("Manual Page Break", "^m"),
    ("Nonbreaking Hyphen", "^~"),
    ("Nonbreaking Space", "^s"),
    ("Optional Hyphen", "^-"),
];
/// Replace with; with wildcards, "Find What Expression" (`\1`) comes first.
const SPECIAL_REPLACE: &[(&str, &str)] = &[
    ("Paragraph Mark", "^p"),
    ("Tab Character", "^t"),
    ("Caret Character", "^^"),
    ("Section Character", "^%"),
    ("Paragraph Character", "^v"),
    ("Clipboard Contents", "^c"),
    ("Column Break", "^n"),
    ("Em Dash", "^+"),
    ("En Dash", "^="),
    ("Find What Text", "^&"),
    ("Manual Line Break", "^l"),
    ("Manual Page Break", "^m"),
    ("Nonbreaking Hyphen", "^~"),
    ("Nonbreaking Space", "^s"),
    ("Optional Hyphen", "^-"),
];

/// The Special ▾ menu button; returns the code picked.
fn special_menu(ui: &mut Ui, extra: &FindExtra) -> Option<&'static str> {
    let mut picked = None;
    // The drop-down arrow follows the word: at its left under RTL.
    let label = if rtl::is_rtl() { format!("▾ {}", tl!("Special")) } else { format!("{} ▾", tl!("Special")) };
    rtl::sub_menu(ui, label, |ui| {
        let wild_replace: &[(&str, &str)] = &[("Find What Expression", "\\1")];
        let lists: [&[(&str, &str)]; 2] = match (extra.to_replace, extra.wildcards) {
            (false, false) => [SPECIAL_FIND, &[]],
            (false, true) => [SPECIAL_FIND_WILD, &[]],
            (true, false) => [SPECIAL_REPLACE, &[]],
            (true, true) => [wild_replace, SPECIAL_REPLACE],
        };
        for (name, code) in lists.into_iter().flatten() {
            let item = if rtl::is_rtl() { rtl::shortcut_item(tl!(name), *code) } else { egui::Button::new(format!("{}   {code}", tl!(name))) };
            if ui.add(item).clicked() {
                picked = Some(*code);
                ui.close();
            }
        }
    });
    picked
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "dialog", rename_all = "camelCase")]
pub enum Dialog {
    Font {
        /// The font (Latin text) and the Asian text font, as the font lists name them, and their
        /// values when the dialog opened: OK changes only the ones changed (#505).
        font: String,
        font_was: String,
        font_ea: String,
        font_ea_was: String,
        /// Show the Asian text font box: a Chinese or Japanese interface, or East Asian text or
        /// an Asian font in the selection.
        asian: bool,
        size: String,
        bold: bool,
        italic: bool,
        underline: bool,
        strike: bool,
        sup: bool,
        sub: bool,
        small_caps: bool,
        caps: bool,
        hidden: bool,
        color: String,
        spacing: f32,
        /// The properties the fields stood for when the dialog opened: OK sends only what the
        /// user changed, so formatting the dialog can't show (a double underline) or left
        /// alone stays as it is.
        #[serde(skip)]
        start: serde_json::Map<String, Value>,
    },
    Paragraph {
        /// Right-to-left reading order (Direction in Word's Paragraph dialog).
        rtl: bool,
        /// Alignment and indents as seen on the page: `left` is the left side whatever the
        /// direction (the model stores them from the paragraph's start edge).
        align: String,
        left: f32,
        right: f32,
        first: f32,
        before: f32,
        after: f32,
        /// Line spacing as a multiple; Exactly and At Least show as 1.0.
        line: f32,
        /// `line` when the dialog opened: OK keeps the paragraph's spacing, whatever its rule,
        /// unless this changed (#467).
        #[serde(skip)]
        line_was: f32,
        keep_next: bool,
        keep_lines: bool,
        page_break: bool,
        widow: bool,
        /// Page: 0 Indents and Spacing, 1 Line and Page Breaks, 2 Asian Typography.
        tab: u8,
        /// Asian Typography as the model has it (`para.asianTypography`): kinsoku, word wrap,
        /// hanging punctuation, compress punctuation at line start, space between Asian and
        /// Latin text, space between Asian text and numbers.
        asian: [bool; 6],
        /// `asian` when the dialog opened: OK sets only the flags that changed.
        #[serde(skip)]
        asian_was: [bool; 6],
        /// Mirror indents, don't add space between paragraphs of the same style, suppress line
        /// numbers, don't hyphenate (#512).
        opts: [bool; 4],
        /// `opts` when the dialog opened: OK sets only the ones that changed.
        #[serde(skip)]
        opts_was: [bool; 4],
    },
    /// Tabs (#320).
    Tabs {
        form: Box<crate::dialogs_para::TabsForm>,
    },
    /// Borders and Shading (#320).
    Borders {
        form: Box<crate::dialogs_para::BordersForm>,
    },
    Find {
        query: String,
        replace: String,
        match_case: bool,
        whole_word: bool,
        regex: bool,
        /// Search options under "Search Options" (#543).
        extra: FindExtra,
        /// Find, Replace or Go To, one dialog as in Word (#712).
        tab: FindTab,
        goto_what: GoToWhat,
        /// Go To's box: a page number (`+2`, `-1` move from the current page) or a bookmark.
        goto_target: String,
        message: String,
        /// Put the caret in Find what on the first frame, so typing searches (#772).
        focus: bool,
    },
    InsertTable {
        rows: u32,
        cols: u32,
    },
    PageSetup {
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
        landscape: bool,
        /// Document grid: kind (0 none, 1 lines, 2 lines and characters, 3 snap to characters),
        /// lines per page and characters per line; `grid_was` as opened (applied only if changed).
        grid: (u8, u32, u32),
        grid_was: (u8, u32, u32),
        /// Section direction: right to left (`layout.sectionRtl`).
        rtl: bool,
    },
    Link {
        url: String,
        text: String,
    },
    Bookmark {
        name: String,
    },
    WordCount {
        stats: Value,
    },
    Zoom {
        percent: f32,
    },
    Watermark {
        text: String,
        diagonal: bool,
    },
    /// Home › Font › Phonetic Guide: ruby text over the selected text (`format.phonetic`).
    Phonetic {
        base: String,
        ruby: String,
        /// OOXML alignment name (`center`, `distributeLetter`…).
        align: String,
        size: f32,
        offset: f32,
        /// The selection already has a phonetic guide (Remove is offered).
        existing: bool,
    },
    NewStyle {
        name: String,
        based_on: String,
        /// Opened from Manage Styles, which comes back when this closes.
        #[serde(skip)]
        back: bool,
    },
    ModifyStyle {
        #[serde(skip)]
        back: bool,
        /// The dialog that comes back when this closes (Table of Contents › Modify…).
        #[serde(skip)]
        resume: Option<Box<Dialog>>,
        id: String,
        name: String,
        font: String,
        size: f32,
        bold: bool,
        italic: bool,
        color: String,
        before: f32,
        after: f32,
    },
    /// Manage Styles: every style with its preview and description; modify, create, delete and
    /// show or hide them.
    ManageStyles {
        alphabetical: bool,
        selected: String,
    },
    /// Table Design › New / Modify Table Style: one formatting region at a time.
    TableStyle {
        /// The style being modified; `None` creates one.
        id: Option<String>,
        name: String,
        /// Style id.
        based_on: String,
        /// Index into `regions` (see `REGION_LABELS`).
        region: usize,
        regions: Box<[TableRegion; 7]>,
        /// What the regions showed when the dialog opened (or the base style changed): only
        /// changes are sent, so everything else stays inherited.
        #[serde(skip)]
        basis: Box<[TableRegion; 7]>,
    },
    Commands {
        query: String,
        /// Opened by a shortcut that may also send its key's text (Alt+Q's "q", in the same frame
        /// or the next): that text isn't typed into the box; the next key pressed ends this (#819).
        #[serde(skip)]
        swallow: bool,
    },
    /// Columns, Symbol and Field ([`crate::dialogs_insert`], #321).
    Insert(Box<crate::dialogs_insert::InsertDialog>),
    /// Caption, Index, Mark Index Entry, Table of Contents, Source Manager and Create / Edit
    /// Source ([`crate::dialogs_refs`], #402).
    Refs(Box<crate::dialogs_refs::RefsDialog>),
    /// Insert › Icons: the icon library, searchable and filtered by category (empty for all).
    Icons {
        query: String,
        category: String,
        selected: String,
    },
    /// Paste Special: the clipboard's formats and the chosen one. The clipboard payload stays out
    /// of the serialized dialog state (it can be megabytes).
    PasteSpecial {
        formats: Vec<String>,
        choice: usize,
        #[serde(skip)]
        payload: Value,
        /// Shown when nothing can be pasted: what the clipboard holds, or that it is empty (#833).
        #[serde(skip)]
        note: String,
    },
    /// Tabs: 0 About, 1 Contributors, 2 Models.
    About {
        tab: u8,
    },
    /// "Do you want to save changes?" before a user's New, Open, Close, Envelopes, Labels or
    /// Finish & Merge replaces the document (see [`WordApp::run`]); `then` runs once it is
    /// answered (`ui.saveChanges`), and only while the document it asked about
    /// (`Session::document_id`) is still the one open.
    SaveChanges {
        name: String,
        then: String,
        #[serde(skip)]
        params: Value,
        #[serde(skip)]
        document: u64,
    },
    /// Mailings › Select Recipients / Edit Recipient List: a table of recipients to type or
    /// edit (field names on top), or a file to load instead (#240).
    RecipientList {
        fields: Vec<String>,
        rows: Vec<Vec<String>>,
        message: String,
    },
    /// Mailings › Select Recipients › Use an Existing List… picked a workbook with several sheets
    /// (#334): which sheet holds the recipients, and whether its first row names the fields.
    /// OK runs `mailings.recipients` with `params` (the file) plus `sheet` and `headers`.
    SelectTable {
        sheets: Vec<String>,
        selected: usize,
        headers: bool,
        message: String,
        #[serde(skip)]
        params: Value,
    },
    /// Mailings › Insert Merge Field: one of the recipient list's fields (or a typed name).
    InsertMergeField {
        fields: Vec<String>,
        field: String,
    },
    /// A field's code: a new field (Ctrl+F9, `edit` false) or the selected field's code to edit
    /// (double-click a field showing its code, right-click › Edit Field…).
    FieldCode {
        code: String,
        edit: bool,
        message: String,
    },
    /// Mailings › Find Recipient.
    FindRecipient {
        text: String,
        message: String,
    },
    /// Mailings › Rules › If…Then…Else… (`IF`) or Skip Record If… (`SKIPIF`): the condition, and
    /// for If the text to insert either way.
    MergeRule {
        rule: String,
        fields: Vec<String>,
        field: String,
        value: String,
        then: String,
        els: String,
    },
    /// Mailings › Match Fields: which recipient-list field fills each address part.
    MatchFields {
        fields: Vec<String>,
        address: Value,
    },
    /// Mailings › Check for Errors: merge fields the recipient list doesn't have.
    CheckErrors {
        unknown: Vec<String>,
        records: u64,
    },
    /// Table Layout › Properties: the table, the caret's row, column and cell. Only what changed
    /// from `basis` (the values when it opened) is applied, as one `table.properties` step.
    TableProperties {
        form: Box<TableForm>,
        #[serde(skip)]
        basis: Box<TableForm>,
    },
    /// Opening a password-protected document (#55): the password, then `file.open` with
    /// `params` again. The password is never serialised or printed.
    Password {
        name: String,
        #[serde(skip)]
        params: Value,
        #[serde(skip)]
        password: Password,
        message: String,
    },
    /// File › Info › Protect Document › Encrypt with Password: a password typed twice
    /// (`file.encrypt`); left empty, it removes the password.
    EncryptPassword {
        #[serde(skip)]
        password: Password,
        #[serde(skip)]
        confirm: Password,
        message: String,
    },
    /// Home › Paragraph › Multilevel List › Define New Multilevel List (#328).
    DefineList {
        form: Box<crate::dialogs_lists::ListForm>,
    },
    /// Review › Tracking › Track Changes Options (#328).
    TrackOptions {
        form: Box<crate::dialogs_lists::TrackForm>,
    },
    /// Compare / Combine Documents and AutoCorrect Options ([`crate::dialogs_review`], #412).
    Review(Box<crate::dialogs_review::ReviewDialog>),
    /// Tools › Customize Keyboard (#368).
    CustomizeKeyboard {
        form: Box<crate::dialogs_keyboard::KeyboardForm>,
    },
    /// Tools › Customize Ribbon and the Quick Access Toolbar (#379).
    CustomizeRibbon {
        form: Box<crate::dialogs_ribbon::RibbonForm>,
    },
    /// Developer › Controls › Properties.
    Control {
        form: Box<crate::dialogs_controls::ControlForm>,
    },
    /// Home › Font › Enclose Characters: the style (`none` takes the enclosure away, `shrink`,
    /// `enlarge`), the text and the shape (`circle`, `square`, `triangle`, `diamond`).
    Enclose {
        style: String,
        text: String,
        shape: String,
    },
    /// References › Footnotes › Footnote and Endnote (#385).
    NoteOptions {
        form: Box<crate::dialogs_notes::NoteForm>,
    },
    /// View › Templates › Templates and Add-ins (#377).
    Templates {
        form: Box<crate::dialogs_templates::TemplatesForm>,
    },
    /// Templates and Add-ins › Organizer: copy, delete and rename styles (#377).
    Organizer {
        form: Box<crate::dialogs_templates::OrganizerForm>,
    },
    /// Line Numbers, Hyphenation, Language, Envelopes and Labels (#407).
    Layout(Box<crate::dialogs_layout::LayoutDialog>),
}

/// The Table Properties dialog's fields. Lengths are in the interface unit
/// ([`wordcraft_geom::Unit`], inches by default).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableForm {
    pub width_on: bool,
    pub width: f32,
    /// `left`, `center` or `right`.
    pub align: String,
    pub indent: f32,
    /// Table direction: right to left (`w:bidiVisual`).
    pub rtl: bool,
    pub row_height_on: bool,
    pub row_height: f32,
    pub row_exact: bool,
    pub allow_break: bool,
    pub header_row: bool,
    pub column_width: f32,
    pub cell_width_on: bool,
    pub cell_width: f32,
    /// `top`, `center` or `bottom`.
    pub valign: String,
}

impl TableForm {
    /// The caret's table, row, column and cell as the dialog shows them.
    pub fn read(app: &WordApp) -> Option<TableForm> {
        use wordcraft_doc::props::{Align, HeightRule, VAlign};
        let s = &app.session;
        let (tp, r, c) = s.sel.focus.path.cell()?;
        let t = s.doc.table(s.sel.focus.story, &tp)?;
        let k = crate::options::unit().pt_per_unit();
        let row = t.rows.get(r).map(|x| x.props.clone()).unwrap_or_default();
        let cell = t.rows.get(r).and_then(|x| x.cells.get(c)).map(|x| x.props.clone()).unwrap_or_default();
        let tw = wordcraft_engine::cmd::page::sect(s).text_width();
        let width = t.props.width_pct.filter(|p| *p > 0.0).map(|p| tw * p.min(100.0) / 100.0).or(t.props.width.filter(|w| *w > 0.0));
        let fin = |v: f32| if v.is_finite() { v } else { 0.0 };
        Some(TableForm {
            width_on: width.is_some(),
            width: fin(width.unwrap_or(tw)) / k,
            align: match t.props.align {
                Some(Align::Center) => "center",
                Some(Align::Right) => "right",
                _ => "left",
            }
            .into(),
            indent: fin(t.props.indent.unwrap_or(0.0)) / k,
            rtl: t.props.bidi_visual,
            row_height_on: row.height.is_some() && row.height_rule != HeightRule::Auto,
            row_height: fin(row.height.unwrap_or(18.0)) / k,
            row_exact: row.height_rule == HeightRule::Exact,
            allow_break: !row.cant_split,
            header_row: row.header,
            column_width: fin(t.grid.get(t.grid_col(r, c)).copied().unwrap_or(72.0)) / k,
            cell_width_on: cell.width.is_some(),
            cell_width: fin(cell.width.unwrap_or(72.0)) / k,
            valign: match cell.valign {
                VAlign::Center => "center",
                VAlign::Bottom => "bottom",
                _ => "top",
            }
            .into(),
        })
    }

    /// `table.properties` parameters for what differs from `basis` (empty when nothing does).
    pub fn changes(&self, basis: &TableForm) -> Value {
        let k = crate::options::unit().pt_per_unit();
        let moved = |a: f32, b: f32| (a - b).abs() > 1e-4;
        let mut v = serde_json::Map::new();
        if self.width_on != basis.width_on || (self.width_on && moved(self.width, basis.width)) {
            v.insert("width".into(), if self.width_on { json!(self.width * k) } else { Value::Null });
        }
        if self.align != basis.align {
            v.insert("align".into(), json!(self.align));
        }
        if moved(self.indent, basis.indent) {
            v.insert("indent".into(), json!(self.indent * k));
        }
        if self.rtl != basis.rtl {
            v.insert("rtl".into(), json!(self.rtl));
        }
        if self.row_height_on != basis.row_height_on
            || (self.row_height_on && (moved(self.row_height, basis.row_height) || self.row_exact != basis.row_exact))
        {
            v.insert("rowHeight".into(), if self.row_height_on { json!(self.row_height * k) } else { Value::Null });
            v.insert("rowHeightRule".into(), json!(if self.row_exact { "exact" } else { "atLeast" }));
        }
        if self.allow_break != basis.allow_break {
            v.insert("allowBreak".into(), json!(self.allow_break));
        }
        if self.header_row != basis.header_row {
            v.insert("headerRow".into(), json!(self.header_row));
        }
        if moved(self.column_width, basis.column_width) {
            v.insert("columnWidth".into(), json!(self.column_width * k));
        }
        if self.cell_width_on != basis.cell_width_on || (self.cell_width_on && moved(self.cell_width, basis.cell_width)) {
            v.insert("cellWidth".into(), if self.cell_width_on { json!(self.cell_width * k) } else { Value::Null });
        }
        if self.valign != basis.valign {
            v.insert("valign".into(), json!(self.valign));
        }
        Value::Object(v)
    }
}

/// The address parts Match Fields lists, with their keys in `mailings.matchFields`' `address`.
pub const ADDRESS_PARTS: [(&str, &str); 9] = [
    ("Courtesy Title", "title"),
    ("First Name", "firstName"),
    ("Last Name", "lastName"),
    ("Company", "company"),
    ("Address", "address"),
    ("City", "city"),
    ("State", "state"),
    ("Postal Code", "postal"),
    ("Country", "country"),
];

/// The `mailings.rules` parameters a Merge Rule dialog stands for, once it names a field.
pub fn rule_params(d: &Dialog) -> Option<Value> {
    let Dialog::MergeRule { rule, field, value, then, els, .. } = d else { return None };
    let field = field.trim();
    if field.is_empty() {
        return None;
    }
    Some(if rule == "SKIPIF" {
        json!({"rule": "SKIPIF", "field": field, "value": value})
    } else {
        json!({"rule": "IF", "field": field, "value": value, "then": then, "else": els})
    })
}

/// The fields a new recipient list starts with.
pub const NEW_LIST_FIELDS: [&str; 5] = ["First Name", "Last Name", "Address", "City", "Postal Code"];

/// A typed recipient list ready for `mailings.recipients`: field names trimmed, blank rows
/// dropped. An error says what to fix.
pub fn recipient_table(fields: &[String], rows: &[Vec<String>]) -> Result<Value, &'static str> {
    let fields: Vec<&str> = fields.iter().map(|f| f.trim()).collect();
    if fields.is_empty() {
        return Err("Add at least one field.");
    }
    if fields.iter().any(|f| f.is_empty()) {
        return Err("Every field needs a name.");
    }
    for (i, f) in fields.iter().enumerate() {
        if fields.iter().skip(i + 1).any(|g| g.eq_ignore_ascii_case(f)) {
            return Err("Field names must be different.");
        }
    }
    let rows: Vec<&Vec<String>> = rows.iter().filter(|r| r.iter().any(|c| !c.trim().is_empty())).collect();
    if rows.is_empty() {
        return Err("Type at least one recipient.");
    }
    Ok(json!({"fields": fields, "rows": rows}))
}

/// One table style region in the Table Style dialog: colours are hex, empty for none.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRegion {
    pub borders: bool,
    pub fill: String,
    pub bold: bool,
    pub color: String,
}

/// The Table Style dialog's regions, in the order of `table_style::REGIONS` (the param names).
const REGION_LABELS: [&str; 7] = ["Whole Table", "Header Row", "Total Row", "First Column", "Last Column", "Banded Rows", "Banded Columns"];

/// The regions of table style `id` as resolved through its based-on chain.
fn table_regions(app: &WordApp, id: &str) -> Box<[TableRegion; 7]> {
    let Some(st) = app.session.doc.styles.table_style(id) else { return Default::default() };
    let p = &st.parts;
    let hex = |c: Option<wordcraft_doc::Rgb>| c.map(|c| c.hex()).unwrap_or_default();
    let text = |c: &wordcraft_doc::CharProps| match c.color {
        Some(wordcraft_doc::TextColor::Rgb(c)) => c.hex(),
        _ => String::new(),
    };
    let lines = |b: Option<wordcraft_doc::props::Borders>| b.is_some_and(|b| b.any_visible());
    // A region's text: the whole table's formatting with the region's over it.
    let region = |borders: Option<wordcraft_doc::props::Borders>, fill: Option<wordcraft_doc::Rgb>, chr: &wordcraft_doc::CharProps| {
        let mut c = st.chr.clone();
        c.overlay(chr);
        TableRegion { borders: lines(borders), fill: hex(fill), bold: c.bold.unwrap_or(false), color: text(&c) }
    };
    let none = wordcraft_doc::CharProps::default();
    Box::new([
        region(p.borders, p.fill, &none),
        region(p.header_borders, p.header_fill, &p.header_chr),
        region(p.total_borders, p.total_fill, &p.total_chr),
        region(p.first_col_borders, p.first_col_fill, &p.first_col_chr),
        region(p.last_col_borders, p.last_col_fill, &p.last_col_chr),
        region(p.band_borders, p.band_fill, &p.band_chr),
        region(p.col_band_borders, p.col_band_fill, &p.col_band_chr),
    ])
}

/// The params `table.newStyle` / `table.modifyStyle` take for the Table Style dialog's state:
/// only what changed from `basis`, so the rest stays inherited.
fn table_style_params(id: Option<&str>, name: &str, based_on: &str, regions: &[TableRegion; 7], basis: &[TableRegion; 7]) -> Value {
    let mut v = json!({"name": name.trim(), "basedOn": based_on});
    for ((key, r), b) in wordcraft_engine::cmd::table_style::REGIONS.into_iter().zip(regions.iter()).zip(basis.iter()) {
        let ch = region_changes(r, b);
        if ch.as_object().is_some_and(|o| !o.is_empty()) {
            v[key] = ch;
        }
    }
    if let Some(id) = id {
        v["style"] = json!(id);
    }
    v
}

/// A small sample table drawn with `style` (5 columns, a header, three body rows and a total
/// row), showing the regions `look` turns on: the Table Style dialog's live preview.
fn table_style_preview(ui: &mut Ui, style: Option<&wordcraft_doc::styles::TableStyleProps>, look: wordcraft_doc::props::TableLook) {
    use wordcraft_doc::props::{Border, Borders};
    const COLS: usize = 5;
    const ROWS: usize = 5;
    const TEXT: [[&str; COLS]; ROWS] = [
        ["", "Mon", "Tue", "Wed", "Sum"],
        ["North", "4", "7", "2", "13"],
        ["South", "6", "1", "5", "12"],
        ["West", "3", "8", "4", "15"],
        ["Total", "13", "16", "11", "40"],
    ];
    let (rect, _) = ui.allocate_exact_size(vec2(300.0, 120.0), Sense::hover());
    let painter = ui.painter_at(rect);
    // The page is white whatever the interface theme.
    painter.rect_filled(rect, 2.0, egui::Color32::WHITE);
    let t = Tokens::get(ui.ctx());
    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    let Some(st) = style else { return };
    let p = &st.parts;
    let table = rect.shrink2(vec2(14.0, 10.0));
    let (cw, rh) = (table.width() / COLS as f32, table.height() / ROWS as f32);
    let rgb = |c: wordcraft_doc::Rgb| egui::Color32::from_rgb(c.0, c.1, c.2);
    let tb = p.borders.unwrap_or_default();
    for (ri, row) in TEXT.iter().enumerate() {
        let header = look.header_row && ri == 0;
        let total = look.total_row && ri + 1 == ROWS;
        let band = look.banded_rows
            && !header
            && (ri.saturating_sub(usize::from(look.header_row)) / p.band_size.unwrap_or(1).clamp(1, 1000) as usize).is_multiple_of(2);
        for (ci, txt) in row.iter().enumerate() {
            // Sample words in the interface language; numbers as they are.
            let txt = &if txt.chars().any(char::is_alphabetic) { tl!(txt) } else { *txt };
            let first = look.first_column && ci == 0;
            let last = look.last_column && ci + 1 == COLS;
            let col_band = look.banded_columns && !first && ci.saturating_sub(usize::from(look.first_column)).is_multiple_of(2);
            let cell = egui::Rect::from_min_size(table.min + vec2(ci as f32 * cw, ri as f32 * rh), vec2(cw, rh));
            // Regions from lowest to highest priority, as layout applies them.
            let regions = [
                (col_band, p.col_band_fill, &p.col_band_chr, p.col_band_borders),
                (band, p.band_fill, &p.band_chr, p.band_borders),
                (first, p.first_col_fill, &p.first_col_chr, p.first_col_borders),
                (last, p.last_col_fill, &p.last_col_chr, p.last_col_borders),
                (header, p.header_fill, &p.header_chr, p.header_borders),
                (total, p.total_fill, &p.total_chr, p.total_borders),
            ];
            let mut fill = p.fill;
            let mut chr = st.chr.clone();
            let mut b = Borders {
                top: if ri == 0 { tb.top } else { tb.between },
                bottom: if ri + 1 == ROWS { tb.bottom } else { tb.between },
                left: if ci == 0 { tb.left } else { tb.inside_v },
                right: if ci + 1 == COLS { tb.right } else { tb.inside_v },
                between: None,
                inside_v: None,
            };
            for (on, f, c, rb) in regions {
                if !on {
                    continue;
                }
                fill = f.or(fill);
                chr.overlay(c);
                if let Some(rb) = rb {
                    b.overlay(&Borders { between: None, inside_v: None, ..rb });
                }
            }
            if total && p.total_borders.is_none_or(|x| x.top.is_none()) {
                b.top = p.total_border_top.or(b.top);
            }
            if let Some(f) = fill {
                painter.rect_filled(cell, 0.0, rgb(f));
            }
            let edge = |from: egui::Pos2, to: egui::Pos2, e: Option<Border>| {
                if let Some(e) = e.filter(Border::is_visible) {
                    let color = e.color.map_or(egui::Color32::BLACK, rgb);
                    painter.line_segment([from, to], egui::Stroke::new(e.width.clamp(0.5, 3.0) * 1.3, color));
                }
            };
            edge(cell.left_top(), cell.right_top(), b.top);
            edge(cell.left_bottom(), cell.right_bottom(), b.bottom);
            edge(cell.left_top(), cell.left_bottom(), b.left);
            edge(cell.right_top(), cell.right_bottom(), b.right);
            let color = match chr.color {
                Some(wordcraft_doc::TextColor::Rgb(c)) => rgb(c),
                _ => egui::Color32::BLACK,
            };
            let font = egui::FontId::proportional(11.0);
            let pos = cell.left_center() + vec2(4.0, 0.0);
            painter.text(pos, egui::Align2::LEFT_CENTER, tl!(*txt), font.clone(), color);
            if chr.bold.unwrap_or(false) {
                // A second pass a hair to the right reads as bold at this size.
                painter.text(pos + vec2(0.6, 0.0), egui::Align2::LEFT_CENTER, tl!(*txt), font, color);
            }
        }
    }
}

/// The style of the table at the caret, when it is a table style.
fn current_table_style(app: &WordApp) -> Option<String> {
    let s = &app.session;
    let (tp, _, _) = s.sel.focus.path.cell()?;
    let id = s.doc.table(s.sel.focus.story, &tp)?.props.style.clone()?;
    s.doc.styles.get(&id).filter(|st| st.kind == wordcraft_doc::StyleKind::Table).map(|st| st.id.clone())
}

/// The params `table.newStyle` / `table.modifyStyle` need for what changed in one region.
fn region_changes(r: &TableRegion, basis: &TableRegion) -> Value {
    let color = |c: &str| if c.is_empty() { Value::Null } else { json!(c) };
    let mut v = json!({});
    if r.borders != basis.borders {
        v["borders"] = json!(r.borders);
    }
    if r.fill != basis.fill {
        v["fill"] = color(&r.fill);
    }
    if r.bold != basis.bold {
        v["bold"] = json!(r.bold);
    }
    if r.color != basis.color {
        v["color"] = color(&r.color);
    }
    v
}

/// A colour menu: a swatch with the colour grid and No Color. `value` is hex, empty for none.
pub(crate) fn color_menu(ui: &mut Ui, theme: &[wordcraft_doc::Rgb], value: &mut String) {
    color_field(ui, theme, value, "No Color", "");
}

/// A colour field: a swatch and a drop-down with `none_label` (which sets `none_value`), the
/// colour grid, Recent Colors and More Colors… (#606). `value` is hex or `none_value`.
pub(crate) fn color_field(ui: &mut Ui, theme: &[wordcraft_doc::Rgb], value: &mut String, none_label: &str, none_value: &str) {
    rtl::row(ui, |ui| {
        let id = ui.id().with("colorField");
        crate::colors::take_field(ui.ctx(), id, value);
        let c = wordcraft_doc::Rgb::parse(value);
        let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
        let t = Tokens::get(ui.ctx());
        match c {
            Some(c) => {
                ui.painter().rect_filled(r, 2.0, crate::theme::c32(c));
            }
            None => {
                ui.painter().line_segment([r.left_bottom(), r.right_top()], egui::Stroke::new(1.0, t.text_dim));
            }
        }
        ui.painter().rect_stroke(r, 2.0, egui::Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
        let label = if c.is_some() { value.clone() } else { tl!(none_label).to_string() };
        rtl::sub_menu(ui, label, |ui| {
            if ui.button(tl!(none_label)).clicked() {
                *value = none_value.to_string();
                ui.close();
            }
            crate::colors::field_menu(ui, theme, id, value);
        });
    });
}

impl Dialog {
    pub fn name(&self) -> &'static str {
        match self {
            Dialog::Font { .. } => "font",
            Dialog::Paragraph { .. } => "paragraph",
            Dialog::Tabs { .. } => "tabs",
            Dialog::Borders { .. } => "borders",
            Dialog::Find { tab: FindTab::Find, .. } => "find",
            Dialog::Find { tab: FindTab::Replace, .. } => "replace",
            Dialog::Find { tab: FindTab::GoTo, .. } => "goto",
            Dialog::InsertTable { .. } => "insertTable",
            Dialog::PageSetup { .. } => "pageSetup",
            Dialog::Link { .. } => "link",
            Dialog::Bookmark { .. } => "bookmark",
            Dialog::WordCount { .. } => "wordCount",
            Dialog::Zoom { .. } => "zoom",
            Dialog::Watermark { .. } => "watermark",
            Dialog::Phonetic { .. } => "phonetic",
            Dialog::NewStyle { .. } => "newStyle",
            Dialog::ModifyStyle { .. } => "modifyStyle",
            Dialog::ManageStyles { .. } => "manageStyles",
            Dialog::TableStyle { id: None, .. } => "newTableStyle",
            Dialog::TableStyle { .. } => "modifyTableStyle",
            Dialog::Commands { .. } => "commands",
            Dialog::Insert(d) => d.name(),
            Dialog::Refs(d) => d.name(),
            Dialog::Icons { .. } => "icons",
            Dialog::PasteSpecial { .. } => "pasteSpecial",
            Dialog::About { .. } => "about",
            Dialog::SaveChanges { .. } => "saveChanges",
            Dialog::RecipientList { .. } => "recipientList",
            Dialog::SelectTable { .. } => "selectTable",
            Dialog::InsertMergeField { .. } => "insertMergeField",
            Dialog::FindRecipient { .. } => "findRecipient",
            Dialog::FieldCode { edit: false, .. } => "fieldCode",
            Dialog::FieldCode { .. } => "editFieldCode",
            Dialog::MergeRule { rule, .. } if rule == "SKIPIF" => "ruleSkipIf",
            Dialog::MergeRule { .. } => "ruleIf",
            Dialog::MatchFields { .. } => "matchFields",
            Dialog::CheckErrors { .. } => "checkErrors",
            Dialog::TableProperties { .. } => "tableProperties",
            Dialog::Password { .. } => "password",
            Dialog::EncryptPassword { .. } => "encryptPassword",
            Dialog::DefineList { .. } => "defineList",
            Dialog::TrackOptions { .. } => "trackChangesOptions",
            Dialog::Review(d) => d.name(),
            Dialog::CustomizeKeyboard { .. } => "customizeKeyboard",
            Dialog::CustomizeRibbon { .. } => "customizeRibbon",
            Dialog::Control { .. } => "controlProperties",
            Dialog::Enclose { .. } => "enclose",
            Dialog::NoteOptions { .. } => "noteOptions",
            Dialog::Templates { .. } => "templates",
            Dialog::Organizer { .. } => "organizer",
            Dialog::Layout(d) => d.name(),
        }
    }

    /// The window title (English; `show` translates it).
    pub fn title(&self) -> &'static str {
        match self {
            Dialog::Font { .. } => "Font",
            Dialog::Paragraph { .. } => "Paragraph",
            Dialog::Tabs { .. } => "Tabs",
            Dialog::Borders { .. } => "Borders and Shading",
            Dialog::Find { .. } => "Find and Replace",
            Dialog::InsertTable { .. } => "Insert Table",
            Dialog::PageSetup { .. } => "Page Setup",
            Dialog::Link { .. } => "Insert Hyperlink",
            Dialog::Bookmark { .. } => "Bookmark",
            Dialog::WordCount { .. } => "Word Count",
            Dialog::Zoom { .. } => "Zoom",
            Dialog::Watermark { .. } => "Custom Watermark",
            Dialog::Phonetic { .. } => "Phonetic Guide",
            Dialog::NewStyle { .. } => "Create New Style",
            Dialog::ModifyStyle { .. } => "Modify Style",
            Dialog::ManageStyles { .. } => "Manage Styles",
            Dialog::TableStyle { id: None, .. } => "New Table Style",
            Dialog::TableStyle { .. } => "Modify Table Style",
            Dialog::Commands { .. } => "Search Commands",
            Dialog::Insert(d) => d.title(),
            Dialog::Refs(d) => d.title(),
            Dialog::Icons { .. } => "Icons",
            Dialog::PasteSpecial { .. } => "Paste Special",
            Dialog::About { .. } => "About WordCraft",
            Dialog::SaveChanges { .. } => "WordCraft",
            Dialog::RecipientList { .. } => "Recipient List",
            Dialog::SelectTable { .. } => "Select Table",
            Dialog::InsertMergeField { .. } => "Insert Merge Field",
            Dialog::FindRecipient { .. } => "Find Recipient",
            Dialog::FieldCode { edit: false, .. } => "Field",
            Dialog::FieldCode { .. } => "Edit Field Code",
            Dialog::MergeRule { rule, .. } if rule == "SKIPIF" => "Skip Record If",
            Dialog::MergeRule { .. } => "If…Then…Else",
            Dialog::MatchFields { .. } => "Match Fields",
            Dialog::CheckErrors { .. } => "Check for Errors",
            Dialog::TableProperties { .. } => "Table Properties",
            Dialog::Password { .. } => "Password",
            Dialog::EncryptPassword { .. } => "Encrypt with Password",
            Dialog::DefineList { .. } => "Define New Multilevel List",
            Dialog::TrackOptions { .. } => "Track Changes Options",
            Dialog::Review(d) => d.title(),
            Dialog::CustomizeKeyboard { .. } => "Customize Keyboard",
            Dialog::CustomizeRibbon { .. } => "Customize Ribbon",
            Dialog::Control { .. } => "Content Control Properties",
            Dialog::Enclose { .. } => "Enclose Characters",
            Dialog::NoteOptions { .. } => "Footnote and Endnote",
            Dialog::Templates { .. } => "Templates and Add-ins",
            Dialog::Organizer { .. } => "Organizer",
            Dialog::Layout(d) => d.title(),
        }
    }

    /// The dialog that shows a report command's result: Match Fields, Check for Errors.
    pub fn report(id: &str, v: &Value) -> Option<Dialog> {
        let strings = |k: &str| -> Vec<String> {
            v.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default()
        };
        match id {
            "mailings.matchFields" => {
                Some(Dialog::MatchFields { fields: strings("fields"), address: v.get("address").cloned().unwrap_or(Value::Null) })
            }
            "mailings.checkErrors" => {
                Some(Dialog::CheckErrors { unknown: strings("unknownFields"), records: v.get("records").and_then(Value::as_u64).unwrap_or(0) })
            }
            _ => None,
        }
    }

    pub fn open(name: &str, app: &mut WordApp) -> Option<Dialog> {
        if let Some(d) = crate::dialogs_insert::open(name, app) {
            return Some(Dialog::Insert(Box::new(d)));
        }
        if let Some(d) = crate::dialogs_refs::open(name, app) {
            return Some(Dialog::Refs(Box::new(d)));
        }
        if let Some(d) = crate::dialogs_layout::open(name, app) {
            return Some(Dialog::Layout(Box::new(d)));
        }
        if matches!(name, "compare" | "combine" | "autoCorrect" | "restrictEditing") {
            return crate::dialogs_review::open(name, app).map(|d| Dialog::Review(Box::new(d)));
        }
        let st = app.session.run("format.state", &json!({})).unwrap_or_default();
        let s = |k: &str| st.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let b = |k: &str| st.get(k).and_then(Value::as_bool).unwrap_or(false);
        Some(match name {
            "font" => {
                let font = app.previews.font_shown(&s("font"));
                let font_ea = app.previews.font_shown(&s("fontEastAsia"));
                let lang = crate::i18n::current().code();
                let text = app.session.selected_text();
                let text = if text.is_empty() {
                    app.session.doc.para_at(&app.session.sel.focus).map(|p| p.plain_text().to_string()).unwrap_or_default()
                } else {
                    text
                };
                let asian = ["zh", "ja", "ko"].iter().any(|l| lang.starts_with(l)) || !font_ea.is_empty() || text.chars().any(east_asian_char);
                let mut d = Dialog::Font {
                    font_was: font.clone(),
                    font,
                    font_ea_was: font_ea.clone(),
                    font_ea,
                    asian,
                    size: st.get("size").and_then(Value::as_f64).map(|v| v.to_string()).unwrap_or_default(),
                    bold: b("bold"),
                    italic: b("italic"),
                    underline: b("underline"),
                    strike: b("strike"),
                    sup: b("superscript"),
                    sub: b("subscript"),
                    small_caps: b("smallCaps"),
                    caps: b("caps"),
                    hidden: b("hidden"),
                    color: s("color"),
                    spacing: st.get("spacing").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                    start: serde_json::Map::new(),
                };
                let props = font_props(&d);
                if let Dialog::Font { start, .. } = &mut d {
                    *start = props;
                }
                d
            }
            "paragraph" | "asianTypography" => {
                let rp = app.session.doc.para_at(&app.session.sel.focus).map(|p| app.session.doc.styles.resolve_para(&p.props));
                let rp = rp?;
                let line = match rp.line_spacing {
                    wordcraft_doc::props::LineSpacing::Multiple(m) => m,
                    _ => 1.0,
                };
                let (vl, vr) = rp.visual_indents();
                Dialog::Paragraph {
                    rtl: rp.bidi,
                    align: format!("{:?}", rp.align.visual(rp.bidi)).to_lowercase(),
                    left: vl / crate::options::unit().pt_per_unit(),
                    right: vr / crate::options::unit().pt_per_unit(),
                    first: rp.indent_first / crate::options::unit().pt_per_unit(),
                    before: rp.space_before,
                    after: rp.space_after,
                    line,
                    line_was: line,
                    keep_next: rp.keep_next,
                    keep_lines: rp.keep_lines,
                    page_break: rp.page_break_before,
                    widow: rp.widow_control,
                    tab: if name == "asianTypography" { 2 } else { 0 },
                    asian: [rp.kinsoku, rp.word_wrap, rp.overflow_punct, rp.top_line_punct, rp.auto_space_de, rp.auto_space_dn],
                    asian_was: [rp.kinsoku, rp.word_wrap, rp.overflow_punct, rp.top_line_punct, rp.auto_space_de, rp.auto_space_dn],
                    opts: [rp.mirror_indents, rp.contextual_spacing, rp.suppress_line_numbers, rp.suppress_hyphens],
                    opts_was: [rp.mirror_indents, rp.contextual_spacing, rp.suppress_line_numbers, rp.suppress_hyphens],
                }
            }
            "tabs" => Dialog::Tabs { form: Box::new(crate::dialogs_para::TabsForm::read(app)?) },
            "borders" | "pageBorders" | "shading" => {
                let tab = match name {
                    "pageBorders" => 1,
                    "shading" => 2,
                    _ => 0,
                };
                Dialog::Borders { form: Box::new(crate::dialogs_para::BordersForm::read(app, tab)?) }
            }
            "find" | "replace" | "goto" => Dialog::Find {
                query: if app.session.sel.is_collapsed() { app.session.find.query.clone() } else { app.session.selected_text() },
                replace: app.session.find.replace.clone(),
                match_case: app.session.find.match_case,
                whole_word: app.session.find.whole_word,
                regex: app.session.find.regex,
                extra: FindExtra {
                    wildcards: app.session.find.wildcards,
                    prefix: app.session.find.match_prefix,
                    suffix: app.session.find.match_suffix,
                    ignore_punct: app.session.find.ignore_punct,
                    ignore_space: app.session.find.ignore_space,
                    more: app.session.find.wildcards
                        || app.session.find.match_prefix
                        || app.session.find.match_suffix
                        || app.session.find.ignore_punct
                        || app.session.find.ignore_space,
                    to_replace: false,
                },
                tab: match name {
                    "replace" => FindTab::Replace,
                    "goto" => FindTab::GoTo,
                    _ => FindTab::Find,
                },
                goto_what: GoToWhat::Page,
                goto_target: String::new(),
                message: String::new(),
                focus: true,
            },
            "insertTable" => Dialog::InsertTable { rows: 2, cols: 5 },
            "pageSetup" => {
                let sp = wordcraft_engine::cmd::page::sect(&app.session);
                use wordcraft_doc::section::DocGridType;
                let kind = match sp.doc_grid.map(|g| g.kind) {
                    Some(DocGridType::Lines) => 1,
                    Some(DocGridType::LinesAndChars) => 2,
                    Some(DocGridType::SnapToChars) => 3,
                    _ => 0,
                };
                let size = app.session.doc.styles.resolve_char(Some("Normal"), &Default::default()).size;
                let grid = (kind, sp.grid_lines_per_page().min(1000), sp.grid_chars_per_line(size).min(1000));
                Dialog::PageSetup {
                    top: sp.margin_top / crate::options::unit().pt_per_unit(),
                    bottom: sp.margin_bottom / crate::options::unit().pt_per_unit(),
                    left: sp.margin_left / crate::options::unit().pt_per_unit(),
                    right: sp.margin_right / crate::options::unit().pt_per_unit(),
                    landscape: sp.landscape,
                    grid,
                    grid_was: grid,
                    rtl: sp.rtl,
                }
            }
            "link" => Dialog::Link { url: "https://".into(), text: app.session.selected_text() },
            "bookmark" => Dialog::Bookmark { name: String::new() },
            "wordCount" => Dialog::WordCount { stats: app.session.run("review.wordCount", &json!({})).unwrap_or_default() },
            "zoom" => Dialog::Zoom { percent: (app.session.view.zoom * 100.0).round() },
            "watermark" => Dialog::Watermark { text: "CONFIDENTIAL".into(), diagonal: true },
            "phonetic" => {
                let v = wordcraft_engine::cmd::phonetic::state(&app.session);
                let f = |k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0) as f32;
                let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
                Dialog::Phonetic {
                    base: s("base"),
                    ruby: s("ruby"),
                    align: s("align"),
                    size: f("size"),
                    offset: f("offset"),
                    existing: v.get("existing").and_then(Value::as_bool).unwrap_or(false),
                }
            }
            "newStyle" => Dialog::NewStyle { name: "Style1".into(), based_on: "Normal".into(), back: false },
            "manageStyles" => Dialog::ManageStyles { alphabetical: false, selected: s("style") },
            "newTableStyle" => {
                let based_on = current_table_style(app).unwrap_or_else(|| "TableGrid".into());
                let styles = &app.session.doc.styles;
                let name = (1..1000).map(|i| format!("Table Style {i}")).find(|n| styles.find(n).is_none()).unwrap_or_default();
                let regions = table_regions(app, &based_on);
                Dialog::TableStyle { id: None, name, based_on, region: 0, basis: regions.clone(), regions }
            }
            "modifyTableStyle" => {
                let id = current_table_style(app)?;
                let st = app.session.doc.styles.get(&id)?;
                let (name, based_on) = (st.name.clone(), st.based_on.clone().unwrap_or_default());
                let regions = table_regions(app, &id);
                Dialog::TableStyle { id: Some(id), name, based_on, region: 0, basis: regions.clone(), regions }
            }
            "commands" => Dialog::Commands { query: String::new(), swallow: true },
            "icons" => Dialog::Icons { query: String::new(), category: String::new(), selected: String::new() },
            "tableProperties" => {
                let form = Box::new(TableForm::read(app)?);
                Dialog::TableProperties { basis: form.clone(), form }
            }
            "enclose" => {
                // The selection's text and its enclosure, if it has one.
                let t = wordcraft_engine::cmd::format::enclose_target(&app.session).ok();
                let cur = t.as_ref().and_then(|t| t.current.clone());
                Dialog::Enclose {
                    style: cur.as_ref().map_or("shrink", |e| e.style.name()).into(),
                    shape: cur.as_ref().map_or("circle", |e| e.shape.name()).into(),
                    text: t.map(|t| t.text).unwrap_or_default(),
                }
            }
            "pasteSpecial" => Dialog::paste_special(app, &json!({})),
            "about" => Dialog::About { tab: 0 },
            "contributors" => Dialog::About { tab: 1 },
            "models" => Dialog::About { tab: 2 },
            // The current list to edit, or a new one to type.
            "recipientList" | "newRecipientList" => {
                let m = &app.session.merge;
                if name == "recipientList" && !m.headers.is_empty() {
                    Dialog::RecipientList { fields: m.headers.clone(), rows: m.rows.clone(), message: String::new() }
                } else {
                    let fields: Vec<String> = NEW_LIST_FIELDS.iter().map(|f| f.to_string()).collect();
                    let rows = vec![vec![String::new(); fields.len()]];
                    Dialog::RecipientList { fields, rows, message: String::new() }
                }
            }
            "insertMergeField" => {
                let fields = app.session.merge.headers.clone();
                Dialog::InsertMergeField { field: fields.first().cloned().unwrap_or_default(), fields }
            }
            "findRecipient" => Dialog::FindRecipient { text: String::new(), message: String::new() },
            "fieldCode" => Dialog::FieldCode { code: String::new(), edit: false, message: String::new() },
            "editFieldCode" => {
                let field = wordcraft_engine::cmd::fields::selected(&app.session).into_iter().next()?;
                Dialog::FieldCode { code: field.instr, edit: true, message: String::new() }
            }
            "encryptPassword" => Dialog::EncryptPassword { password: Password::default(), confirm: Password::default(), message: String::new() },
            "ruleIf" | "ruleSkipIf" => {
                let fields = app.session.merge.headers.clone();
                Dialog::MergeRule {
                    rule: if name == "ruleSkipIf" { "SKIPIF" } else { "IF" }.into(),
                    field: fields.first().cloned().unwrap_or_default(),
                    fields,
                    value: String::new(),
                    then: String::new(),
                    els: String::new(),
                }
            }
            "matchFields" | "checkErrors" => {
                let id = if name == "matchFields" { "mailings.matchFields" } else { "mailings.checkErrors" };
                return Self::report(id, &app.session.run(id, &json!({})).unwrap_or_default());
            }
            "defineList" => Dialog::DefineList { form: Box::new(crate::dialogs_lists::ListForm::read(app)) },
            "trackChangesOptions" => Dialog::TrackOptions { form: Box::new(crate::dialogs_lists::TrackForm::read(app)) },
            "customizeKeyboard" => Dialog::CustomizeKeyboard { form: Box::new(crate::dialogs_keyboard::KeyboardForm::read(app)) },
            "customizeRibbon" => {
                Dialog::CustomizeRibbon { form: Box::new(crate::dialogs_ribbon::RibbonForm::read(app, crate::dialogs_ribbon::Page::Ribbon)) }
            }
            "customizeQuickAccess" => {
                Dialog::CustomizeRibbon { form: Box::new(crate::dialogs_ribbon::RibbonForm::read(app, crate::dialogs_ribbon::Page::QuickAccess)) }
            }
            "noteOptions" => Dialog::NoteOptions { form: Box::new(crate::dialogs_notes::NoteForm::read(app)) },
            "templates" => Dialog::Templates { form: Box::new(crate::dialogs_templates::TemplatesForm::read(app)) },
            "organizer" => Dialog::Organizer { form: Box::new(crate::dialogs_templates::OrganizerForm::read(app, None)) },
            "controlProperties" => match crate::dialogs_controls::ControlForm::read(app) {
                Some(form) => Dialog::Control { form: Box::new(form) },
                None => {
                    app.status(tl!("Put the caret in a content control first."));
                    return None;
                }
            },
            _ => return None,
        })
    }

    /// Paste Special for a clipboard payload (`text`, `html`, `rtf`; empty = WordCraft's own copy).
    pub fn paste_special(app: &WordApp, payload: &Value) -> Dialog {
        let formats: Vec<String> = wordcraft_engine::cmd::paste::available(&app.session, payload).into_iter().map(str::to_string).collect();
        let mut kept = json!({});
        for k in ["text", "html", "rtf", "picture"] {
            if let Some(t) = payload.get(k).and_then(Value::as_str) {
                kept[k] = json!(t);
            }
        }
        let note = if formats.is_empty() { crate::system_clipboard::nothing_to_paste(app) } else { String::new() };
        Dialog::PasteSpecial { formats, choice: 0, payload: kept, note }
    }

    pub fn modify_style(app: &WordApp, id: &str) -> Option<Dialog> {
        let st = app.session.doc.styles.get(id)?;
        let rc = app.session.doc.styles.resolve_char(
            if st.kind == wordcraft_doc::StyleKind::Paragraph { Some(id) } else { None },
            &wordcraft_doc::CharProps {
                style: if st.kind == wordcraft_doc::StyleKind::Character { Some(id.into()) } else { None },
                ..Default::default()
            },
        );
        let rp = app.session.doc.styles.resolve_para(&wordcraft_doc::ParaProps { style: Some(id.into()), ..Default::default() });
        Some(Dialog::ModifyStyle {
            back: false,
            resume: None,
            id: id.into(),
            name: st.name.clone(),
            font: rc.font,
            size: rc.size,
            bold: rc.bold,
            italic: rc.italic,
            color: match rc.color {
                wordcraft_doc::TextColor::Rgb(c) => c.hex(),
                wordcraft_doc::TextColor::Auto => "auto".into(),
            },
            before: rp.space_before,
            after: rp.space_after,
        })
    }
}

/// Grid picker for Insert › Table (hover to size, click to insert).
pub fn table_grid_picker(ui: &mut Ui, app: &mut WordApp) {
    let t = Tokens::get(ui.ctx());
    let id = egui::Id::new("table_picker_hover");
    let hover: (usize, usize) = ui.data(|d| d.get_temp(id)).unwrap_or((0, 0));
    ui.label(if hover.0 > 0 {
        crate::i18n::fmt(tl!("{cols}×{rows} Table"), &[("cols", &hover.1.to_string()), ("rows", &hover.0.to_string())])
    } else {
        tl!("Insert Table").to_string()
    });
    let mut new_hover = (0, 0);
    let mut clicked = None;
    // Right to left (the Insert › Table menu) column 1 is the right one (`rtl::cell_grid`).
    crate::rtl::cell_grid(ui, "tgp", Some(vec2(2.0, 2.0)), 10, 80, |ui, i| {
        let (r, c) = (i / 10 + 1, i % 10 + 1);
        let (rect, resp) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
        let on = r <= hover.0 && c <= hover.1;
        ui.painter().rect(
            rect,
            1.0,
            if on { t.checked } else { t.input },
            egui::Stroke::new(1.0, if on { t.accent } else { t.border_strong }),
            egui::StrokeKind::Inside,
        );
        if resp.hovered() {
            new_hover = (r, c);
        }
        if resp.clicked() {
            clicked = Some((r, c));
        }
    });
    ui.data_mut(|d| d.insert_temp(id, new_hover));
    if let Some((r, c)) = clicked {
        let _ = app.run("insert.table", json!({"rows": r, "cols": c}));
        ui.close();
    }
}

/// The open dialog, as (window id, frames it has been centred for).
fn dialog_top_key() -> egui::Id {
    egui::Id::new("dialog_top")
}

/// Frames a dialog stays centred: the first one measures it, the second centres that size.
const DIALOG_CENTRED_FRAMES: u8 = 2;

/// Places a dialog: centred for its first frames, then left where it is with its top-centre point
/// as the pivot, so a taller or shorter tab (Paragraph, Font, Borders and Shading…) only moves
/// the bottom edge and the tab strip stays under the mouse (#507). It is no longer anchored after
/// that, so dragging the title bar moves it (#732). egui keeps it on screen if it grows past the
/// bottom.
fn place_dialog<'a>(ctx: &egui::Context, id: egui::Id, w: egui::Window<'a>) -> egui::Window<'a> {
    let w = w.pivot(egui::Align2::CENTER_TOP);
    match ctx.data(|d| d.get_temp::<(egui::Id, u8)>(dialog_top_key())) {
        Some((open, frames)) if open == id && frames >= DIALOG_CENTRED_FRAMES => w,
        _ => w.anchor(egui::Align2::CENTER_CENTER, vec2(0.0, -40.0)),
    }
}

/// After a frame of a dialog: counts the frames it has been centred for.
fn count_dialog_frame(ctx: &egui::Context, id: egui::Id) {
    let frames = match ctx.data(|d| d.get_temp::<(egui::Id, u8)>(dialog_top_key())) {
        Some((open, frames)) if open == id => frames,
        _ => 0,
    };
    if frames < DIALOG_CENTRED_FRAMES {
        ctx.data_mut(|d| d.insert_temp(dialog_top_key(), (id, frames + 1)));
    }
}

pub fn show(app: &mut WordApp, ctx: &egui::Context) {
    let Some(mut d) = app.dialog.take() else {
        ctx.data_mut(|d| d.remove::<(egui::Id, u8)>(dialog_top_key()));
        return;
    };
    let mut open = true;
    let mut close = false;
    let title = d.title();
    let id = egui::Id::new(("dialog", title));
    let shown = place_dialog(ctx, id, egui::Window::new(tl!(title)).id(id).collapsible(false).resizable(false)).open(&mut open).show(ctx, |ui| {
        ui.set_min_width(340.0);
        if crate::rtl::is_rtl() {
            // Every dialog is mirrored: captions, headings and rows start at the right.
            close = crate::rtl::column(ui, |ui| body(app, ui, &mut d)).inner;
        } else {
            close = body(app, ui, &mut d);
        }
    });
    if shown.is_some() {
        count_dialog_frame(ctx, id);
    }
    if open && !close {
        app.dialog = Some(d);
    } else {
        app.canvas.want_focus = true;
    }
}

pub(crate) fn buttons(ui: &mut Ui, ok: &str) -> (bool, bool) {
    let mut r = (false, false);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        // At the end edge, OK before Cancel in reading order: OK is the rightmost under RTL.
        ui.with_layout(crate::rtl::end_layout(egui::Align::Center), |ui| {
            // Enter and Escape belong to the Colors dialog while it is open above this one.
            let keys = !crate::colors::is_open(ui.ctx());
            if ui.button(tl!("Cancel")).clicked() || (keys && ui.input(|i| i.key_pressed(egui::Key::Escape))) {
                r.1 = true;
            }
            let okb = ui.add(crate::widgets::primary_button(ui, ok));
            if okb.clicked() || (keys && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                r.0 = true;
            }
        });
    });
    r
}

/// A form caption (`tl!(text)`), e.g. a grid row's first cell.
pub(crate) fn label(ui: &mut Ui, text: &str) {
    ui.label(tl!(text));
}

/// A dialog's tab strip: one selectable label per page, from the start edge (the right under RTL).
pub(crate) fn tab_strip(ui: &mut Ui, tab: &mut u8, names: &[&str]) {
    rtl::row(ui, |ui| {
        for (k, name) in names.iter().enumerate() {
            if ui.selectable_label(usize::from(*tab) == k, tl!(name)).clicked() {
                *tab = u8::try_from(k).unwrap_or(u8::MAX);
            }
        }
    });
}

/// A length field in the measurement unit ([`MeasureExt::measure`]), 3.6 pt per point dragged.
fn length_value(v: &mut f32) -> egui::DragValue<'_> {
    egui::DragValue::new(v).speed(3.6 / crate::options::unit().pt_per_unit()).measure(crate::options::unit()).max_decimals(2)
}

/// A field in points: ` pt` after the number, or under RTL the unit before it in draw order, so
/// it reads number-then-unit from the right.
fn points(v: egui::DragValue<'_>) -> egui::DragValue<'_> {
    if rtl::is_rtl() { v.prefix(format!("{} ", tl!("pt"))) } else { v.suffix(format!(" {}", tl!("pt"))) }
}

/// Under RTL, widen the current grid cell to `width` before a field that fills its cell (a text
/// field). A right-to-left cell is only as wide as its column was last frame, so the field would
/// otherwise start at egui's minimum cell width and stay there. English: nothing.
pub(crate) fn cell_width(ui: &mut Ui, width: f32) {
    if rtl::is_rtl() {
        ui.set_min_width(width);
    }
}

/// A single-line text field `width` wide (egui's default text field width when `None`) that keeps
/// that width in a right-to-left grid cell ([`cell_width`]). English: exactly
/// `ui.text_edit_singleline` (or with `desired_width`).
pub(crate) fn text_field(ui: &mut Ui, text: &mut String, width: Option<f32>) -> egui::Response {
    cell_width(ui, width.unwrap_or(ui.spacing().text_edit_width));
    match width {
        Some(w) => ui.add(egui::TextEdit::singleline(text).desired_width(w)),
        None => ui.text_edit_singleline(text),
    }
}

/// The character properties the Font dialog's fields stand for.
fn font_props(d: &Dialog) -> serde_json::Map<String, Value> {
    let Dialog::Font { font, font_ea, size, bold, italic, underline, strike, sup, sub, small_caps, caps, hidden, color, spacing, .. } = d else {
        return serde_json::Map::new();
    };
    let mut props = serde_json::Map::new();
    props.insert("bold".into(), json!(*bold));
    props.insert("italic".into(), json!(*italic));
    props.insert("strike".into(), json!(*strike));
    props.insert("underline".into(), json!(if *underline { "single" } else { "none" }));
    let vert = if *sup {
        "superscript"
    } else if *sub {
        "subscript"
    } else {
        "baseline"
    };
    props.insert("vertAlign".into(), json!(vert));
    props.insert("smallCaps".into(), json!(*small_caps));
    props.insert("caps".into(), json!(*caps));
    props.insert("hidden".into(), json!(*hidden));
    props.insert("spacing".into(), json!(*spacing));
    if !font.trim().is_empty() {
        props.insert("font".into(), json!(font.trim()));
    }
    if !font_ea.trim().is_empty() {
        props.insert("fontEa".into(), json!(font_ea.trim()));
    }
    if let Ok(s) = size.trim().parse::<f64>() {
        props.insert("size".into(), json!(s));
    }
    if let Some(c) = wordcraft_doc::Rgb::parse(color) {
        props.insert("color".into(), json!({"Rgb": [c.0, c.1, c.2]}));
    } else if color.eq_ignore_ascii_case("auto") {
        // Automatic, from the colour field's drop-down (#606).
        props.insert("color".into(), json!("Auto"));
    }
    props
}

/// Is `c` Chinese, Japanese or Korean text (ideographs, kana, Hangul, CJK punctuation, fullwidth
/// forms)?
fn east_asian_char(c: char) -> bool {
    matches!(u32::from(c), 0x1100..=0x11FF | 0x2E80..=0x9FFF | 0xA960..=0xA97F | 0xAC00..=0xD7FF | 0xF900..=0xFAFF | 0xFE30..=0xFE4F | 0xFF00..=0xFFEF | 0x20000..=0x3FFFF)
}

/// The Font dialog's OK: one formatting step that sets only the properties the user changed, so
/// formatting the dialog can't show or left alone stays as it is. Fonts are set by their stored
/// (English) names, the Latin font for Persian/Arabic text too (as `format.font`).
fn apply_font(app: &mut WordApp, d: &Dialog) {
    let Dialog::Font { start, .. } = d else { return };
    let mut changed: serde_json::Map<String, Value> = font_props(d).into_iter().filter(|(k, v)| start.get(k) != Some(v)).collect();
    if let Some(f) = changed.get("font").and_then(Value::as_str).map(str::to_string) {
        let f = app.previews.font_stored(&f);
        changed.insert("font".into(), json!(f));
        changed.insert("fontCs".into(), json!(f));
    }
    if let Some(f) = changed.get("fontEa").and_then(Value::as_str).map(str::to_string) {
        changed.insert("fontEa".into(), json!(app.previews.font_stored(&f)));
    }
    if !changed.is_empty() {
        let _ = app.run("format.set", json!({"props": changed}));
    }
}

/// Returns true to close.
fn body(app: &mut WordApp, ui: &mut Ui, d: &mut Dialog) -> bool {
    match d {
        Dialog::Font { font, font_ea, asian, size, bold, italic, underline, strike, sup, sub, small_caps, caps, hidden, color, spacing, .. } => {
            // Build the sample text format for the preview pane below.
            let preview_size = size.trim().parse::<f32>().unwrap_or(12.0).clamp(6.0, 72.0);
            let preview_color = wordcraft_doc::Rgb::parse(color).map(crate::theme::c32).unwrap_or_else(|| ui.visuals().text_color());
            let mut fmt = egui::TextFormat {
                font_id: egui::FontId::new(preview_size, egui::FontFamily::Proportional),
                color: preview_color,
                ..Default::default()
            };
            if *bold {
                fmt.font_id = crate::theme::semibold(preview_size);
            }
            if *italic {
                fmt.italics = true;
            }
            if *underline {
                fmt.underline = egui::Stroke::new(1.0, preview_color);
            }
            if *strike {
                fmt.strikethrough = egui::Stroke::new(1.0, preview_color);
            }
            rtl::fitted(ui, "fontdlg", |ui| {
                rtl::grid("fontdlg", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                    // Word's two font boxes when East Asian text is in use: the Latin text font and
                    // the Asian text font, each with its own list (type to find a font).
                    let prev = app.previews.font_preview_fn();
                    if *asian {
                        let fams = app.previews.east_asian_families();
                        rtl::grid_row(
                            ui,
                            [&mut |ui: &mut Ui| label(ui, "Asian text font:"), &mut |ui: &mut Ui| {
                                if let Some(f) = crate::widgets::font_combo(ui, "fontdlg_asian", 220.0, font_ea, &fams, Some(&*prev)) {
                                    *font_ea = f;
                                }
                            }],
                        );
                    }
                    let fams = app.previews.families();
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, if *asian { "Latin text font:" } else { "Font:" }), &mut |ui: &mut Ui| {
                            if let Some(f) = crate::widgets::font_combo(ui, "fontdlg_font", 220.0, font, &fams, Some(&*prev)) {
                                *font = f;
                            }
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Size:"), &mut |ui: &mut Ui| {
                            text_field(ui, size, None);
                        }],
                    );
                    let theme = app.session.doc.settings.theme_colors.clone();
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Font color:"), &mut |ui: &mut Ui| color_field(ui, &theme, color, "Automatic", "auto")],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Spacing (pt):"), &mut |ui: &mut Ui| {
                            ui.add(egui::DragValue::new(spacing).speed(0.1).range(-20.0..=20.0));
                        }],
                    );
                });
            });
            ui.separator();
            ui.label(egui::RichText::new(tl!("Effects")).font(semibold(12.5)));
            rtl::fitted(ui, "fx", |ui| {
                rtl::grid("fx", 2).show(ui, |ui| {
                    for (a, la, b, lb) in [
                        (bold, "Bold", italic, "Italic"),
                        (underline, "Underline", strike, "Strikethrough"),
                        (sup, "Superscript", sub, "Subscript"),
                        (small_caps, "Small caps", caps, "All caps"),
                    ] {
                        rtl::grid_row(
                            ui,
                            [
                                &mut |ui: &mut Ui| {
                                    rtl::checkbox(ui, a, tl!(la));
                                },
                                &mut |ui: &mut Ui| {
                                    rtl::checkbox(ui, b, tl!(lb));
                                },
                            ],
                        );
                    }
                    // Two cells, so Hidden is in the first (right under RTL) column.
                    rtl::grid_row(
                        ui,
                        [
                            &mut |ui: &mut Ui| {
                                rtl::checkbox(ui, hidden, tl!("Hidden"));
                            },
                            &mut |_: &mut Ui| {},
                        ],
                    );
                });
            });
            // Live preview of the chosen font/size/colour/effects.
            ui.separator();
            ui.label(egui::RichText::new(tl!("Preview")).small().weak());
            let sample = if *caps { tl!("AaBbYyZz").to_uppercase() } else { tl!("AaBbYyZz").to_string() };
            let mut job = egui::text::LayoutJob::single_section(sample, fmt);
            job.wrap.max_width = ui.available_width().min(400.0);
            ui.label(job);
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                apply_font(app, d);
            }
            ok || cancel
        }
        Dialog::Paragraph {
            rtl,
            align,
            left,
            right,
            first,
            before,
            after,
            line,
            line_was,
            keep_next,
            keep_lines,
            page_break,
            widow,
            tab,
            asian,
            asian_was,
            opts,
            opts_was,
        } => {
            tab_strip(ui, tab, &["Indents and Spacing", "Line and Page Breaks", "Asian Typography"]);
            ui.separator();
            match *tab {
                0 => {
                    ui.label(egui::RichText::new(tl!("General")).font(semibold(12.5)));
                    rtl::row(ui, |ui| {
                        ui.label(tl!("Direction:"));
                        rtl::radio_value(ui, rtl, true, tl!("Right-to-left"));
                        rtl::radio_value(ui, rtl, false, tl!("Left-to-right"));
                    });
                    const ALIGNS: [(&str, &str); 4] = [("left", "Left"), ("center", "Center"), ("right", "Right"), ("justify", "Justify")];
                    rtl::row(ui, |ui| {
                        ui.label(tl!("Alignment:"));
                        let shown = ALIGNS.iter().find(|(v, _)| v == align).map_or(align.as_str(), |(_, l)| tl!(l));
                        egui::ComboBox::from_id_salt("para_align").selected_text(shown).show_ui(ui, |ui| {
                            for (v, l) in ALIGNS {
                                ui.selectable_value(align, v.to_string(), tl!(l));
                            }
                        });
                    });
                    ui.label(egui::RichText::new(tl!("Indentation")).font(semibold(12.5)));
                    // Mirrored indents are the inside and outside ones (they swap on even pages).
                    let (l_label, r_label) = if opts[0] { ("Inside:", "Outside:") } else { ("Left:", "Right:") };
                    rtl::fitted(ui, "ind", |ui| {
                        rtl::grid("ind", 4).show(ui, |ui| {
                            rtl::grid_row(
                                ui,
                                [
                                    &mut |ui: &mut Ui| label(ui, l_label),
                                    &mut |ui: &mut Ui| {
                                        ui.add(length_value(left));
                                    },
                                    &mut |ui: &mut Ui| label(ui, r_label),
                                    &mut |ui: &mut Ui| {
                                        ui.add(length_value(right));
                                    },
                                ],
                            );
                            rtl::grid_row(
                                ui,
                                [
                                    &mut |ui: &mut Ui| label(ui, "First line:"),
                                    &mut |ui: &mut Ui| {
                                        ui.add(length_value(first));
                                    },
                                    &mut |_: &mut Ui| {},
                                    &mut |_: &mut Ui| {},
                                ],
                            );
                        });
                    });
                    rtl::checkbox(ui, &mut opts[0], tl!("Mirror indents"));
                    ui.label(egui::RichText::new(tl!("Spacing")).font(semibold(12.5)));
                    rtl::fitted(ui, "sp", |ui| {
                        rtl::grid("sp", 4).show(ui, |ui| {
                            rtl::grid_row(
                                ui,
                                [
                                    &mut |ui: &mut Ui| label(ui, "Before:"),
                                    &mut |ui: &mut Ui| {
                                        ui.add(points(egui::DragValue::new(before).speed(1.0).range(0.0..=1584.0)));
                                    },
                                    &mut |ui: &mut Ui| label(ui, "Line spacing:"),
                                    &mut |ui: &mut Ui| {
                                        ui.add(egui::DragValue::new(line).speed(0.05).range(0.5..=5.0));
                                    },
                                ],
                            );
                            rtl::grid_row(
                                ui,
                                [
                                    &mut |ui: &mut Ui| label(ui, "After:"),
                                    &mut |ui: &mut Ui| {
                                        ui.add(points(egui::DragValue::new(after).speed(1.0).range(0.0..=1584.0)));
                                    },
                                    &mut |_: &mut Ui| {},
                                    &mut |_: &mut Ui| {},
                                ],
                            );
                        });
                    });
                    rtl::checkbox(ui, &mut opts[1], tl!("Don't add space between paragraphs of the same style"));
                }
                1 => {
                    ui.label(egui::RichText::new(tl!("Line and Page Breaks")).font(semibold(12.5)));
                    rtl::checkbox(ui, widow, tl!("Widow/Orphan control"));
                    rtl::checkbox(ui, keep_next, tl!("Keep with next"));
                    rtl::checkbox(ui, keep_lines, tl!("Keep lines together"));
                    rtl::checkbox(ui, page_break, tl!("Page break before"));
                    ui.label(egui::RichText::new(tl!("Formatting exceptions")).font(semibold(12.5)));
                    rtl::checkbox(ui, &mut opts[2], tl!("Suppress line numbers"));
                    rtl::checkbox(ui, &mut opts[3], tl!("Don't hyphenate"));
                }
                _ => {
                    let [kinsoku, word_wrap, overflow, top_line, de, dn] = asian;
                    ui.label(egui::RichText::new(tl!("Line breaking")).font(semibold(12.5)));
                    rtl::checkbox(ui, kinsoku, tl!("Apply Asian line-breaking rules (kinsoku)"));
                    // The model's flag is "wrap whole words"; the box offers the opposite.
                    let mut mid_word = !*word_wrap;
                    if rtl::checkbox(ui, &mut mid_word, tl!("Allow Latin text to wrap in the middle of a word")).changed() {
                        *word_wrap = !mid_word;
                    }
                    rtl::checkbox(ui, overflow, tl!("Allow hanging punctuation"));
                    ui.label(egui::RichText::new(tl!("Character Spacing")).font(semibold(12.5)));
                    rtl::checkbox(ui, top_line, tl!("Compress punctuation at the start of a line"));
                    rtl::checkbox(ui, de, tl!("Add space between Asian and Latin text"));
                    rtl::checkbox(ui, dn, tl!("Add space between Asian text and numbers"));
                }
            }
            ui.add_space(6.0);
            let tabs = ui.button(tl!("Tabs…")).clicked();
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok || tabs {
                apply_paragraph(
                    app,
                    *rtl,
                    align,
                    *left,
                    *right,
                    *first,
                    *before,
                    *after,
                    (*line, *line_was),
                    [*keep_next, *keep_lines, *page_break, *widow],
                    (*opts, *opts_was),
                );
                apply_asian(app, *asian, *asian_was);
            }
            // Tabs… keeps what was set here and moves on to the Tabs dialog.
            if tabs {
                app.dialog = Dialog::open("tabs", app);
            }
            ok || cancel || tabs
        }
        Dialog::Tabs { form } => crate::dialogs_para::tabs(app, ui, form),
        Dialog::Borders { form } => crate::dialogs_para::borders(app, ui, form),
        Dialog::Find { query, replace, match_case, whole_word, regex, extra, tab, goto_what, goto_target, message, focus } => {
            rtl::row(ui, |ui| {
                for (t, label) in [(FindTab::Find, "Find"), (FindTab::Replace, "Replace"), (FindTab::GoTo, "Go To")] {
                    if ui.selectable_label(*tab == t, tl!(label)).clicked() && *tab != t {
                        *tab = t;
                        message.clear();
                    }
                }
            });
            ui.separator();
            if *tab == FindTab::GoTo {
                return goto_body(app, ui, goto_what, goto_target, message);
            }
            let replace_mode = *tab == FindTab::Replace;
            rtl::row(ui, |ui| {
                ui.label(tl!("Find what:"));
                let r = ui.add(egui::TextEdit::singleline(query).id(egui::Id::new("find_what")).desired_width(240.0));
                if std::mem::take(focus) {
                    r.request_focus();
                }
                if r.gained_focus() {
                    extra.to_replace = false;
                }
            });
            if replace_mode {
                rtl::row(ui, |ui| {
                    ui.label(tl!("Replace with:"));
                    if ui.add(egui::TextEdit::singleline(replace).desired_width(228.0)).gained_focus() {
                        extra.to_replace = true;
                    }
                });
            } else {
                extra.to_replace = false;
            }
            // Word greys out Match case and the word options with wildcards (they always match case).
            let plain = !extra.wildcards;
            rtl::row(ui, |ui| {
                rtl::checkbox_enabled(ui, plain, match_case, tl!("Match case"));
                rtl::checkbox_enabled(ui, plain, whole_word, tl!("Whole words"));
                rtl::checkbox(ui, &mut extra.wildcards, tl!("Use wildcards"));
            });
            // The disclosure arrow leads the label: at its right, pointing left, under RTL.
            let options = if rtl::is_rtl() {
                format!("{} {}", tl!("Search Options"), if extra.more { "▾" } else { "◂" })
            } else {
                format!("{} {}", if extra.more { "▾" } else { "▸" }, tl!("Search Options"))
            };
            if ui.selectable_label(extra.more, options).clicked() {
                extra.more = !extra.more;
            }
            if extra.more {
                rtl::row(ui, |ui| {
                    rtl::checkbox_enabled(ui, plain, &mut extra.prefix, tl!("Match prefix"));
                    rtl::checkbox_enabled(ui, plain, &mut extra.suffix, tl!("Match suffix"));
                });
                rtl::row(ui, |ui| {
                    rtl::checkbox_enabled(ui, plain, &mut extra.ignore_punct, tl!("Ignore punctuation characters"));
                    rtl::checkbox_enabled(ui, plain, &mut extra.ignore_space, tl!("Ignore white-space characters"));
                });
                rtl::row(ui, |ui| {
                    rtl::checkbox(ui, regex, tl!("Regular expressions"));
                    if let Some(code) = special_menu(ui, extra) {
                        if extra.to_replace { replace.push_str(code) } else { query.push_str(code) }
                    }
                });
            }
            let opts = json!({
                "text": query,
                "with": replace,
                "matchCase": *match_case,
                "wholeWord": *whole_word,
                "regex": *regex,
                "wildcards": extra.wildcards,
                "matchPrefix": extra.prefix,
                "matchSuffix": extra.suffix,
                "ignorePunct": extra.ignore_punct,
                "ignoreSpace": extra.ignore_space,
            });
            if !replace_mode {
                rtl::row(ui, |ui| {
                    let lit = app.session.find.highlight;
                    if ui.selectable_label(lit, tl!("Reading Highlight")).clicked() {
                        let mut v = opts.clone();
                        v["highlight"] = json!(!lit);
                        match app.run("edit.advancedFind", v) {
                            Ok(r) if !lit => *message = crate::i18n::fmt(tl!("{count} items highlighted"), &[("count", &r["count"].to_string())]),
                            Ok(_) => message.clear(),
                            Err(e) => *message = e,
                        }
                    }
                });
            }
            if !message.is_empty() {
                ui.label(egui::RichText::new(message.as_str()).weak());
            }
            let mut close = false;
            rtl::row(ui, |ui| {
                if replace_mode {
                    if ui.button(tl!("Replace All")).clicked() {
                        match app.run("edit.replaceAll", opts.clone()) {
                            Ok(r) => *message = format!("All done. We made {} replacements.", r["replaced"]),
                            Err(e) => *message = e,
                        }
                    }
                    if ui.button(tl!("Replace")).clicked() {
                        let _ = app.run("edit.replace", opts.clone());
                    }
                }
                if ui.button(tl!("Find Next")).clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // One step per click: `edit.find` would select a match itself, and the step after it skip one.
                    match app.run("edit.findNext", opts.clone()) {
                        Ok(r) => *message = format!("Match {} of {}", r["index"].as_u64().unwrap_or(0) + 1, r["count"]),
                        Err(e) => *message = e,
                    }
                }
                if ui.button(tl!("Close")).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    close = true;
                }
            });
            close
        }
        Dialog::Enclose { style, text, shape } => {
            ui.label(egui::RichText::new(tl!("Style")).font(semibold(12.5)));
            rtl::row(ui, |ui| {
                for (v, l) in [("none", "None"), ("shrink", "Shrink text"), ("enlarge", "Enlarge symbol")] {
                    rtl::radio_value(ui, style, v.to_string(), tl!(l));
                }
            });
            ui.add_space(6.0);
            // In the right-aligned column `horizontal_top` already runs right to left.
            ui.horizontal_top(|ui| {
                rtl::column(ui, |ui| {
                    ui.label(tl!("Text:"));
                    ui.add(egui::TextEdit::singleline(text).desired_width(110.0).char_limit(2));
                    ui.add_space(6.0);
                    enclose_preview(ui, style, text, shape);
                });
                ui.add_space(12.0);
                rtl::column(ui, |ui| {
                    ui.label(tl!("Enclosure"));
                    for (v, l) in [("circle", "Circle"), ("square", "Square"), ("triangle", "Triangle"), ("diamond", "Diamond")] {
                        ui.add_enabled_ui(style != "none", |ui| ui.selectable_value(shape, v.to_string(), tl!(l)));
                    }
                });
            });
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let params = if style == "none" { json!({"shape": null}) } else { json!({"shape": shape, "style": style, "text": text}) };
                let _ = app.run("format.enclose", params);
            }
            ok || cancel
        }
        Dialog::TableProperties { form, basis } => {
            table_properties(ui, form);
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let changes = form.changes(basis);
                if changes.as_object().is_some_and(|m| !m.is_empty()) {
                    let _ = app.run("table.properties", changes);
                }
            }
            ok || cancel
        }
        Dialog::InsertTable { rows, cols } => {
            rtl::fitted(ui, "it", |ui| {
                rtl::grid("it", 2).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Number of columns:"), &mut |ui: &mut Ui| {
                            ui.add(egui::DragValue::new(cols).range(1..=63));
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Number of rows:"), &mut |ui: &mut Ui| {
                            ui.add(egui::DragValue::new(rows).range(1..=1000));
                        }],
                    );
                });
            });
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let _ = app.run("insert.table", json!({"rows": *rows, "cols": *cols}));
            }
            ok || cancel
        }
        Dialog::PageSetup { top, bottom, left, right, landscape, rtl, grid, grid_was } => {
            ui.label(egui::RichText::new(tl!("Margins")).font(semibold(12.5)));
            rtl::fitted(ui, "ps", |ui| {
                rtl::grid("ps", 4).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [
                            &mut |ui: &mut Ui| label(ui, "Top:"),
                            &mut |ui: &mut Ui| {
                                ui.add(length_value(top));
                            },
                            &mut |ui: &mut Ui| label(ui, "Bottom:"),
                            &mut |ui: &mut Ui| {
                                ui.add(length_value(bottom));
                            },
                        ],
                    );
                    rtl::grid_row(
                        ui,
                        [
                            &mut |ui: &mut Ui| label(ui, "Left:"),
                            &mut |ui: &mut Ui| {
                                ui.add(length_value(left));
                            },
                            &mut |ui: &mut Ui| label(ui, "Right:"),
                            &mut |ui: &mut Ui| {
                                ui.add(length_value(right));
                            },
                        ],
                    );
                });
            });
            ui.label(egui::RichText::new(tl!("Orientation")).font(semibold(12.5)));
            rtl::row(ui, |ui| {
                rtl::radio_value(ui, landscape, false, tl!("Portrait"));
                rtl::radio_value(ui, landscape, true, tl!("Landscape"));
            });
            ui.label(egui::RichText::new(tl!("Document Grid")).font(semibold(12.5)));
            // In the right-aligned column `horizontal_wrapped` already wraps right to left.
            ui.horizontal_wrapped(|ui| {
                rtl::radio_value(ui, &mut grid.0, 0, tl!("No grid"));
                rtl::radio_value(ui, &mut grid.0, 1, tl!("Lines only"));
                rtl::radio_value(ui, &mut grid.0, 2, tl!("Lines and characters"));
                rtl::radio_value(ui, &mut grid.0, 3, tl!("Text snaps to characters"));
            });
            let kind = grid.0;
            rtl::fitted(ui, "ps_grid", |ui| {
                rtl::grid("ps_grid", 4).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [
                            &mut |ui: &mut Ui| {
                                ui.add_enabled(kind >= 2, egui::Label::new(tl!("Characters per line:")));
                            },
                            &mut |ui: &mut Ui| {
                                ui.add_enabled(kind >= 2, egui::DragValue::new(&mut grid.2).range(1..=1000));
                            },
                            &mut |ui: &mut Ui| {
                                ui.add_enabled(kind >= 1, egui::Label::new(tl!("Lines per page:")));
                            },
                            &mut |ui: &mut Ui| {
                                ui.add_enabled(kind >= 1, egui::DragValue::new(&mut grid.1).range(1..=1000));
                            },
                        ],
                    );
                });
            });
            rtl::row(ui, |ui| {
                ui.label(tl!("Section direction:"));
                rtl::radio_value(ui, rtl, false, tl!("Left-to-right"));
                rtl::radio_value(ui, rtl, true, tl!("Right-to-left"));
            });
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let _ = app.run("layout.orientation", json!({"value": if *landscape { "landscape" } else { "portrait" }}));
                let _ =
                    app.run("layout.margins", json!({"top": *top * crate::options::unit().pt_per_unit(), "bottom": *bottom * crate::options::unit().pt_per_unit(), "left": *left * crate::options::unit().pt_per_unit(), "right": *right * crate::options::unit().pt_per_unit()}));
                if grid != grid_was {
                    let kind = ["none", "lines", "linesAndChars", "snapToChars"].get(grid.0 as usize).copied().unwrap_or("none");
                    let mut v = json!({"type": kind});
                    if grid.1 != grid_was.1 {
                        v["linesPerPage"] = json!(grid.1);
                    }
                    if grid.2 != grid_was.2 {
                        v["charsPerLine"] = json!(grid.2);
                    }
                    let _ = app.run("layout.documentGrid", v);
                }
                if *rtl != wordcraft_engine::cmd::page::sect(&app.session).rtl {
                    let _ = app.run("layout.sectionRtl", json!({"value": *rtl}));
                }
            }
            ok || cancel
        }
        Dialog::Link { url, text } => {
            rtl::fitted(ui, "lnk", |ui| {
                rtl::grid("lnk", 2).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Text to display:"), &mut |ui: &mut Ui| {
                            text_field(ui, text, None);
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Address:"), &mut |ui: &mut Ui| {
                            text_field(ui, url, None);
                        }],
                    );
                });
            });
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let _ = app.run("insert.link", json!({"url": url, "text": text}));
            }
            ok || cancel
        }
        Dialog::Bookmark { name } => {
            ui.label(tl!("Bookmark name:"));
            ui.text_edit_singleline(name);
            let marks = app.session.doc.bookmarks();
            for (n, _) in &marks {
                if ui.selectable_label(false, n).clicked() {
                    let _ = app.run("edit.goto", json!({"bookmark": n}));
                }
            }
            let (ok, cancel) = buttons(ui, tl!("Add"));
            if ok {
                let _ = app.run("insert.bookmark", json!({"name": name}));
            }
            ok || cancel
        }
        Dialog::WordCount { stats } => {
            rtl::fitted(ui, "wc", |ui| {
                rtl::grid("wc", 2).spacing(vec2(30.0, 6.0)).show(ui, |ui| {
                    for (l, k) in [
                        ("Pages", "pages"),
                        ("Words", "words"),
                        ("Characters (no spaces)", "characters"),
                        ("Characters (with spaces)", "charactersWithSpaces"),
                        ("Paragraphs", "paragraphs"),
                        ("Lines", "lines"),
                    ] {
                        rtl::grid_row(
                            ui,
                            [&mut |ui: &mut Ui| label(ui, l), &mut |ui: &mut Ui| {
                                ui.label(stats.get(k).map(|v| v.to_string()).unwrap_or_default());
                            }],
                        );
                    }
                });
            });
            let mut include = stats.get("includeTextBoxes").and_then(Value::as_bool).unwrap_or(true);
            if rtl::checkbox(ui, &mut include, tl!("Count words in text boxes and notes")).changed()
                && let Ok(v) = app.run("review.wordCount", json!({"includeTextBoxes": include}))
            {
                *stats = v;
            }
            let (ok, cancel) = buttons(ui, tl!("Close"));
            ok || cancel
        }
        Dialog::Zoom { percent } => {
            rtl::row(ui, |ui| {
                for p in [200.0, 100.0, 75.0] {
                    rtl::radio_value(ui, percent, p, format!("{p}%"));
                }
            });
            // Logarithmic like the status bar slider, and wide enough to aim: the default 100 pt
            // linear slider jumped from 100% to 480% within a short drag (issue #67).
            ui.spacing_mut().slider_width = 220.0;
            ui.add(egui::Slider::new(percent, 10.0..=500.0).logarithmic(true).step_by(1.0).suffix("%"));
            let mut fit = None;
            rtl::row(ui, |ui| {
                if ui.button(tl!("Page width")).clicked() {
                    fit = Some("pageWidth");
                }
                if ui.button(tl!("Whole page")).clicked() {
                    fit = Some("onePage");
                }
                if ui.button(tl!("Many pages")).clicked() {
                    fit = Some("multiplePages");
                }
            });
            if let Some(f) = fit {
                let _ = app.run("view.zoom", json!({"value": f}));
                return true;
            }
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let _ = app.run("view.zoom", json!({"value": *percent}));
            }
            ok || cancel
        }
        Dialog::Watermark { text, diagonal } => {
            rtl::row(ui, |ui| {
                ui.label(tl!("Text:"));
                ui.text_edit_singleline(text);
            });
            rtl::row(ui, |ui| {
                rtl::radio_value(ui, diagonal, true, tl!("Diagonal"));
                rtl::radio_value(ui, diagonal, false, tl!("Horizontal"));
            });
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let _ = app.run("design.watermark", json!({"text": text, "diagonal": *diagonal}));
            }
            ok || cancel
        }
        Dialog::Phonetic { base, ruby, align, size, offset, existing } => {
            // Base and ruby text are usually Chinese or Japanese: fonts for them load from the next frame.
            app.want_system_cjk = true;
            const ALIGNS: [(&str, &str); 6] = [
                ("center", "Center"),
                ("distributeLetter", "Distribute letters"),
                ("distributeSpace", "Distribute spacing"),
                ("left", "Left"),
                ("right", "Right"),
                ("rightVertical", "Right (vertical)"),
            ];
            rtl::fitted(ui, "phonetic", |ui| {
                rtl::grid("phonetic", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Base text:"), &mut |ui: &mut Ui| {
                            ui.label(egui::RichText::new(crate::i18n::isolate(base)).strong());
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Ruby text:"), &mut |ui: &mut Ui| {
                            text_field(ui, ruby, None);
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Alignment:"), &mut |ui: &mut Ui| {
                            let shown = ALIGNS.iter().find(|(k, _)| k == align).map_or("Center", |(_, l)| l);
                            egui::ComboBox::from_id_salt("phonetic_align").selected_text(tl!(shown)).show_ui(ui, |ui| {
                                for (k, l) in ALIGNS {
                                    ui.selectable_value(align, k.to_string(), tl!(l));
                                }
                            });
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Offset (pt):"), &mut |ui: &mut Ui| {
                            ui.add(egui::DragValue::new(offset).speed(0.5).range(-100.0..=100.0));
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Size (pt):"), &mut |ui: &mut Ui| {
                            ui.add(egui::DragValue::new(size).speed(0.5).range(1.0..=200.0));
                        }],
                    );
                });
            });
            let mut remove = false;
            if *existing {
                ui.add_space(4.0);
                remove = ui.button(tl!("Remove")).clicked();
            }
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if remove {
                let _ = app.run("format.phonetic", json!({"ruby": null}));
            } else if ok {
                let _ = app.run("format.phonetic", json!({"ruby": ruby, "align": align, "size": *size, "offset": *offset}));
            }
            ok || cancel || remove
        }
        Dialog::NewStyle { name, based_on, back } => {
            rtl::fitted(ui, "ns", |ui| {
                rtl::grid("ns", 2).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Name:"), &mut |ui: &mut Ui| {
                            text_field(ui, name, None);
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Style based on:"), &mut |ui: &mut Ui| {
                            text_field(ui, based_on, None);
                        }],
                    );
                });
            });
            ui.label(egui::RichText::new(tl!("The new style takes the formatting of the current paragraph.")).small().weak());
            let (ok, cancel) = buttons(ui, tl!("OK"));
            let mut created = None;
            if ok {
                created =
                    app.run("styles.create", json!({"name": name, "basedOn": based_on})).ok().and_then(|v| v.get("id")?.as_str().map(str::to_string));
            }
            if (ok || cancel) && *back {
                app.dialog = Some(Dialog::ManageStyles { alphabetical: false, selected: created.unwrap_or_default() });
            }
            ok || cancel
        }
        Dialog::ModifyStyle { back, resume, id, name, font, size, bold, italic, color, before, after } => {
            rtl::fitted(ui, "ms", |ui| {
                rtl::grid("ms", 2).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Name:"), &mut |ui: &mut Ui| {
                            text_field(ui, name, None);
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Font:"), &mut |ui: &mut Ui| {
                            text_field(ui, font, None);
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Size:"), &mut |ui: &mut Ui| {
                            ui.add(egui::DragValue::new(size).range(1.0..=1638.0).speed(0.5));
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Color:"), &mut |ui: &mut Ui| {
                            text_field(ui, color, None);
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Space before/after:"), &mut |ui: &mut Ui| {
                            rtl::row(ui, |ui| {
                                ui.add(egui::DragValue::new(before).range(0.0..=1584.0));
                                ui.add(egui::DragValue::new(after).range(0.0..=1584.0));
                            });
                        }],
                    );
                });
            });
            rtl::row(ui, |ui| {
                rtl::checkbox(ui, bold, tl!("Bold"));
                rtl::checkbox(ui, italic, tl!("Italic"));
            });
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let mut chr = json!({"font": font, "size": *size, "bold": *bold, "italic": *italic});
                if let Some(c) = wordcraft_doc::Rgb::parse(color) {
                    chr["color"] = json!({"Rgb": [c.0, c.1, c.2]});
                }
                let _ =
                    app.run("styles.modify", json!({"style": id, "name": name, "chr": chr, "para": {"spaceBefore": *before, "spaceAfter": *after}}));
            }
            if (ok || cancel) && *back {
                app.dialog = Some(Dialog::ManageStyles { alphabetical: false, selected: id.clone() });
            }
            if (ok || cancel)
                && let Some(r) = resume.take()
            {
                app.dialog = Some(*r);
            }
            ok || cancel
        }
        Dialog::ManageStyles { alphabetical, selected } => manage_styles(app, ui, alphabetical, selected),
        Dialog::TableStyle { id, name, based_on, region, regions, basis } => {
            let styles: Vec<(String, String)> = app
                .session
                .doc
                .styles
                .styles
                .iter()
                .filter(|s| s.kind == wordcraft_doc::StyleKind::Table && Some(&s.id) != id.as_ref())
                .map(|s| (s.id.clone(), s.name.clone()))
                .collect();
            let base_name = styles.iter().find(|(i, _)| i == based_on).map(|(_, n)| n.clone()).unwrap_or_else(|| based_on.clone());
            let mut rebased = false;
            rtl::fitted(ui, "tstyle", |ui| {
                rtl::grid("tstyle", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Name:"), &mut |ui: &mut Ui| {
                            text_field(ui, name, None);
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Style based on:"), &mut |ui: &mut Ui| {
                            egui::ComboBox::from_id_salt("tstyle_base").width(200.0).selected_text(base_name.as_str()).show_ui(ui, |ui| {
                                for (sid, sname) in &styles {
                                    if ui.selectable_label(sid == based_on, sname).clicked() && sid != based_on {
                                        *based_on = sid.clone();
                                        rebased = true;
                                    }
                                }
                            });
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Apply formatting to:"), &mut |ui: &mut Ui| {
                            egui::ComboBox::from_id_salt("tstyle_region")
                                .width(200.0)
                                .selected_text(tl!(REGION_LABELS.get(*region).copied().unwrap_or("Whole Table")))
                                .show_ui(ui, |ui| {
                                    for (i, n) in REGION_LABELS.iter().enumerate() {
                                        ui.selectable_value(region, i, tl!(n));
                                    }
                                });
                        }],
                    );
                });
            });
            // A new style shows its new base's formatting; a modified one keeps its own.
            if rebased && id.is_none() {
                *regions = table_regions(app, based_on);
                *basis = regions.clone();
            }
            ui.separator();
            let theme = app.session.doc.settings.theme_colors.clone();
            if let Some(r) = regions.get_mut(*region) {
                rtl::row(ui, |ui| {
                    rtl::checkbox(ui, &mut r.borders, tl!("Borders"));
                    rtl::checkbox(ui, &mut r.bold, tl!("Bold"));
                });
                rtl::fitted(ui, "tstyle_colors", |ui| {
                    rtl::grid("tstyle_colors", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                        rtl::grid_row(ui, [&mut |ui: &mut Ui| label(ui, "Fill color:"), &mut |ui: &mut Ui| color_menu(ui, &theme, &mut r.fill)]);
                        rtl::grid_row(ui, [&mut |ui: &mut Ui| label(ui, "Font color:"), &mut |ui: &mut Ui| color_menu(ui, &theme, &mut r.color)]);
                    });
                });
            }
            // Live preview: the style as OK would leave it, on the current table's options with
            // the region being edited turned on.
            let v = table_style_params(id.as_deref(), name, based_on, regions, basis);
            let mut look = app
                .session
                .sel
                .focus
                .path
                .cell()
                .and_then(|(tp, _, _)| app.session.doc.table(app.session.sel.focus.story, &tp).map(|t| t.props.look))
                .unwrap_or_default();
            match *region {
                1 => look.header_row = true,
                2 => look.total_row = true,
                3 => look.first_column = true,
                4 => look.last_column = true,
                5 => look.banded_rows = true,
                6 => look.banded_columns = true,
                _ => {}
            }
            ui.add_space(4.0);
            ui.label(egui::RichText::new(tl!("Preview")).small().weak());
            table_style_preview(ui, wordcraft_engine::cmd::table_style::preview(&app.session, &v).as_ref(), look);
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                let cmd = if id.is_some() { "table.modifyStyle" } else { "table.newStyle" };
                if let Err(e) = app.run(cmd, v) {
                    app.status(e);
                    return false;
                }
            }
            ok || cancel
        }
        Dialog::Insert(f) => crate::dialogs_insert::body(app, ui, f),
        Dialog::Refs(f) => crate::dialogs_refs::body(app, ui, f),
        Dialog::Commands { query, swallow } => {
            if *swallow {
                ui.ctx().input_mut(|i| i.events.retain(|e| !swallow_shortcut_text(swallow, e)));
            }
            let r = ui.add(egui::TextEdit::singleline(query).hint_text(tl!("Type a command, e.g. \"insert table\"")).desired_width(380.0));
            r.request_focus();
            let reg = app.session.registry.clone();
            // An empty box suggests a few common commands, as Word's Search does.
            let mut hits: Vec<&wordcraft_engine::CommandSpec> = if query.trim().is_empty() {
                ui.label(egui::RichText::new(tl!("Suggested")).font(semibold(12.5)));
                SEARCH_SUGGESTIONS.iter().filter_map(|id| reg.get(id)).collect()
            } else {
                reg.all().iter().filter(|c| command_matches(c, query)).collect()
            };
            hits.truncate(14);
            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let mut run: Option<&str> = enter.then(|| hits.first().map(|c| c.id)).flatten();
            for c in &hits {
                let sc = crate::widgets::shortcut_text(app, c.id);
                // Both parts are catalog text in draw order: under RTL the command reads first, at the right.
                let (label, location) = (tl!(c.label), crate::i18n::location(c.location));
                let label = if rtl::is_rtl() { format!("{location}   —   {label}") } else { format!("{label}   —   {location}") };
                if ui.add(rtl::shortcut_item(label, sc).min_size(vec2(380.0, 0.0))).clicked() {
                    run = Some(c.id);
                }
            }
            // Search also finds the words in the document (the Navigation pane's results).
            let text = query.trim().to_string();
            let mut find = false;
            if !text.is_empty() {
                ui.separator();
                let label = crate::i18n::fmt(tl!("Find “{text}” in the document"), &[("text", &text)]);
                find = ui.add(egui::Button::new(label).min_size(vec2(380.0, 0.0))).clicked() || (enter && hits.is_empty());
            }
            if let Some(id) = run {
                let _ = app.run(id, json!({}));
            } else if find {
                find_in_navigation_pane(app, ui.ctx(), &text);
            }
            run.is_some() || find || ui.input(|i| i.key_pressed(egui::Key::Escape))
        }
        Dialog::Icons { query, category, selected } => icons_body(app, ui, query, category, selected),
        Dialog::PasteSpecial { formats, choice, payload, note } => {
            ui.label(egui::RichText::new(tl!("Paste as:")).font(semibold(12.5)));
            if formats.is_empty() {
                // The engine's wording, sentence-cased; the format names in it aren't translated.
                let text = if note.is_empty() || note == "the clipboard is empty" {
                    tl!("The clipboard is empty.").to_string()
                } else {
                    let mut c = note.chars();
                    c.next().map(|f| f.to_uppercase().chain(c).collect::<String>() + ".").unwrap_or_default()
                };
                ui.label(egui::RichText::new(text).weak());
            }
            for (i, f) in formats.iter().enumerate() {
                let label = wordcraft_engine::cmd::paste::FORMATS.iter().find(|(id, _)| id == f).map(|(_, l)| *l).unwrap_or(f.as_str());
                rtl::radio_value(ui, choice, i, tl!(label));
            }
            if let Some(f) = formats.get(*choice) {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(tl!("Result")).font(semibold(12.5)));
                let hint = match f.as_str() {
                    "formatted" => "Keeps the fonts and formatting of the copied text.",
                    "rtf" => "Reads the clipboard's rich text and keeps its formatting.",
                    "html" => "Reads the clipboard's web content and keeps its formatting.",
                    "picture" => "Pastes the clipboard's picture.",
                    _ => "Drops all formatting and pastes plain text.",
                };
                rtl::wrapped_label(ui, tl!(hint), |s| egui::RichText::new(s).weak());
            }
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok && let Some(f) = formats.get(*choice) {
                let mut params = payload.clone();
                params["as"] = json!(f);
                let _ = app.run("edit.pasteSpecial", params);
            }
            ok || cancel
        }
        Dialog::About { tab } => {
            ui.set_width(660.0);
            tab_strip(ui, tab, &["About", "Contributors", "Models"]);
            ui.separator();
            match *tab {
                // The credit tables (egui grids of names and numbers) keep their left-to-right
                // columns, in a box of their own width (at the right under RTL).
                1 => rtl::fitted(ui, "credits_contributors", crate::credits::contributors_ui),
                2 => rtl::fitted(ui, "credits_models", crate::credits::models_ui),
                _ => {
                    ui.label(egui::RichText::new(tl!("WordCraft")).font(semibold(22.0)));
                    // The build is a commit id or a catalog string (already in draw order).
                    let build = option_env!("WORDCRAFT_BUILD_SHA")
                        .map(|s| crate::i18n::isolate(s.get(..8).unwrap_or(s)))
                        .unwrap_or_else(|| tl!("development build").to_string());
                    let version = crate::i18n::isolate(env!("CARGO_PKG_VERSION"));
                    ui.label(crate::i18n::fmt_translated(tl!("Version {version} ({build})"), &[("version", &version), ("build", &build)]));
                    ui.label(tl!(
                        "A free, open-source word processor written from scratch in Rust.\nPart of the Crafting Apps from the ArtCraft team."
                    ));
                    ui.add_space(6.0);
                    ui.hyperlink_to(tl!("getartcraft.com/apps/wordcraft"), "https://getartcraft.com/apps/wordcraft");
                    ui.hyperlink_to(tl!("Join us on Discord: discord.gg/artcraft"), "https://discord.gg/artcraft");
                    ui.hyperlink_to(tl!("Source code: github.com/storytold/wordcraft"), "https://github.com/storytold/wordcraft");
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(tl!("MIT OR Apache-2.0. Copyright (c) 2026 ArtCraft Team and the WordCraft contributors."))
                            .small()
                            .weak(),
                    );
                }
            }
            let (ok, cancel) = buttons(ui, tl!("Close"));
            ok || cancel
        }
        Dialog::SaveChanges { name, .. } => {
            ui.label(egui::RichText::new(crate::i18n::fmt(tl!("Do you want to save changes to {name}?"), &[("name", name)])).font(semibold(15.0)));
            ui.label(tl!("Your changes will be lost if you don't save them."));
            ui.add_space(8.0);
            let buttons = ui.horizontal(|ui| {
                // At the end edge; Save is the rightmost under RTL.
                ui.with_layout(rtl::end_layout(egui::Align::Center), |ui| {
                    let cancel = ui.button(tl!("Cancel"));
                    let dont_save = ui.button(tl!("Don't Save"));
                    let save = ui.add(crate::widgets::primary_button(ui, tl!("Save")));
                    [(cancel, "cancel"), (dont_save, "dontSave"), (save, "save")]
                })
                .inner
            });
            let buttons = buttons.inner;
            // The keyboard alone answers, as in Word (#861): Save has focus when the prompt opens,
            // so Enter saves, and the arrow keys (egui's focus navigation) or Tab move between the
            // buttons, shown as Save, Don't Save, Cancel.
            if ui.memory(|m| m.focused().is_none())
                && let Some((save, _)) = buttons.get(2)
            {
                save.request_focus();
            }
            // Enter clicks the focused button; it means Save only when none of them has focus.
            let focused = buttons.iter().any(|(b, _)| b.has_focus());
            let answer = if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                Some("cancel")
            } else if let Some((_, a)) = buttons.iter().find(|(b, _)| b.clicked()) {
                Some(*a)
            } else if !focused && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                Some("save")
            } else {
                None
            };
            let Some(answer) = answer else { return false };
            // `ui.saveChanges` reads the prompt from `app.dialog`, which `show` has taken out.
            app.dialog = Some(d.clone());
            let _ = app.run("ui.saveChanges", json!({"answer": answer}));
            true
        }
        Dialog::RecipientList { fields, rows, message } => recipient_list(app, ui, fields, rows, message),
        Dialog::InsertMergeField { fields, field } => {
            if fields.is_empty() {
                ui.label(egui::RichText::new(tl!("Select recipients first to choose from their fields.")).small().weak());
            } else {
                ui.label(tl!("Fields:"));
                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    for f in fields.iter() {
                        let resp = ui.selectable_label(field == f, f);
                        if resp.clicked() {
                            field.clone_from(f);
                        }
                    }
                });
            }
            rtl::row(ui, |ui| {
                ui.label(tl!("Field:"));
                ui.text_edit_singleline(field);
            });
            let (ok, cancel) = buttons(ui, tl!("Insert"));
            if ok && !field.trim().is_empty() {
                let _ = app.run("mailings.insertField", json!({"field": field.trim()}));
                return true;
            }
            cancel
        }
        Dialog::Password { name, params, password, message } => {
            ui.label(crate::i18n::fmt(tl!("{name} is protected with a password."), &[("name", name)]));
            ui.add_space(4.0);
            rtl::row(ui, |ui| {
                ui.label(tl!("Password:"));
                let r = ui.add(egui::TextEdit::singleline(password.as_mut_string()).password(true).desired_width(220.0));
                if password.is_empty() && !r.has_focus() {
                    r.request_focus();
                }
            });
            if !message.is_empty() {
                ui.label(egui::RichText::new(message.as_str()).small().color(ui.visuals().error_fg_color));
            }
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok && !password.is_empty() {
                let mut p = params.clone();
                if let Some(o) = p.as_object_mut() {
                    o.insert("password".into(), json!(password.as_str()));
                }
                // `execute`, not `run`: Save Changes was asked when the open began.
                match app.execute("file.open", p) {
                    Ok(_) => {
                        app.ui.backstage = false;
                        return true;
                    }
                    Err(e) if wordcraft_engine::io::wrong_password(&e) => {
                        *message = tl!("That password isn't right. Check it (passwords are case-sensitive) and try again.").to_string();
                        password.as_mut_string().clear();
                    }
                    Err(e) => *message = e,
                }
            }
            cancel
        }
        Dialog::EncryptPassword { password, confirm, message } => {
            ui.label(tl!("Anyone who opens the document will need this password."));
            ui.add_space(4.0);
            rtl::fitted(ui, "encrypt_password", |ui| {
                rtl::grid("encrypt_password", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                    let empty = confirm.is_empty();
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Password:"), &mut |ui: &mut Ui| {
                            cell_width(ui, 220.0);
                            let r = ui.add(egui::TextEdit::singleline(password.as_mut_string()).password(true).desired_width(220.0));
                            if password.is_empty() && empty && !r.has_focus() {
                                r.request_focus();
                            }
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Confirm password:"), &mut |ui: &mut Ui| {
                            cell_width(ui, 220.0);
                            ui.add(egui::TextEdit::singleline(confirm.as_mut_string()).password(true).desired_width(220.0));
                        }],
                    );
                });
            });
            rtl::wrapped_label(
                ui,
                tl!(
                    "Keep the password somewhere safe: without it nobody, WordCraft included, can open the document. Leave both boxes empty to remove the password."
                ),
                |s| egui::RichText::new(s).small().weak(),
            );
            if !message.is_empty() {
                ui.label(egui::RichText::new(message.as_str()).small().color(ui.visuals().error_fg_color));
            }
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok {
                if password != confirm {
                    *message = tl!("The passwords don't match.").to_string();
                    confirm.as_mut_string().clear();
                    return false;
                }
                let pw = if password.is_empty() { Value::Null } else { json!(password.as_str()) };
                match app.run("file.encrypt", json!({"password": pw})) {
                    Ok(_) => {
                        app.status(if password.is_empty() {
                            tl!("The password is removed. Save the document to keep the change.")
                        } else {
                            tl!("The document will be saved with a password. Save it to protect it.")
                        });
                        return true;
                    }
                    Err(e) => *message = e,
                }
            }
            cancel
        }
        Dialog::DefineList { form } => crate::dialogs_lists::define_list(app, ui, form),
        Dialog::TrackOptions { form } => crate::dialogs_lists::track_options(app, ui, form),
        Dialog::Review(d) => crate::dialogs_review::body(app, ui, d),
        Dialog::CustomizeKeyboard { form } => crate::dialogs_keyboard::customize_keyboard(app, ui, form),
        Dialog::CustomizeRibbon { form } => crate::dialogs_ribbon::customize_ribbon(app, ui, form),
        Dialog::FieldCode { code, edit, message } => {
            ui.label(tl!("Field codes:"));
            let r = ui.add(egui::TextEdit::singleline(code).desired_width(320.0).font(egui::TextStyle::Monospace));
            if !r.has_focus() && code.is_empty() {
                r.request_focus();
            }
            ui.label(egui::RichText::new("PAGE · NUMPAGES · DATE \\@ \"MMMM d, yyyy\" · AUTHOR · TITLE · SEQ Figure").small().weak());
            if !message.is_empty() {
                ui.label(egui::RichText::new(message.as_str()).small().weak());
            }
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok && !code.trim().is_empty() {
                let r = if *edit {
                    app.run("fields.setCode", json!({"code": code.trim()}))
                } else {
                    app.run("insert.field", json!({"instr": code.trim()}))
                };
                match r {
                    Ok(_) => return true,
                    Err(e) => *message = e,
                }
            }
            cancel
        }
        Dialog::Control { form } => crate::dialogs_controls::body(app, ui, form),
        Dialog::SelectTable { sheets, selected, headers, message, params } => {
            ui.label(tl!("Choose the sheet that holds the recipients."));
            ui.add_space(4.0);
            // A double-click picks the sheet as OK does.
            let mut picked = false;
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_width(300.0);
                egui::ScrollArea::vertical().id_salt("select_table").max_height(220.0).show(ui, |ui| {
                    for (i, name) in sheets.iter().enumerate() {
                        let r = ui.selectable_label(*selected == i, name.as_str());
                        if r.clicked() || r.double_clicked() {
                            *selected = i;
                        }
                        picked |= r.double_clicked();
                    }
                });
            });
            rtl::checkbox(ui, headers, tl!("First row of data contains column headers"));
            if !message.is_empty() {
                ui.label(egui::RichText::new(message.as_str()).color(Tokens::get(ui.ctx()).red));
            }
            let (ok, cancel) = buttons(ui, tl!("OK"));
            if ok || picked {
                let mut p = params.clone();
                if let Some(o) = p.as_object_mut() {
                    o.insert("sheet".into(), json!(*selected + 1));
                    o.insert("headers".into(), json!(*headers));
                }
                match app.load_recipients(p) {
                    Ok(_) => return true,
                    Err(e) => *message = e,
                }
            }
            cancel
        }
        Dialog::NoteOptions { form } => crate::dialogs_notes::note_options(app, ui, form),
        Dialog::Templates { form } => crate::dialogs_templates::templates(app, ui, form),
        Dialog::Organizer { form } => crate::dialogs_templates::organizer(app, ui, form),
        Dialog::Layout(d) => crate::dialogs_layout::body(app, ui, d),
        Dialog::FindRecipient { text, message } => {
            rtl::row(ui, |ui| {
                ui.label(tl!("Find:"));
                ui.text_edit_singleline(text);
            });
            if !message.is_empty() {
                ui.label(egui::RichText::new(message.as_str()).small().weak());
            }
            let (ok, cancel) = buttons(ui, tl!("Find"));
            if ok && !text.trim().is_empty() {
                match app.run("mailings.findRecipient", json!({"text": text.trim()})) {
                    Ok(_) => return true,
                    Err(e) => *message = e,
                }
            }
            cancel
        }
        Dialog::MergeRule { rule, fields, field, value, then, els } => {
            let skip = rule == "SKIPIF";
            ui.label(tl!(if skip {
                "Leave a recipient out of the merge when a field has this value."
            } else {
                "Insert one text or another, depending on a field's value."
            }));
            ui.add_space(4.0);
            rtl::fitted(ui, "merge_rule", |ui| {
                rtl::grid("merge_rule", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Field:"), &mut |ui: &mut Ui| {
                            if fields.is_empty() {
                                text_field(ui, field, None);
                            } else {
                                egui::ComboBox::from_id_salt("merge_rule_field").selected_text(field.as_str()).width(200.0).show_ui(ui, |ui| {
                                    for f in fields.iter() {
                                        ui.selectable_value(field, f.clone(), f);
                                    }
                                });
                            }
                        }],
                    );
                    rtl::grid_row(
                        ui,
                        [&mut |ui: &mut Ui| label(ui, "Equals:"), &mut |ui: &mut Ui| {
                            text_field(ui, value, None);
                        }],
                    );
                    if !skip {
                        rtl::grid_row(
                            ui,
                            [&mut |ui: &mut Ui| label(ui, "Then insert:"), &mut |ui: &mut Ui| {
                                text_field(ui, then, None);
                            }],
                        );
                        rtl::grid_row(
                            ui,
                            [&mut |ui: &mut Ui| label(ui, "Otherwise insert:"), &mut |ui: &mut Ui| {
                                text_field(ui, els, None);
                            }],
                        );
                    }
                });
            });
            let (ok, cancel) = buttons(ui, tl!("OK"));
            let params = rule_params(d);
            if ok && let Some(params) = params {
                let _ = app.run("mailings.rules", params);
                return true;
            }
            cancel
        }
        Dialog::MatchFields { fields, address } => {
            if fields.is_empty() {
                ui.label(egui::RichText::new(tl!("Select recipients first to match their fields.")).small().weak());
            } else {
                rtl::wrapped_label(ui, tl!("The recipient-list field that fills each part of an address block and greeting line."), |s| {
                    egui::RichText::new(s)
                });
                ui.add_space(4.0);
                rtl::fitted(ui, "match_fields", |ui| {
                    rtl::grid("match_fields", 2).spacing(vec2(24.0, 6.0)).striped(true).show(ui, |ui| {
                        for (part, key) in ADDRESS_PARTS {
                            rtl::grid_row(
                                ui,
                                [&mut |ui: &mut Ui| label(ui, part), &mut |ui: &mut Ui| {
                                    match address.get(key).and_then(Value::as_str) {
                                        Some(f) => ui.label(egui::RichText::new(crate::i18n::isolate(f)).strong()),
                                        None => ui.label(egui::RichText::new(tl!("(not matched)")).weak()),
                                    };
                                }],
                            );
                        }
                    });
                });
            }
            let (ok, cancel) = buttons(ui, tl!("Close"));
            ok || cancel
        }
        Dialog::CheckErrors { unknown, records } => {
            if unknown.is_empty() {
                ui.label(tl!("No errors found: every merge field is in the recipient list."));
            } else {
                ui.label(tl!("These merge fields aren't in the recipient list:"));
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for f in unknown.iter() {
                        // The bullet leads: at the right under RTL.
                        let line = if rtl::is_rtl() { format!("{} •", crate::i18n::isolate(f)) } else { format!("• {f}") };
                        ui.label(egui::RichText::new(line).color(Tokens::get(ui.ctx()).red));
                    }
                });
            }
            ui.add_space(4.0);
            ui.label(egui::RichText::new(crate::i18n::fmt(tl!("Recipients: {count}"), &[("count", &records.to_string())])).weak());
            let (ok, cancel) = buttons(ui, tl!("Close"));
            ok || cancel
        }
    }
}

/// The Recipient List dialog's body: a file button, the editable table, OK/Cancel. True to close.
fn recipient_list(app: &mut WordApp, ui: &mut Ui, fields: &mut Vec<String>, rows: &mut Vec<Vec<String>>, message: &mut String) -> bool {
    const CELL_W: f32 = 120.0;
    const X_W: f32 = 18.0;
    const ROW_H: f32 = 24.0;
    // Wide enough for every field (up to a limit; more scroll sideways).
    let gap = ui.spacing().item_spacing.x;
    let table_w = fields.len() as f32 * (CELL_W + X_W + 2.0 * gap) + X_W + 2.0 * gap;
    ui.set_min_width(table_w.clamp(340.0, 720.0));
    ui.label(tl!("Type the recipients, or load them from a file."));
    if ui.button(tl!("Use an Existing List…")).clicked() {
        // The picker takes over; the list it loads replaces this one.
        let _ = app.run("ui.openRecipientList", json!({}));
        return true;
    }
    ui.add_space(6.0);
    let mut remove_field = None;
    let can_remove = fields.len() > 1;
    let mut remove_row = None;
    let mut x_w = X_W;
    // The table keeps its left-to-right columns (a sideways scroll area can't lay out right to
    // left), in a box of its own width: at the right under RTL.
    rtl::fitted(ui, "recipients", |ui| {
        egui::ScrollArea::horizontal().id_salt("recipients_h").max_width(720.0).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    for (i, f) in fields.iter_mut().enumerate() {
                        ui.add_sized([CELL_W, ROW_H - 4.0], egui::TextEdit::singleline(f).font(egui::TextStyle::Button).hint_text(tl!("Field name")));
                        let x =
                            ui.add_enabled(can_remove, egui::Button::new("×").small().min_size(vec2(X_W, X_W))).on_hover_text(tl!("Remove field"));
                        x_w = x.rect.width();
                        if x.clicked() {
                            remove_field = Some(i);
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().id_salt("recipients_v").max_height(260.0).show_rows(ui, ROW_H, rows.len(), |ui, range| {
                    for r in range {
                        let Some(row) = rows.get_mut(r) else { continue };
                        ui.horizontal(|ui| {
                            for c in row.iter_mut() {
                                ui.add_sized([CELL_W, ROW_H - 4.0], egui::TextEdit::singleline(c));
                                // Under the field's remove button.
                                ui.allocate_exact_size(vec2(x_w, X_W), Sense::hover());
                            }
                            if ui.add(egui::Button::new("×").small().min_size(vec2(X_W, X_W))).on_hover_text(tl!("Remove recipient")).clicked() {
                                remove_row = Some(r);
                            }
                        });
                    }
                });
            });
        })
    });
    if let Some(i) = remove_field.filter(|_| fields.len() > 1) {
        fields.remove(i);
        for r in rows.iter_mut() {
            if i < r.len() {
                r.remove(i);
            }
        }
    }
    if let Some(r) = remove_row.filter(|r| *r < rows.len()) {
        rows.remove(r);
    }
    rtl::row(ui, |ui| {
        if ui.button(tl!("New Entry")).clicked() {
            rows.push(vec![String::new(); fields.len()]);
        }
        if ui.button(tl!("Add Field")).clicked() {
            fields.push(String::new());
            for r in rows.iter_mut() {
                r.push(String::new());
            }
        }
    });
    if !message.is_empty() {
        ui.label(egui::RichText::new(message.as_str()).color(Tokens::get(ui.ctx()).red));
    }
    let (ok, cancel) = buttons(ui, tl!("OK"));
    if !ok {
        return cancel;
    }
    match recipient_table(fields, rows).map_err(|e| tl!(e).to_string()).and_then(|params| app.load_recipients(params)) {
        Ok(_) => true,
        Err(e) => {
            *message = e;
            false
        }
    }
}

/// The Table Properties dialog's fields: Table, Row, Column and Cell sections.
fn table_properties(ui: &mut Ui, f: &mut TableForm) {
    let unit = crate::options::unit();
    fn len(v: &mut f32, lo: f32, unit: wordcraft_geom::Unit) -> egui::DragValue<'_> {
        egui::DragValue::new(v).speed(0.01).range(lo..=22.0).measure(unit).max_decimals(2)
    }
    let heading = |ui: &mut Ui, s: &str| {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tl!(s)).font(semibold(12.5)));
    };
    heading(ui, "Table");
    rtl::fitted(ui, "tp_table", |ui| {
        rtl::grid("tp_table", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            checked_row(ui, &mut f.width_on, "Preferred width:", |ui, on| {
                ui.add_enabled(on, len(&mut f.width, 0.1, unit));
            });
            rtl::grid_row(
                ui,
                [&mut |ui: &mut Ui| label(ui, "Alignment:"), &mut |ui: &mut Ui| {
                    rtl::row(ui, |ui| {
                        for (v, l) in [("left", "Left"), ("center", "Center"), ("right", "Right")] {
                            rtl::radio_value(ui, &mut f.align, v.to_string(), tl!(l));
                        }
                    });
                }],
            );
            let left = f.align == "left";
            rtl::grid_row(
                ui,
                [&mut |ui: &mut Ui| label(ui, "Indent from left:"), &mut |ui: &mut Ui| {
                    ui.add_enabled(left, egui::DragValue::new(&mut f.indent).speed(0.01).range(-11.0..=22.0).measure(unit).max_decimals(2));
                }],
            );
            rtl::grid_row(
                ui,
                [&mut |ui: &mut Ui| label(ui, "Table direction:"), &mut |ui: &mut Ui| {
                    rtl::row(ui, |ui| {
                        rtl::radio_value(ui, &mut f.rtl, false, tl!("Left-to-right"));
                        rtl::radio_value(ui, &mut f.rtl, true, tl!("Right-to-left"));
                    });
                }],
            );
        });
    });
    heading(ui, "Row");
    rtl::fitted(ui, "tp_row", |ui| {
        rtl::grid("tp_row", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            let exact = &mut f.row_exact;
            checked_row(ui, &mut f.row_height_on, "Specify height:", |ui, on| {
                rtl::row(ui, |ui| {
                    ui.add_enabled(on, len(&mut f.row_height, 0.02, unit));
                    ui.add_enabled_ui(on, |ui| {
                        egui::ComboBox::from_id_salt("tp_rule").selected_text(tl!(if *exact { "Exactly" } else { "At least" })).show_ui(ui, |ui| {
                            ui.selectable_value(exact, false, tl!("At least"));
                            ui.selectable_value(exact, true, tl!("Exactly"));
                        });
                    });
                });
            });
        });
    });
    rtl::checkbox(ui, &mut f.allow_break, tl!("Allow row to break across pages"));
    rtl::checkbox(ui, &mut f.header_row, tl!("Repeat as header row on each page"));
    heading(ui, "Column");
    rtl::fitted(ui, "tp_col", |ui| {
        rtl::grid("tp_col", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            rtl::grid_row(
                ui,
                [&mut |ui: &mut Ui| label(ui, "Preferred width:"), &mut |ui: &mut Ui| {
                    ui.add(len(&mut f.column_width, 0.1, unit));
                }],
            );
        });
    });
    heading(ui, "Cell");
    rtl::fitted(ui, "tp_cell", |ui| {
        rtl::grid("tp_cell", 2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            checked_row(ui, &mut f.cell_width_on, "Preferred width:", |ui, on| {
                ui.add_enabled(on, len(&mut f.cell_width, 0.1, unit));
            });
            rtl::grid_row(
                ui,
                [&mut |ui: &mut Ui| label(ui, "Vertical alignment:"), &mut |ui: &mut Ui| {
                    rtl::row(ui, |ui| {
                        for (v, l) in [("top", "Top"), ("center", "Center"), ("bottom", "Bottom")] {
                            rtl::radio_value(ui, &mut f.valign, v.to_string(), tl!(l));
                        }
                    });
                }],
            );
        });
    });
}

/// A grid row of a check box (the caption) and the field it turns on, `field(ui, on)`. In English
/// the box is drawn first and the field sees a click in the same frame; under RTL the field is
/// drawn first and sees it on the next frame (a click repaints).
pub(crate) fn checked_row(ui: &mut Ui, on: &mut bool, caption: &str, mut field: impl FnMut(&mut Ui, bool)) {
    let state = std::cell::Cell::new(*on);
    rtl::grid_row(
        ui,
        [
            &mut |ui: &mut Ui| {
                let mut v = state.get();
                rtl::checkbox(ui, &mut v, tl!(caption));
                state.set(v);
            },
            &mut |ui: &mut Ui| field(ui, state.get()),
        ],
    );
    *on = state.get();
}

/// The Paragraph dialog's OK. `align`, `left` and `right` are as seen on the page; `flags` are
/// keep with next, keep lines together, page break before, widow/orphan control; `opts` (now,
/// when opened) are mirror indents, contextual spacing, suppress line numbers and don't
/// hyphenate, of which only the changed ones are set.
#[allow(clippy::too_many_arguments)]
fn apply_paragraph(
    app: &mut WordApp,
    rtl: bool,
    align: &str,
    left: f32,
    right: f32,
    first: f32,
    before: f32,
    after: f32,
    line: (f32, f32),
    flags: [bool; 4],
    opts: ([bool; 4], [bool; 4]),
) {
    let [keep_next, keep_lines, page_break, widow] = flags;
    let (line, line_was) = line;
    // Direction first: the alignment and indents are as seen on the page, and their logical
    // values depend on it.
    let _ = app.run(if rtl { "para.rtl" } else { "para.ltr" }, json!({}));
    let _ = app.run("para.align", json!({"value": align}));
    let (start, end) = if rtl { (right, left) } else { (left, right) };
    let mut props = json!({
        "indentLeft": start * crate::options::unit().pt_per_unit(), "indentRight": end * crate::options::unit().pt_per_unit(), "indentFirst": first * crate::options::unit().pt_per_unit(),
        "spaceBefore": before, "spaceAfter": after,
        "keepNext": keep_next, "keepLines": keep_lines, "pageBreakBefore": page_break, "widowControl": widow,
    });
    // The dialog edits only multiples: an untouched value keeps Exactly and At Least.
    if line != line_was
        && let Some(m) = props.as_object_mut()
    {
        m.insert("lineSpacing".into(), json!({"rule": "multiple", "value": line}));
    }
    let (now, was) = opts;
    for (key, (on, was)) in ["mirrorIndents", "contextualSpacing", "suppressLineNumbers", "suppressHyphens"].into_iter().zip(now.into_iter().zip(was))
    {
        if on != was
            && let Some(m) = props.as_object_mut()
        {
            m.insert(key.into(), json!(on));
        }
    }
    let _ = app.run("para.set", json!({ "props": props }));
}

/// The Paragraph dialog's Asian Typography: sets the flags that changed since it opened, so an
/// untouched tab adds no direct formatting.
fn apply_asian(app: &mut WordApp, asian: [bool; 6], was: [bool; 6]) {
    let keys = ["kinsoku", "wordWrap", "overflowPunct", "topLinePunct", "autoSpaceDE", "autoSpaceDN"];
    let changed: serde_json::Map<String, Value> =
        keys.iter().zip(asian.iter().zip(was)).filter(|(_, (now, was))| **now != *was).map(|(k, (now, _))| (k.to_string(), json!(now))).collect();
    if !changed.is_empty() {
        let _ = app.run("para.asianTypography", Value::Object(changed));
    }
}

/// Manage Styles' body. Reads the list from `styles.manage`'s data (never re-running the command,
/// which would ask to open this dialog again) and acts through commands.
fn manage_styles(app: &mut WordApp, ui: &mut Ui, alphabetical: &mut bool, selected: &mut String) -> bool {
    let list = wordcraft_engine::cmd::para::manage_list(&app.session, *alphabetical);
    let list = list.as_array().cloned().unwrap_or_default();
    let str_of = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    if !list.iter().any(|x| str_of(x, "id") == *selected) {
        *selected = list.first().map(|x| str_of(x, "id")).unwrap_or_default();
    }
    rtl::row(ui, |ui| {
        ui.label(tl!("Sort order:"));
        egui::ComboBox::from_id_salt("manage_styles_sort")
            .selected_text(if *alphabetical { tl!("Alphabetical") } else { tl!("As Recommended") })
            .show_ui(ui, |ui| {
                ui.selectable_value(alphabetical, false, tl!("As Recommended"));
                ui.selectable_value(alphabetical, true, tl!("Alphabetical"));
            });
    });
    let t = Tokens::get(ui.ctx());
    let shown = egui::Id::new("manage_styles_shown");
    egui::Frame::NONE.stroke(egui::Stroke::new(1.0, t.border)).inner_margin(4).show(ui, |ui| {
        egui::ScrollArea::vertical().id_salt("manage_styles_list").max_height(200.0).auto_shrink([false, false]).show(ui, |ui| {
            ui.set_width(380.0);
            for st in &list {
                let id = str_of(st, "id");
                let glyph = match st.get("type").and_then(Value::as_str) {
                    Some("character") => "a",
                    Some("linked") => "¶a",
                    Some("table") => "⊞",
                    _ => "¶",
                };
                let hidden = st.get("hidden").and_then(Value::as_bool).unwrap_or(false);
                // The type glyph leads the name: at its right under RTL.
                let line = if rtl::is_rtl() {
                    format!("{}  {glyph}", crate::i18n::isolate(&str_of(st, "name")))
                } else {
                    format!("{glyph}  {}", str_of(st, "name"))
                };
                let mut name = egui::RichText::new(line);
                if hidden {
                    name = name.weak();
                }
                let r = ui.selectable_label(*selected == id, name);
                // Bring the selection into view once, when it changes (opening, New Style…).
                if *selected == id && ui.data(|d| d.get_temp::<String>(shown)).as_deref() != Some(id.as_str()) {
                    r.scroll_to_me(Some(egui::Align::Center));
                    ui.data_mut(|d| d.insert_temp(shown, id.clone()));
                }
                if r.clicked() {
                    ui.data_mut(|d| d.insert_temp(shown, id.clone()));
                    *selected = id.clone();
                }
                if r.double_clicked() {
                    *selected = id;
                    if let Some(Dialog::ModifyStyle { id, name, font, size, bold, italic, color, before, after, .. }) =
                        Dialog::modify_style(app, selected)
                    {
                        app.dialog = Some(Dialog::ModifyStyle { back: true, resume: None, id, name, font, size, bold, italic, color, before, after });
                    }
                }
            }
        });
    });
    let leave = |ui: &mut Ui| {
        ui.data_mut(|d| d.remove::<String>(shown));
        true
    };
    if app.dialog.is_some() {
        return leave(ui);
    }
    let Some(cur) = list.iter().find(|x| str_of(x, "id") == *selected).cloned() else { return close_button(ui) && leave(ui) };
    let ty = str_of(&cur, "type");
    ui.add_space(6.0);
    crate::previews::style_preview(app, ui, selected, &ty, vec2(388.0, if ty == "table" { 64.0 } else { 40.0 }));
    ui.add_space(4.0);
    if rtl::is_rtl() {
        // Each part is in draw order on its own: they run from the right (the dialog's column is
        // right-aligned, so `horizontal_wrapped` lays out right to left), each but the last with
        // the Arabic comma at its left end, where it falls after the part when read.
        let parts = describe_parts(&cur);
        let last = parts.len().saturating_sub(1);
        ui.horizontal_wrapped(|ui| {
            for (k, part) in parts.iter().enumerate() {
                let text = if k < last { format!("\u{60C}{part}") } else { part.clone() };
                ui.add(egui::Label::new(egui::RichText::new(text).small()).wrap_mode(egui::TextWrapMode::Extend));
            }
        });
    } else {
        ui.add(egui::Label::new(egui::RichText::new(describe(&cur)).small()).wrap());
    }
    ui.add_space(6.0);
    let builtin = cur.get("builtIn").and_then(Value::as_bool).unwrap_or(true);
    if ty != "table" {
        let mut gallery = cur.get("inGallery").and_then(Value::as_bool).unwrap_or(false);
        if rtl::checkbox(ui, &mut gallery, tl!("Show in the Styles gallery")).changed() {
            let _ = app.run("styles.setVisibility", json!({"style": selected, "gallery": gallery}));
        }
    }
    let mut hidden = cur.get("hidden").and_then(Value::as_bool).unwrap_or(false);
    if rtl::checkbox(ui, &mut hidden, tl!("Hide from style lists")).changed() {
        let _ = app.run("styles.setVisibility", json!({"style": selected, "hidden": hidden}));
    }
    ui.add_space(4.0);
    rtl::row(ui, |ui| {
        // Table styles are edited from Table Design.
        if ui.add_enabled(ty != "table", egui::Button::new(tl!("Modify…"))).clicked()
            && let Some(Dialog::ModifyStyle { id, name, font, size, bold, italic, color, before, after, .. }) = Dialog::modify_style(app, selected)
        {
            app.dialog = Some(Dialog::ModifyStyle { back: true, resume: None, id, name, font, size, bold, italic, color, before, after });
        }
        if ui.button(tl!("New Style…")).clicked() {
            app.dialog = Some(Dialog::NewStyle { name: "Style1".into(), based_on: str_of(&cur, "name"), back: true });
        }
        let del = ui.add_enabled(!builtin, egui::Button::new(tl!("Delete")));
        let del = if builtin { del.on_disabled_hover_text(tl!("Built-in styles can't be deleted.")) } else { del };
        if del.clicked() {
            let _ = app.run("styles.delete", json!({"style": selected}));
            selected.clear();
        }
    });
    (app.dialog.is_some() || close_button(ui)) && leave(ui)
}

/// A dialog's single Close button (Escape too).
fn close_button(ui: &mut Ui) -> bool {
    ui.add_space(8.0);
    let mut close = ui.input(|i| i.key_pressed(egui::Key::Escape));
    ui.with_layout(rtl::end_layout(egui::Align::Center), |ui| {
        close |= ui.add(crate::widgets::primary_button(ui, tl!("Close"))).clicked();
    });
    close
}

/// A style's description: its type, the formatting it sets and what it is based on.
fn describe(st: &Value) -> String {
    describe_parts(st).join(", ")
}

/// The parts of [`describe`], each a catalog string or a `fmt` result (in draw order).
fn describe_parts(st: &Value) -> Vec<String> {
    let f = st.get("format").cloned().unwrap_or_default();
    let fs = |k: &str| f.get(k).and_then(Value::as_str);
    let fnum = |k: &str| f.get(k).and_then(Value::as_f64);
    let pt = |x: f64| format!("{}", (x * 10.0).round() / 10.0);
    let mut parts: Vec<String> = vec![
        match st.get("type").and_then(Value::as_str) {
            Some("character") => tl!("Character style"),
            Some("linked") => tl!("Linked style (paragraph and character)"),
            Some("table") => tl!("Table style"),
            _ => tl!("Paragraph style"),
        }
        .to_string(),
    ];
    if st.get("builtIn").and_then(Value::as_bool).unwrap_or(false) {
        parts.push(tl!("Built-in style").to_string());
    }
    if let Some(x) = fs("font") {
        parts.push(crate::i18n::fmt(tl!("Font: {font}"), &[("font", x)]));
    }
    if let Some(x) = fnum("size") {
        parts.push(crate::i18n::fmt(tl!("{size} pt"), &[("size", &pt(x))]));
    }
    if f.get("bold").and_then(Value::as_bool) == Some(true) {
        parts.push(tl!("Bold").to_string());
    }
    if f.get("italic").and_then(Value::as_bool) == Some(true) {
        parts.push(tl!("Italic").to_string());
    }
    if let Some(x) = fs("color") {
        parts.push(crate::i18n::fmt(tl!("Color: {color}"), &[("color", &format!("#{x}"))]));
    }
    if let Some(x) = fnum("spaceBefore") {
        parts.push(crate::i18n::fmt(tl!("Space before: {pt} pt"), &[("pt", &pt(x))]));
    }
    if let Some(x) = fnum("spaceAfter") {
        parts.push(crate::i18n::fmt(tl!("Space after: {pt} pt"), &[("pt", &pt(x))]));
    }
    if let Some(b) = st.get("basedOn").and_then(Value::as_str) {
        // `manage_list` gives the base style's name as the document has it (not catalog output,
        // like the list's own names), so `fmt` isolates it.
        parts.push(crate::i18n::fmt(tl!("Based on: {style}"), &[("style", b)]));
    }
    parts
}

/// What Search lists before anything is typed: common commands (#819).
const SEARCH_SUGGESTIONS: [&str; 3] = ["review.spelling", "review.wordCount", "file.print"];

/// Search's "Find … in the document": the Navigation pane opens on its results for `text`.
fn find_in_navigation_pane(app: &mut WordApp, ctx: &egui::Context, text: &str) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("nav_query"), text.to_string()));
    app.ui.nav_tab = "results".into();
    let _ = app.run("view.navigationPane", json!({"value": true}));
    let _ = app.run("edit.find", json!({"text": text}));
}

/// Whether `e` is the text of the shortcut that opened a box (see `Dialog::Commands::swallow`):
/// a plain key press first means the user is typing, and one swallowed text is all there is.
fn swallow_shortcut_text(swallow: &mut bool, e: &egui::Event) -> bool {
    match e {
        egui::Event::Key { pressed: true, modifiers, .. } if !modifiers.alt => {
            *swallow = false;
            false
        }
        egui::Event::Text(_) if *swallow => {
            *swallow = false;
            true
        }
        _ => false,
    }
}

/// Search's filter: every word of the query is in the command's label, location or id, in English
/// or in the interface language, so "insert table" finds Insert › Tables › Table (#819).
fn command_matches(c: &wordcraft_engine::CommandSpec, query: &str) -> bool {
    if query.trim().is_empty() {
        return false;
    }
    let hay = format!(
        "{} {} {} {} {}",
        c.label.to_lowercase(),
        c.location.to_lowercase(),
        c.id.to_lowercase(),
        crate::i18n::lowercase(tl!(c.label)),
        crate::i18n::lowercase(&crate::i18n::location(c.location))
    );
    query.split_whitespace().all(|w| hay.contains(&w.to_lowercase()) || hay.contains(&crate::i18n::lowercase(w)))
}

/// The Find and Replace dialog's Go To tab (#712): a page (a number, or `+2` / `-1` from the
/// current page; empty, Next and Previous step a page) or a bookmark. Returns true to close.
fn goto_body(app: &mut WordApp, ui: &mut Ui, what: &mut GoToWhat, target: &mut String, message: &mut String) -> bool {
    // The document's bookmarks, only while they are listed (finding them walks the document).
    let bookmarks: Vec<String> = if *what == GoToWhat::Bookmark {
        app.session.doc.bookmarks().into_iter().map(|(n, _)| n).filter(|n| !n.starts_with('_')).collect()
    } else {
        Vec::new()
    };
    // In the right-aligned column `horizontal_top` already runs right to left.
    ui.horizontal_top(|ui| {
        rtl::column(ui, |ui| {
            ui.label(tl!("Go to what:"));
            for (w, label) in [(GoToWhat::Page, "Page"), (GoToWhat::Bookmark, "Bookmark")] {
                if ui.selectable_label(*what == w, tl!(label)).clicked() && *what != w {
                    *what = w;
                    target.clear();
                    message.clear();
                }
            }
        });
        ui.add_space(16.0);
        rtl::column(ui, |ui| match what {
            GoToWhat::Page => {
                ui.label(tl!("Enter page number:"));
                ui.add(egui::TextEdit::singleline(target).desired_width(220.0)).request_focus();
                ui.label(egui::RichText::new(tl!("Type +2 or -1 to move from the page you are on.")).weak());
            }
            GoToWhat::Bookmark => {
                ui.label(tl!("Enter bookmark name:"));
                egui::ComboBox::from_id_salt("goto_bookmark").width(220.0).selected_text(target.as_str()).show_ui(ui, |ui| {
                    for b in &bookmarks {
                        ui.selectable_value(target, b.clone(), b.as_str());
                    }
                });
                if bookmarks.is_empty() {
                    ui.label(egui::RichText::new(tl!("This document has no bookmarks.")).weak());
                }
            }
        });
    });
    if !message.is_empty() {
        ui.label(egui::RichText::new(message.as_str()).weak());
    }
    let mut close = false;
    let mut go: Option<Value> = None;
    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
    ui.add_space(8.0);
    rtl::row(ui, |ui| {
        let page_step = *what == GoToWhat::Page && target.trim().is_empty();
        if page_step && ui.button(tl!("Previous")).clicked() {
            go = Some(json!({"pageBy": -1}));
        }
        let label = if page_step { tl!("Next") } else { tl!("Go To") };
        if ui.button(label).clicked() || enter {
            go = match what {
                GoToWhat::Page => match goto_page(target) {
                    Some(p) => Some(p),
                    None => {
                        *message = tl!("Type a page number, or +2 / -1.").to_string();
                        None
                    }
                },
                GoToWhat::Bookmark if !target.is_empty() => Some(json!({"bookmark": target.clone()})),
                GoToWhat::Bookmark => None,
            };
        }
        if ui.button(tl!("Close")).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
    });
    if let Some(params) = go {
        match app.run("edit.goto", params) {
            Ok(_) => message.clear(),
            Err(e) => *message = e,
        }
    }
    close
}

/// Go To's page box as `edit.goto` params: empty steps one page on, `+n` / `-n` move from the
/// current page, a number is that page. `None` for anything else.
fn goto_page(text: &str) -> Option<Value> {
    let t = text.trim().replace('\u{2212}', "-");
    if t.is_empty() {
        return Some(json!({"pageBy": 1}));
    }
    if let Some(rest) = t.strip_prefix('+') {
        return rest.parse::<i64>().ok().filter(|n| *n >= 0).map(|n| json!({"pageBy": n}));
    }
    if t.starts_with('-') {
        return t.parse::<i64>().ok().map(|n| json!({"pageBy": n}));
    }
    t.parse::<u64>().ok().filter(|n| *n >= 1).map(|n| json!({"page": n}))
}

/// path data; Insert (or a double click) runs `insert.icon`. Returns true to close.
fn icons_body(app: &mut WordApp, ui: &mut Ui, query: &mut String, category: &mut String, selected: &mut String) -> bool {
    use wordcraft_render::icon_lib::{self, Category};
    let t = Tokens::get(ui.ctx());
    ui.set_width(560.0);
    // Typing goes to the search box (never to the document behind the dialog).
    ui.add(egui::TextEdit::singleline(query).hint_text(tl!("Search icons")).desired_width(560.0)).request_focus();
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        if ui.selectable_label(category.is_empty(), tl!("All")).clicked() {
            category.clear();
        }
        for c in Category::ALL {
            if ui.selectable_label(*category == c.name(), tl!(c.name())).clicked() {
                *category = c.name().to_string();
            }
        }
    });
    ui.add_space(4.0);
    let hits = icon_lib::search(query, Category::parse(category));
    let mut insert = false;
    egui::ScrollArea::vertical().max_height(300.0).min_scrolled_height(300.0).auto_shrink([true, false]).show(ui, |ui| {
        if hits.is_empty() {
            ui.label(egui::RichText::new(tl!("No icons match your search.")).color(t.text_dim));
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for icon in &hits {
                let (rect, resp) = ui.allocate_exact_size(vec2(56.0, 56.0), Sense::click());
                let on = *selected == icon.id;
                let fill = if on {
                    t.checked
                } else if resp.hovered() {
                    t.hover
                } else {
                    egui::Color32::TRANSPARENT
                };
                let edge = if on { egui::Stroke::new(1.5, t.accent) } else { egui::Stroke::new(1.0, t.border) };
                ui.painter().rect(rect, 4.0, fill, edge, egui::StrokeKind::Inside);
                crate::icons::library(ui.painter(), rect.shrink(12.0), icon, t.text);
                let resp = resp.on_hover_text(icon.name);
                if resp.clicked() {
                    *selected = icon.id.to_string();
                }
                if resp.double_clicked() {
                    *selected = icon.id.to_string();
                    insert = true;
                }
            }
        });
    });
    let name = icon_lib::find(selected).map(|i| i.name).unwrap_or("");
    ui.add_space(4.0);
    ui.label(egui::RichText::new(name).color(t.text_dim));
    let (ok, cancel) = buttons(ui, tl!("Insert"));
    if (ok || insert) && !selected.is_empty() {
        let _ = app.run("insert.icon", json!({"id": selected.as_str()}));
        return true;
    }
    cancel
}

/// Enclose Characters' preview: the text in the chosen shape, shrunk or with the shape enlarged
/// (drawn, so it doesn't depend on the interface font having ○ □ △ ◇).
fn enclose_preview(ui: &mut Ui, style: &str, text: &str, shape: &str) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(110.0, 64.0), Sense::hover());
    let p = ui.painter_at(r);
    p.rect_filled(r, 2.0, t.input);
    p.rect_stroke(r, 2.0, egui::Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
    let ink = ui.visuals().text_color();
    let c = r.center();
    let (box_h, text_px) = match style {
        "none" => (0.0, 26.0),
        "enlarge" => (48.0, 26.0),
        _ => (34.0, 20.0),
    };
    let h = box_h / 2.0;
    let stroke = egui::Stroke::new(1.5, ink);
    match (shape, box_h > 0.0) {
        (_, false) => {}
        ("square", _) => {
            p.rect_stroke(egui::Rect::from_center_size(c, vec2(box_h, box_h)), 0.0, stroke, egui::StrokeKind::Middle);
        }
        ("triangle", _) => {
            let pts = vec![egui::pos2(c.x, c.y - h), egui::pos2(c.x + h * 1.1, c.y + h), egui::pos2(c.x - h * 1.1, c.y + h)];
            p.add(egui::Shape::closed_line(pts, stroke));
        }
        ("diamond", _) => {
            let pts = vec![egui::pos2(c.x, c.y - h), egui::pos2(c.x + h, c.y), egui::pos2(c.x, c.y + h), egui::pos2(c.x - h, c.y)];
            p.add(egui::Shape::closed_line(pts, stroke));
        }
        _ => {
            p.circle_stroke(c, h, stroke);
        }
    }
    let px = match (style, shape) {
        ("shrink", "triangle" | "diamond") => text_px * 0.75,
        _ => text_px,
    };
    let dy = if shape == "triangle" && box_h > 0.0 { h * 0.3 } else { 0.0 };
    p.text(c + vec2(0.0, dy), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(px), ink);
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::Pos;
    use wordcraft_doc::props::Align;
    use wordcraft_engine::Session;

    /// Go To's page box (#712): a number, +n / -n from the current page (a typographic minus
    /// too), empty for the next page; anything else is refused.
    #[test]
    fn go_to_reads_its_page_box() {
        assert_eq!(goto_page("7"), Some(json!({"page": 7})));
        assert_eq!(goto_page(" +2 "), Some(json!({"pageBy": 2})));
        assert_eq!(goto_page("-1"), Some(json!({"pageBy": -1})));
        assert_eq!(goto_page("\u{2212}3"), Some(json!({"pageBy": -3})));
        assert_eq!(goto_page(""), Some(json!({"pageBy": 1})));
        for junk in ["0", "abc", "+x", "1.5", "99999999999999999999999"] {
            assert_eq!(goto_page(junk), None, "{junk}");
        }
    }

    /// Find, Replace and Go To are one dialog with three tabs (#712).
    #[test]
    fn find_replace_and_go_to_are_one_dialog() {
        let mut app = app_with("One");
        for (name, tab) in [("find", FindTab::Find), ("replace", FindTab::Replace), ("goto", FindTab::GoTo)] {
            let _ = app.run("ui.dialog", json!({"name": name}));
            assert!(matches!(&app.dialog, Some(Dialog::Find { tab: t, .. }) if *t == tab), "{name}");
            assert_eq!(app.dialog.as_ref().map(|d| d.name()), Some(name));
        }
    }

    fn app_with(text: &str) -> WordApp {
        let mut app = WordApp::new(Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        let _ = app.run("document.setText", json!({"text": text}));
        app
    }

    fn props(app: &WordApp) -> wordcraft_doc::props::ParaProps {
        app.session.doc.para_at(&app.session.sel.focus).map(|p| p.props.clone()).unwrap_or_default()
    }

    /// #819: the text a shortcut sends with its key isn't typed into the box it opened, whether it
    /// comes in the same frame or the next; real typing (a key press first) is.
    #[test]
    fn shortcut_text_is_swallowed_once() {
        let key = |key, alt| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers { alt, ..Default::default() },
        };
        let text = |t: &str| egui::Event::Text(t.into());
        let run = |events: &[egui::Event]| {
            let mut swallow = true;
            events.iter().filter(|e| !swallow_shortcut_text(&mut swallow, e)).cloned().collect::<Vec<_>>()
        };
        assert_eq!(run(&[key(egui::Key::Q, true), text("q")]), vec![key(egui::Key::Q, true)], "Alt+Q's q, same frame");
        assert_eq!(run(&[text("q"), text("b")]), vec![text("b")], "the q a frame late; then typing");
        assert_eq!(run(&[key(egui::Key::B, false), text("b")]), vec![key(egui::Key::B, false), text("b")], "a click-opened box takes typing");
    }

    /// #819: Search matches every word anywhere in the label, location or id, in any order.
    #[test]
    fn command_search_matches_every_word() {
        let app = app_with("");
        let reg = app.session.registry.clone();
        let table = reg.get("insert.table").unwrap();
        assert!(command_matches(table, "insert table"), "the box's own example");
        assert!(command_matches(table, "  Table   INSERT "), "any order, any case, extra spaces");
        assert!(!command_matches(table, "insert chart"), "every word has to match");
        assert!(!command_matches(table, ""), "an empty box lists nothing");
        assert!(!command_matches(table, "   "));
    }

    /// Ordinary Turkish queries must find dotted-I labels, while canonical English labels and
    /// ASCII command IDs retain their old case-insensitive search in every interface language.
    #[test]
    fn command_search_respects_turkish_i_pairs_and_keeps_canonical_matches() {
        use egui_kittest::kittest::Queryable;
        let mut fonts = false;
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui_state(
            move |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                crate::i18n::set_current(crate::i18n::Lang::from_pref(&app.ui.language));
                if fonts {
                    show(app, &ctx);
                } else {
                    crate::theme::install_fonts_for(&ctx, false, false);
                    fonts = true;
                }
            },
            app_with("Unchanged document"),
        );
        let before = serde_json::to_value(&h.state().session.doc).unwrap();
        for (language, query, expected) in [
            ("tr", "italik", "İtalik   —   Giriş › Yazı Tipi"),
            ("tr", "İTALİK", "İtalik   —   Giriş › Yazı Tipi"),
            ("tr", "içindekiler", "İçindekiler Tablosu   —   Başvurular › İçindekiler Tablosu"),
            ("tr", "YAZI TİPİ", "İtalik   —   Giriş › Yazı Tipi"),
            ("tr", "INSERT", "Tablo   —   Ekle › Tablolar"),
            ("tr", "INSERT.TABLE", "Tablo   —   Ekle › Tablolar"),
            ("tr", "FORMAT.ITALIC", "İtalik   —   Giriş › Yazı Tipi"),
            ("tr", "TABLE OF CONTENTS", "İçindekiler Tablosu   —   Başvurular › İçindekiler Tablosu"),
            ("en", "ITALIC", "Italic   —   Home › Font"),
            ("en", "FORMAT.ITALIC", "Italic   —   Home › Font"),
            ("en", "INSERT.TABLE", "Table   —   Insert › Tables"),
        ] {
            h.state_mut().ui.language = language.into();
            h.state_mut().dialog = Some(Dialog::Commands { query: query.into(), swallow: false });
            h.run_steps(4);
            assert!(
                h.query(egui_kittest::kittest::By::new().role(egui::accesskit::Role::Button).label_contains(expected)).is_some(),
                "{language}: {query}\n{:?}",
                h.query_all_by_role(egui::accesskit::Role::Button).collect::<Vec<_>>()
            );
            assert_eq!(crate::i18n::lowercase("Iİıi"), if language == "tr" { "ıiıi" } else { "ii\u{307}ıi" });
        }
        assert_eq!(serde_json::to_value(&h.state().session.doc).unwrap(), before, "search does not change the document");
        crate::i18n::set_current(crate::i18n::Lang::EN);
    }

    /// Open the Paragraph dialog and press OK without changing anything.
    fn ok_unchanged(app: &mut WordApp) {
        ok_with(app, |_| {});
    }

    /// Open the Paragraph dialog, change its options (`opts`) with `edit` and press OK.
    fn ok_with(app: &mut WordApp, edit: impl FnOnce(&mut [bool; 4])) {
        let Some(Dialog::Paragraph {
            rtl,
            align,
            left,
            right,
            first,
            before,
            after,
            line,
            line_was,
            keep_next,
            keep_lines,
            page_break,
            widow,
            mut opts,
            ..
        }) = Dialog::open("paragraph", app)
        else {
            panic!("no paragraph dialog")
        };
        let was = opts;
        edit(&mut opts);
        apply_paragraph(
            app,
            rtl,
            &align,
            left,
            right,
            first,
            before,
            after,
            (line, line_was),
            [keep_next, keep_lines, page_break, widow],
            (opts, was),
        );
    }

    #[test]
    fn paragraph_dialog_sets_only_the_options_that_changed() {
        // #512: mirror indents, same-style spacing, suppress line numbers, don't hyphenate.
        let mut app = app_with("Hello");
        ok_unchanged(&mut app);
        let p = props(&app);
        assert_eq!((p.mirror_indents, p.contextual_spacing, p.suppress_line_numbers, p.suppress_hyphens), (None, None, None, None));
        ok_with(&mut app, |o| *o = [true; 4]);
        let p = props(&app);
        assert_eq!(
            (p.mirror_indents, p.contextual_spacing, p.suppress_line_numbers, p.suppress_hyphens),
            (Some(true), Some(true), Some(true), Some(true))
        );
        let Some(Dialog::Paragraph { opts, .. }) = Dialog::open("paragraph", &mut app) else { panic!() };
        assert_eq!(opts, [true; 4], "the dialog shows what is set");
        ok_with(&mut app, |o| o[3] = false);
        assert_eq!(props(&app).suppress_hyphens, Some(false));
        assert_eq!(props(&app).mirror_indents, Some(true));
    }

    #[test]
    fn paragraph_dialog_shows_direction_and_visual_alignment() {
        let mut app = app_with("سلام دنیا");
        let _ = app.run("para.rtl", json!({}));
        let _ = app.run("para.set", json!({"props": {"indentLeft": 36.0}}));
        let Some(Dialog::Paragraph { rtl, align, left, right, .. }) = Dialog::open("paragraph", &mut app) else { panic!() };
        assert!(rtl);
        assert_eq!(align, "right", "a start-aligned right-to-left paragraph shows as right aligned");
        assert_eq!((left, right), (0.0, 0.5), "the start indent is the right one");
    }

    #[test]
    fn paragraph_dialog_ok_keeps_what_it_shows() {
        for (rtl, align) in [(false, "para.alignCenter"), (false, "para.alignRight"), (true, "para.alignLeft"), (true, "para.alignRight")] {
            let mut app = app_with("متن");
            let _ = app.run(if rtl { "para.rtl" } else { "para.ltr" }, json!({}));
            let _ = app.run(align, json!({}));
            let _ = app.run("para.set", json!({"props": {"indentLeft": 18.0, "indentRight": 9.0}}));
            let before = props(&app);
            ok_unchanged(&mut app);
            let after = props(&app);
            assert_eq!((after.bidi, after.align), (before.bidi, before.align), "{align} in rtl={rtl}");
            assert_eq!((after.indent_left, after.indent_right), (Some(18.0), Some(9.0)), "{align} in rtl={rtl}");
        }
    }

    #[test]
    fn paragraph_dialog_keeps_fixed_line_spacing() {
        use wordcraft_doc::props::LineSpacing;
        for (cmd, rule) in [(json!({"exactly": 24}), LineSpacing::Exactly(24.0)), (json!({"atLeast": 24}), LineSpacing::AtLeast(24.0))] {
            // OK without changes, and OK after changing only Keep with next (#467).
            let mut app = app_with("Example");
            let _ = app.run("para.lineSpacing", cmd.clone());
            ok_unchanged(&mut app);
            assert_eq!(props(&app).line_spacing, Some(rule), "unchanged OK, {cmd}");
            let Some(Dialog::Paragraph { rtl, align, left, right, first, before, after, line, line_was, keep_lines, page_break, widow, .. }) =
                Dialog::open("paragraph", &mut app)
            else {
                panic!("no paragraph dialog")
            };
            apply_paragraph(
                &mut app,
                rtl,
                &align,
                left,
                right,
                first,
                before,
                after,
                (line, line_was),
                [true, keep_lines, page_break, widow],
                ([false; 4], [false; 4]),
            );
            let p = props(&app);
            assert_eq!((p.line_spacing, p.keep_next), (Some(rule), Some(true)), "Keep with next, {cmd}");
        }
        // A changed value still sets a multiple.
        let mut app = app_with("Example");
        let _ = app.run("para.lineSpacing", json!({"exactly": 24}));
        let Some(Dialog::Paragraph { rtl, align, left, right, first, before, after, line_was, keep_next, keep_lines, page_break, widow, .. }) =
            Dialog::open("paragraph", &mut app)
        else {
            panic!("no paragraph dialog")
        };
        apply_paragraph(
            &mut app,
            rtl,
            &align,
            left,
            right,
            first,
            before,
            after,
            (1.5, line_was),
            [keep_next, keep_lines, page_break, widow],
            ([false; 4], [false; 4]),
        );
        assert_eq!(props(&app).line_spacing, Some(LineSpacing::Multiple(1.5)));
    }

    #[test]
    fn font_dialog_sets_the_asian_and_latin_fonts_separately() {
        // #505: the Asian text font box and the Latin text font box each set their own font, and
        // OK leaves the one not changed as it was.
        let mut app = app_with("Hello 你好");
        let _ = app.run("select.all", json!({}));
        let fonts = |app: &WordApp| {
            let c = app.session.doc.para_at(&Pos::body(0, 0)).map(|p| p.props_of_char(0).clone()).unwrap_or_default();
            (c.font, c.font_ea)
        };
        let latin = fonts(&app).0;
        let Some(mut d) = Dialog::open("font", &mut app) else { panic!("no font dialog") };
        let Dialog::Font { asian, font_ea, .. } = &mut d else { panic!() };
        assert!(*asian, "Chinese text in the selection shows the Asian text font box");
        *font_ea = "SimSun".into();
        apply_font(&mut app, &d);
        assert_eq!(fonts(&app), (latin, Some("SimSun".into())));
        let Some(mut d) = Dialog::open("font", &mut app) else { panic!() };
        let Dialog::Font { font, font_ea, .. } = &mut d else { panic!() };
        assert_eq!(font_ea, "SimSun");
        *font = "Georgia".into();
        apply_font(&mut app, &d);
        assert_eq!(fonts(&app), (Some("Georgia".into()), Some("SimSun".into())));
        // One undo step each.
        let _ = app.run("edit.undo", json!({}));
        assert_eq!(fonts(&app).1.as_deref(), Some("SimSun"));
        assert_ne!(fonts(&app).0.as_deref(), Some("Georgia"));
    }

    #[test]
    fn paragraph_dialog_changes_direction() {
        let mut app = app_with("Hello");
        let Some(Dialog::Paragraph { align, left, right, first, before, after, line, line_was, keep_next, keep_lines, page_break, widow, .. }) =
            Dialog::open("paragraph", &mut app)
        else {
            panic!()
        };
        // Right to left, aligned right on the page, 1" from the right edge.
        apply_paragraph(
            &mut app,
            true,
            "right",
            left,
            right + 1.0,
            first,
            before,
            after,
            (line, line_was),
            [keep_next, keep_lines, page_break, widow],
            ([false; 4], [false; 4]),
        );
        let _ = align;
        let p = props(&app);
        assert_eq!(p.bidi, Some(true));
        assert_eq!(p.align, Some(Align::Left), "right on the page is the start edge");
        assert_eq!(p.indent_left, Some(72.0), "the right indent is the start indent");
    }

    /// Switching the Paragraph dialog's tabs re-centred it, so the tab strip jumped up and down
    /// by half the height difference (#507): the top edge now stays put and only the bottom moves.
    #[test]
    fn switching_paragraph_tabs_keeps_the_top_edge() {
        use egui_kittest::kittest::Queryable;
        let mut app = app_with("Some text");
        app.dialog = Dialog::open("paragraph", &mut app);
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        let mut tops = vec![];
        let mut oks = vec![];
        for page in [0u8, 1, 2, 0] {
            if let Some(Dialog::Paragraph { tab, .. }) = &mut h.state_mut().dialog {
                *tab = page;
            }
            for _ in 0..4 {
                h.step();
            }
            tops.push(h.get_by_label("Indents and Spacing").rect().top());
            oks.push(h.get_by_role_and_label(egui::accesskit::Role::Button, "OK").rect().top());
        }
        assert!((oks[0] - oks[1]).abs() > 10.0, "the tabs differ in height: {oks:?}");
        for t in &tops {
            assert!((t - tops[0]).abs() <= 0.5, "the tab strip stays put: {tops:?}");
        }
    }

    #[test]
    fn font_dialog_ok_keeps_what_the_user_left_alone() {
        // Issue #466: OK sent every field, clearing small caps, spacing and double underline
        // when only the size changed.
        let mut app = app_with("Example");
        let _ = app.run("select.all", json!({}));
        let _ = app.run("format.set", json!({"props": {"smallCaps": true, "caps": true, "hidden": true, "spacing": 2, "underline": "double"}}));
        let Some(mut d) = Dialog::open("font", &mut app) else { panic!("no font dialog") };
        if let Dialog::Font { size, small_caps, spacing, .. } = &mut d {
            assert!(*small_caps, "the dialog shows small caps");
            assert_eq!(*spacing, 2.0);
            *size = "14".into();
        }
        apply_font(&mut app, &d);
        let c = app.session.doc.para_at(&wordcraft_doc::Pos::body(0, 0)).map(|p| p.props_of_char(0).clone()).unwrap_or_default();
        assert_eq!(c.size, Some(14.0));
        assert_eq!((c.small_caps, c.caps, c.hidden, c.spacing), (Some(true), Some(true), Some(true), Some(2.0)));
        assert_eq!(c.underline, Some(wordcraft_doc::props::Underline::Double));
        // A field the user does change still applies.
        let Some(mut d) = Dialog::open("font", &mut app) else { panic!("no font dialog") };
        if let Dialog::Font { small_caps, .. } = &mut d {
            *small_caps = false;
        }
        apply_font(&mut app, &d);
        let c = app.session.doc.para_at(&wordcraft_doc::Pos::body(0, 0)).map(|p| p.props_of_char(0).clone()).unwrap_or_default();
        assert_eq!((c.small_caps, c.caps, c.size), (Some(false), Some(true), Some(14.0)));
    }

    /// A dialog was pinned by its anchor, so dragging its title bar did nothing (#732).
    #[test]
    fn dragging_a_dialog_title_bar_moves_it() {
        let mut app = app_with("Some text");
        app.dialog = Dialog::open("paragraph", &mut app);
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        for _ in 0..4 {
            h.step();
        }
        let id = egui::Id::new(("dialog", "Paragraph"));
        let before = h.ctx.memory(|m| m.area_rect(id)).expect("dialog is open");
        let screen = h.ctx.content_rect();
        assert!((before.center().x - screen.center().x).abs() <= 1.0, "opens centred: {before:?} in {screen:?}");
        let grab = egui::pos2(before.center().x, before.top() + 10.0);
        let drop = grab + vec2(-200.0, 120.0);
        let button =
            |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
        h.event(egui::Event::PointerMoved(grab));
        h.step();
        h.event(button(grab, true));
        h.step();
        h.event(egui::Event::PointerMoved(drop));
        h.step();
        h.event(button(drop, false));
        for _ in 0..4 {
            h.step();
        }
        let after = h.ctx.memory(|m| m.area_rect(id)).expect("dialog is still open");
        assert!((after.min.x - (before.min.x - 200.0)).abs() <= 1.0, "moved left: {before:?} -> {after:?}");
        assert!((after.min.y - (before.min.y + 120.0)).abs() <= 1.0, "moved down: {before:?} -> {after:?}");
    }

    // ---- Right-to-left dialogs (L10N-AR.11) ----

    fn ar() -> crate::i18n::Lang {
        crate::i18n::Lang::from_code("ar").unwrap()
    }

    /// The whole app at 1440×900 in `lang`, after `setup`, run for four frames.
    fn rtl_harness(lang: &str, setup: impl FnOnce(&mut WordApp)) -> egui_kittest::Harness<'static, WordApp> {
        let mut app = app_with("Some text for the dialogs");
        app.run("ui.language", json!({"value": lang})).unwrap();
        setup(&mut app);
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(4);
        h
    }

    /// The rect of the node labelled `label` inside the open dialog's window.
    fn in_dialog(h: &egui_kittest::Harness<'_, WordApp>, title: &str, label: &str) -> egui::Rect {
        use egui_kittest::kittest::Queryable;
        let area = h.ctx.memory(|m| m.area_rect(egui::Id::new(("dialog", title)))).expect("the dialog is open");
        let found: Vec<egui::Rect> = h.query_all_by_label(label).map(|n| n.rect()).filter(|r| area.contains_rect(*r)).collect();
        assert_eq!(found.len(), 1, "{label:?} once in {title}: {found:?}");
        found[0]
    }

    /// (OK, Cancel) button rects of the Paragraph dialog in `lang`.
    fn paragraph_buttons(lang: &str) -> (egui::Rect, egui::Rect) {
        use egui_kittest::kittest::Queryable;
        let h = rtl_harness(lang, |a| {
            a.run("ui.dialog", json!({"name": "paragraph"})).unwrap();
        });
        let l = crate::i18n::Lang::from_code(lang).unwrap();
        let ok = h.get_by_role_and_label(egui::accesskit::Role::Button, crate::i18n::tr(l, "OK")).rect();
        let cancel = h.get_by_role_and_label(egui::accesskit::Role::Button, crate::i18n::tr(l, "Cancel")).rect();
        crate::i18n::set_current(crate::i18n::Lang::EN);
        (ok, cancel)
    }

    /// The width of an auto-sized dialog-like window holding `form`, over 12 frames, in `lang`.
    fn form_window_widths(lang: &str, form: fn(&mut Ui)) -> Vec<f32> {
        let set = move || crate::i18n::set_current(crate::i18n::Lang::from_code(lang).unwrap());
        set();
        let mut h = egui_kittest::Harness::builder().with_size(vec2(1440.0, 900.0)).build_ui(move |ui| {
            set();
            let ctx = ui.ctx().clone();
            egui::Window::new("w").id(egui::Id::new("form_window")).collapsible(false).resizable(false).show(&ctx, |ui| {
                ui.set_min_width(340.0);
                if rtl::is_rtl() {
                    rtl::column(ui, form);
                } else {
                    form(ui);
                }
                let _ = buttons(ui, "OK");
            });
        });
        let widths = (0..12)
            .map(|_| {
                h.run();
                h.ctx.memory(|m| m.area_rect(egui::Id::new("form_window"))).map_or(0.0, |a| a.width())
            })
            .collect();
        crate::i18n::set_current(crate::i18n::Lang::EN);
        widths
    }

    /// A first frame where right-to-left grid cells spilled past the grid's left edge, or a fitted
    /// box narrower than its grid, widened the dialog for good (Table Properties was ~960 pt wide).
    #[test]
    fn rtl_form_grids_keep_their_dialog_as_narrow_as_in_english() {
        let forms: [(&str, fn(&mut Ui)); 2] = [
            ("check box row", |ui| {
                let (mut on, mut v) = (true, 1.0f32);
                rtl::fitted(ui, "g", |ui| {
                    rtl::grid("g", 2).show(ui, |ui| {
                        checked_row(ui, &mut on, "Preferred width:", |ui, on| {
                            ui.add_enabled(on, egui::DragValue::new(&mut v));
                        });
                    });
                });
            }),
            ("radio row", |ui| {
                let mut a = 1;
                rtl::fitted(ui, "g", |ui| {
                    rtl::grid("g", 2).show(ui, |ui| {
                        rtl::grid_row(
                            ui,
                            [&mut |ui: &mut Ui| label(ui, "Alignment:"), &mut |ui: &mut Ui| {
                                rtl::row(ui, |ui| {
                                    rtl::radio_value(ui, &mut a, 1, "Left");
                                    rtl::radio_value(ui, &mut a, 2, "Right");
                                });
                            }],
                        );
                    });
                });
            }),
        ];
        for (name, form) in forms {
            let en = form_window_widths("en", form);
            let ar = form_window_widths("ar", form);
            assert!(ar.iter().all(|w| (w - en[0]).abs() <= 0.5), "{name}: arabic widths {ar:?} vs english {en:?}");
        }
    }

    #[test]
    fn dialog_ok_is_rightmost_in_arabic() {
        let (ok, cancel) = paragraph_buttons("ar");
        assert!(ok.left() >= cancel.right() - 0.5, "arabic: OK ({ok:?}) is right of Cancel ({cancel:?})");
        let (ok, cancel) = paragraph_buttons("en");
        assert!(ok.right() <= cancel.left() + 0.5, "english: OK ({ok:?}) is left of Cancel ({cancel:?})");
    }

    /// (the Left: indent label, its field) in the Paragraph dialog in `lang`.
    fn paragraph_left_indent(lang: &str) -> (egui::Rect, egui::Rect) {
        use egui_kittest::kittest::Queryable;
        let h = rtl_harness(lang, |a| {
            a.run("ui.dialog", json!({"name": "paragraph"})).unwrap();
        });
        let l = crate::i18n::Lang::from_code(lang).unwrap();
        let label = in_dialog(&h, "Paragraph", crate::i18n::tr(l, "Left:"));
        // The spin field on the label's row nearest to it.
        let field = h
            .query_all_by_role(egui::accesskit::Role::SpinButton)
            .map(|n| n.rect())
            .filter(|r| (r.center().y - label.center().y).abs() < 6.0)
            .min_by(|a, b| (a.center().x - label.center().x).abs().total_cmp(&(b.center().x - label.center().x).abs()))
            .expect("a field on the label's row");
        crate::i18n::set_current(crate::i18n::Lang::EN);
        (label, field)
    }

    #[test]
    fn paragraph_dialog_labels_sit_right_of_their_fields_in_arabic() {
        let (label, field) = paragraph_left_indent("ar");
        assert!(field.right() <= label.left() + 0.5, "arabic: the field ({field:?}) is left of its label ({label:?})");
        let (label, field) = paragraph_left_indent("en");
        assert!(field.left() >= label.right() - 0.5, "english: the field ({field:?}) is right of its label ({label:?})");
    }

    #[test]
    fn options_language_list_shows_arabic_in_its_own_name() {
        use egui_kittest::kittest::Queryable;
        let mut h = rtl_harness("ar", |a| {
            a.run("file.options", json!({"pane": "language"})).unwrap();
        });
        // The row reads from the right: the caption, then the list to its left.
        let caption = h.get_by_label(crate::i18n::tr(ar(), "Interface language:")).rect();
        let combo = h.get_by_role(egui::accesskit::Role::ComboBox);
        let list = combo.rect();
        assert!(list.right() <= caption.left() + 0.5, "the list ({list:?}) is left of its caption ({caption:?})");
        combo.click();
        h.run_steps(4);
        // A single Arabic word draws as it is stored (epaint shapes the run right to left).
        let items: Vec<egui::Rect> = h.query_all_by_label("العربية").map(|n| n.rect()).collect();
        crate::i18n::set_current(crate::i18n::Lang::EN);
        assert!(!items.is_empty(), "the list has العربية in its own name");
    }

    /// Every dialog of `dialogs`, `dialogs_para`, `dialogs_layout`, `dialogs_insert` and
    /// `dialogs_lists` (plus File › Options and the built-in file browser) for a few frames in
    /// Arabic: nothing panics and every window stays inside the screen.
    #[test]
    fn every_part1_dialog_opens_in_arabic_without_panicking() {
        use egui_kittest::kittest::Queryable;
        const NAMES: &[&str] = &[
            "font",
            "paragraph",
            "asianTypography",
            "tabs",
            "borders",
            "pageBorders",
            "shading",
            "find",
            "replace",
            "goto",
            "insertTable",
            "pageSetup",
            "link",
            "bookmark",
            "wordCount",
            "zoom",
            "watermark",
            "phonetic",
            "newStyle",
            "manageStyles",
            "newTableStyle",
            "modifyTableStyle",
            "commands",
            "icons",
            "tableProperties",
            "enclose",
            "pasteSpecial",
            "about",
            "contributors",
            "models",
            "recipientList",
            "newRecipientList",
            "insertMergeField",
            "findRecipient",
            "fieldCode",
            "encryptPassword",
            "ruleIf",
            "ruleSkipIf",
            "matchFields",
            "checkErrors",
            "defineList",
            "trackChangesOptions",
            "lineNumbers",
            "hyphenation",
            "manualHyphenation",
            "language",
            "envelopes",
            "labels",
            "columns",
            "symbol",
            "specialCharacters",
            "field",
        ];
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1440.0, 900.0));
        let check = |h: &egui_kittest::Harness<'_, WordApp>, what: &str, title: &str| {
            let area = h.ctx.memory(|m| m.area_rect(egui::Id::new(("dialog", title))));
            let area = area.unwrap_or_else(|| panic!("{what}: the dialog is shown"));
            assert!(area.is_finite() && screen.expand(1.0).contains_rect(area), "{what}: {area:?} inside the screen");
            area
        };
        // About as wide as in English (Arabic labels are a little longer or shorter): a dialog
        // that came out much wider kept a first frame's spill (Table Properties was ~960 pt).
        let english_width = |name: &str| {
            let h = rtl_harness("en", |a| {
                if name.contains("able") && name != "insertTable" {
                    a.run("insert.table", json!({"rows": 2, "cols": 2})).unwrap();
                }
                a.run("ui.dialog", json!({"name": name})).unwrap();
            });
            let title = h.state().dialog.as_ref().map(Dialog::title).unwrap_or("");
            h.ctx.memory(|m| m.area_rect(egui::Id::new(("dialog", title)))).map_or(0.0, |a| a.width())
        };
        let mut too_wide = Vec::new();
        for name in NAMES {
            let mut h = rtl_harness("ar", |a| {
                // A table for Table Properties and Modify Table Style, the caret in it.
                if name.contains("able") && *name != "insertTable" {
                    a.run("insert.table", json!({"rows": 2, "cols": 2})).unwrap();
                }
                a.run("ui.dialog", json!({"name": name})).unwrap();
                assert!(a.dialog.is_some(), "{name} opens");
            });
            let title = h.state().dialog.as_ref().map(Dialog::title).unwrap_or_else(|| panic!("{name} is still open"));
            let area = check(&h, name, title);
            let en = english_width(name);
            if area.width() > en * 1.3 + 40.0 {
                too_wide.push(format!("{name}: {} pt wide in arabic, {en} pt in english", area.width()));
            }
            // A second Paragraph/Font tab and the search options too.
            if let Some(Dialog::Paragraph { tab, .. }) = &mut h.state_mut().dialog {
                *tab = 1;
            }
            if let Some(Dialog::Find { extra, .. }) = &mut h.state_mut().dialog {
                extra.more = true;
            }
            h.run_steps(3);
            if h.state().dialog.is_some() {
                let _ = check(&h, name, title);
            }
        }
        assert!(too_wide.is_empty(), "{too_wide:#?}");
        // Dialogs that other code builds.
        let built = [
            Dialog::SaveChanges { name: "Document1".into(), then: "file.close".into(), params: json!({}), document: 0 },
            Dialog::Password { name: "secret.docx".into(), params: json!({}), password: Password::default(), message: "x".into() },
            Dialog::SelectTable { sheets: vec!["One".into(), "Two".into()], selected: 0, headers: true, message: String::new(), params: json!({}) },
        ];
        for d in built {
            let title = d.title();
            let h = rtl_harness("ar", |a| {
                a.dialog = Some(d);
            });
            let _ = check(&h, title, title);
        }
        let h = rtl_harness("ar", |a| {
            a.dialog = Dialog::modify_style(a, "Normal");
        });
        let _ = check(&h, "modifyStyle", "Modify Style");
        // File › Options, every pane.
        for pane in wordcraft_engine::cmd::options::PANES {
            let h = rtl_harness("ar", |a| {
                a.run("file.options", json!({"pane": pane})).unwrap();
            });
            assert!(h.query_all_by_label(crate::i18n::tr(ar(), "Options")).count() >= 1, "{pane}");
        }
        // The built-in file browser (Open and Save As).
        #[cfg(not(target_arch = "wasm32"))]
        for purpose in ["document", "save"] {
            let h = rtl_harness("ar", |a| a.browse_builtin(purpose));
            let area = h.ctx.memory(|m| m.area_rect(egui::Id::new("builtin_file_browser"))).expect("the file browser is shown");
            assert!(area.is_finite() && screen.expand(1.0).contains_rect(area), "{purpose}: {area:?}");
        }
        crate::i18n::set_current(crate::i18n::Lang::EN);
    }

    /// How a part-2 dialog ends: `Ok(primary, other)`, a confirming button in a row with Cancel
    /// (or Close): right of it in Arabic, left of it in English; or `Close`, a lone Close at the
    /// end edge (the left in Arabic, the right in English).
    #[derive(Clone, Copy)]
    enum Ends {
        Ok(&'static str, &'static str),
        Close,
    }

    /// A part-2 dialog: its name in failures, how to open it, how it ends.
    type Part2 = (&'static str, fn(&mut WordApp), Ends);

    fn named(a: &mut WordApp, name: &str) {
        a.run("ui.dialog", json!({"name": name})).unwrap();
        assert!(a.dialog.is_some(), "{name} opens");
    }

    fn refs(a: &mut WordApp, d: crate::dialogs_refs::RefsDialog) {
        a.dialog = Some(Dialog::Refs(Box::new(d)));
    }

    fn part2_dialogs() -> Vec<Part2> {
        use crate::dialogs_refs::{CaptionForm, RefsDialog, SourceForm, TocForm};
        vec![
            ("customizeKeyboard", |a| named(a, "customizeKeyboard"), Ends::Close),
            ("customizeRibbon", |a| named(a, "customizeRibbon"), Ends::Close),
            ("customizeQuickAccess", |a| named(a, "customizeQuickAccess"), Ends::Close),
            ("noteOptions", |a| named(a, "noteOptions"), Ends::Ok("Insert Footnote", "Cancel")),
            ("templates", |a| named(a, "templates"), Ends::Ok("OK", "Cancel")),
            ("organizer", |a| named(a, "organizer"), Ends::Close),
            (
                "controlProperties",
                |a| {
                    a.run("developer.dropDown", json!({})).unwrap();
                    named(a, "controlProperties");
                },
                Ends::Ok("OK", "Cancel"),
            ),
            ("insertSmartArt", |a| named(a, "insertSmartArt"), Ends::Ok("OK", "Cancel")),
            ("insertChart", |a| named(a, "insertChart"), Ends::Ok("OK", "Cancel")),
            (
                "changeChartType",
                |a| {
                    a.run("insert.chart", json!({"type": "column"})).unwrap();
                    named(a, "changeChartType");
                },
                Ends::Ok("OK", "Cancel"),
            ),
            (
                "chartData",
                |a| {
                    a.run("insert.chart", json!({"type": "column"})).unwrap();
                    named(a, "chartData");
                },
                Ends::Ok("OK", "Cancel"),
            ),
            ("caption", |a| named(a, "caption"), Ends::Ok("OK", "Cancel")),
            (
                "captionNumbering",
                |a| {
                    let f = CaptionForm::read(a);
                    refs(a, RefsDialog::CaptionNumbering { was: Box::new(f.clone()), form: f });
                },
                Ends::Ok("OK", "Cancel"),
            ),
            ("newLabel", |a| refs(a, RefsDialog::NewLabel(CaptionForm::read(a))), Ends::Ok("OK", "Cancel")),
            ("index", |a| named(a, "index"), Ends::Ok("OK", "Cancel")),
            ("markEntry", |a| named(a, "markEntry"), Ends::Ok("Mark", "Close")),
            ("toc", |a| named(a, "toc"), Ends::Ok("OK", "Cancel")),
            (
                "tocOptions",
                |a| {
                    let f = TocForm::read(a);
                    refs(a, RefsDialog::TocOptions { was: Box::new(f.clone()), form: f });
                },
                Ends::Ok("OK", "Cancel"),
            ),
            ("tocStyles", |a| refs(a, RefsDialog::TocStyles { form: TocForm::read(a), level: 1 }), Ends::Ok("OK", "Cancel")),
            ("sourceManager", |a| named(a, "sourceManager"), Ends::Close),
            ("createSource", |a| named(a, "createSource"), Ends::Ok("OK", "Cancel")),
            (
                "editSource",
                |a| {
                    let mut f = SourceForm::new(false, true);
                    f.edit = true;
                    refs(a, RefsDialog::Source(Box::new(f)));
                },
                Ends::Ok("OK", "Cancel"),
            ),
            ("compare", |a| named(a, "compare"), Ends::Ok("OK", "Cancel")),
            ("combine", |a| named(a, "combine"), Ends::Ok("OK", "Cancel")),
            ("autoCorrect", |a| named(a, "autoCorrect"), Ends::Ok("OK", "Cancel")),
        ]
    }

    /// The buttons labelled `label` inside `area` (the open dialog's window).
    fn dialog_buttons(h: &egui_kittest::Harness<'_, WordApp>, area: egui::Rect, label: &str) -> Vec<egui::Rect> {
        use egui_kittest::kittest::Queryable;
        h.query_all_by_role_and_label(egui::accesskit::Role::Button, label).map(|n| n.rect()).filter(|r| area.contains_rect(*r)).collect()
    }

    /// Every dialog of `dialogs_chart`, `dialogs_controls`, `dialogs_keyboard`, `dialogs_notes`,
    /// `dialogs_refs`, `dialogs_review`, `dialogs_ribbon`, `dialogs_smart_art` and
    /// `dialogs_templates`, for a few frames in Arabic and in English: nothing panics, every
    /// window is finite, inside the screen whenever it is in English, no more than 40 pt wider in
    /// Arabic, and it ends mirrored:
    /// OK right of Cancel (a lone Close at the left) in Arabic, the reverse in English.
    #[test]
    fn every_dialog_opens_in_arabic_without_panicking_and_ok_is_rightmost() {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1440.0, 900.0));
        let mut failures = Vec::new();
        for (name, setup, ends) in part2_dialogs() {
            let mut widths = Vec::new();
            let mut inside = Vec::new();
            for lang in ["ar", "en"] {
                let mut h = rtl_harness(lang, setup);
                h.run_steps(3);
                let l = crate::i18n::Lang::from_code(lang).unwrap();
                let title = h.state().dialog.as_ref().map(Dialog::title).unwrap_or_else(|| panic!("{name} ({lang}) is still open"));
                let area = h.ctx.memory(|m| m.area_rect(egui::Id::new(("dialog", title))));
                let area = area.unwrap_or_else(|| panic!("{name} ({lang}): the dialog is shown"));
                if !area.is_finite() {
                    failures.push(format!("{name} ({lang}): {area:?}"));
                    continue;
                }
                inside.push(screen.expand(1.0).contains_rect(area));
                widths.push(area.width());
                let arabic = lang == "ar";
                match ends {
                    Ends::Ok(primary, other) => {
                        let ok = dialog_buttons(&h, area, crate::i18n::tr(l, primary));
                        let cancel = dialog_buttons(&h, area, crate::i18n::tr(l, other));
                        let pair = ok.iter().find_map(|o| cancel.iter().find(|c| (c.center().y - o.center().y).abs() < 6.0).map(|c| (*o, *c)));
                        match pair {
                            None => failures.push(format!("{name} ({lang}): no {primary} and {other} in one row ({ok:?}, {cancel:?})")),
                            Some((o, c)) if arabic && o.left() < c.right() - 0.5 => {
                                failures.push(format!("{name} (ar): {primary} {o:?} is not right of {other} {c:?}"));
                            }
                            Some((o, c)) if !arabic && o.right() > c.left() + 0.5 => {
                                failures.push(format!("{name} (en): {primary} {o:?} is not left of {other} {c:?}"));
                            }
                            _ => {}
                        }
                    }
                    Ends::Close => {
                        let close = dialog_buttons(&h, area, crate::i18n::tr(l, "Close"));
                        match close.last() {
                            None => failures.push(format!("{name} ({lang}): no Close button")),
                            Some(c) if arabic == (c.center().x > area.center().x) => {
                                failures.push(format!("{name} ({lang}): Close {c:?} is on the wrong side of {area:?}"));
                            }
                            _ => {}
                        }
                    }
                }
            }
            // Insert SmartArt runs past the bottom in English too (its separators stretch it): a
            // pre-existing layout bug, not a mirroring one.
            if let [false, true] = inside[..] {
                failures.push(format!("{name}: inside the screen in english, not in arabic"));
            }
            if let [ar, en] = widths[..]
                && ar > en + 40.0
            {
                failures.push(format!("{name}: {ar} pt wide in arabic, {en} pt in english"));
            }
        }
        crate::i18n::set_current(crate::i18n::Lang::EN);
        assert!(failures.is_empty(), "{failures:#?}");
    }

    /// The other windows and pages of part 2 in Arabic and English: Read Aloud (its Speed caption
    /// at the start edge, the slider after it), File › Print (settings at the start edge, the
    /// preview after them), the Restrict Editing pane and the SmartArt Text Pane. Nothing panics
    /// and each reads from the right in Arabic, from the left in English.
    #[test]
    fn part2_windows_and_panes_mirror_in_arabic() {
        use egui_kittest::kittest::Queryable;
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1440.0, 900.0));
        let mut player_widths = Vec::new();
        for lang in ["ar", "en"] {
            let arabic = lang == "ar";
            let l = crate::i18n::Lang::from_code(lang).unwrap();
            // Read Aloud.
            let h = rtl_harness(lang, |a| {
                a.run("review.readAloud", json!({})).unwrap();
                a.session.read_aloud.open = true;
            });
            let area = h.ctx.memory(|m| m.area_rect(egui::Id::new("read-aloud-player"))).expect("the player is shown");
            assert!(area.is_finite() && screen.expand(1.0).contains_rect(area), "{lang}: player {area:?}");
            player_widths.push(area.width());
            let caption = h.get_by_label(crate::i18n::tr(l, "Speed")).rect();
            let slider =
                h.query_all_by_role(egui::accesskit::Role::Slider).map(|n| n.rect()).find(|r| area.contains_rect(*r)).expect("the speed slider");
            assert_eq!(slider.center().x < caption.center().x, arabic, "{lang}: slider {slider:?}, caption {caption:?}");
            // The caption hugs the start edge of the player (its right under RTL).
            let gap = if arabic { area.right() - caption.right() } else { caption.left() - area.left() };
            assert!(gap < 20.0, "{lang}: Speed {caption:?} is {gap} pt from the start edge of {area:?}");
            // File › Print: the settings column (its Save as PDF button) at the start edge.
            let h = rtl_harness(lang, |a| {
                a.run("ui.backstage", json!({"value": true, "page": "print"})).unwrap();
            });
            let save = h.get_by_role_and_label(egui::accesskit::Role::Button, crate::i18n::tr(l, "Save these pages as PDF…")).rect();
            assert_eq!(save.center().x > screen.center().x, arabic, "{lang}: save {save:?}");
            // Restrict Editing (docked at the end edge) and the SmartArt Text Pane.
            let h = rtl_harness(lang, |a| {
                a.run("ui.dialog", json!({"name": "restrictEditing"})).unwrap();
                a.run("insert.smartArt", json!({"layout": "hierarchy"})).unwrap();
                a.run("smartArt.textPane", json!({"value": true})).unwrap();
            });
            let limit = h.get_by_label(crate::i18n::tr(l, "Limit formatting to a selection of styles")).rect();
            assert_eq!(limit.center().x < screen.center().x, arabic, "{lang}: restrict pane check box {limit:?}");
            let add = h.query_all_by_role_and_label(egui::accesskit::Role::Button, crate::i18n::tr(l, "Promote")).count();
            assert_eq!(add, 1, "{lang}: the text pane is shown");
        }
        crate::i18n::set_current(crate::i18n::Lang::EN);
        // Right-to-left rows wider than the player's 236 pt spilled past its left edge and the
        // window kept growing (~412 pt against ~338 pt in English).
        if let [ar, en] = player_widths[..] {
            assert!(ar <= en + 40.0, "the player is {ar} pt wide in arabic, {en} pt in english");
        }
    }
}
