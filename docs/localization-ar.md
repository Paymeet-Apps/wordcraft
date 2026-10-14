# Arabic interface

> **Last reviewed:** 2026-10-14 · **Last updated:** 2026-10-14 · **Change:** minor (first Arabic catalog, `ar`) · **Target:** WordCraft 0.7.x

Choose **File › Options › Interface language › العربية**, or follow an Arabic system display
language (`ar`, `ar-SA`, `ar_EG.UTF-8`, any `ar-*`). `ui.language` with `{"value":"ar"}` selects it
explicitly. Document text, command ids, CLI/MCP/control responses and file paths do not change.

`crates/ui-egui/src/i18n/ar.tsv` is written in Modern Standard Arabic from the meaning of the English
labels (clean-room; no product localization consulted). It covers every ribbon tab, group, command
label and location segment, File/backstage navigation, the status bar and dialog buttons. Strings
not yet translated show in English. The catalog is written in logical order; the parser stores it in
draw order (`bidi::visual`). The layout is not mirrored yet (later tasks of the plan).

## Conventions

- Digits stay Western (0-9). Arabic punctuation (، ؟ ؛) in running text; trailing `…` kept.
- Counts use "noun: {n}" wording (`الكلمات: {words}`): Arabic has six plural forms and the catalog has none.
- Commands are verbal nouns (`حفظ`, `نسخ`, `لصق`); placeholders never take a prefix or ending.
- Product and technical names stay Latin (Zotero, Discord, SmartArt, WordArt, PDF, PNG, Unicode).
- Left/Right in commands name the physical side (`يسار`, `يمين`).

## Terminology

| Source | Arabic |
|---|---|
| File / Home / Insert / Draw / Design / Layout | ملف / الصفحة الرئيسية / إدراج / رسم / تصميم / تخطيط |
| References / Mailings / Review / View / Help | المراجع / المراسلات / مراجعة / عرض / مساعدة |
| Save / Save As / Open / New / Close / Print | حفظ / حفظ باسم / فتح / جديد / إغلاق / طباعة |
| Undo / Redo / Repeat | تراجع / إعادة / تكرار |
| Cut / Copy / Paste / Clipboard | قص / نسخ / لصق / الحافظة |
| Bold / Italic / Underline / Strikethrough | غامق / مائل / تسطير / يتوسطه خط |
| Subscript / Superscript | منخفض / مرتفع |
| Font / Font Size / Font Color | الخط / حجم الخط / لون الخط |
| Paragraph / Style / Styles | الفقرة / النمط / الأنماط |
| Heading / Title (style) | عنوان / العنوان الرئيسي |
| Header / Footer | الرأس / التذييل |
| Footnote / Endnote | حاشية سفلية / حاشية ختامية |
| Table / Row / Column / Cell | جدول / صف / عمود / خلية |
| Picture / Shape / Chart / Text Box | صورة / شكل / مخطط / مربع نص |
| Margins / Orientation / Page Setup | الهوامش / الاتجاه / إعداد الصفحة |
| Line Spacing / Indent | تباعد الأسطر / مسافة بادئة |
| Bullets / Numbering / Multilevel List | تعداد نقطي / ترقيم / قائمة متعددة المستويات |
| Find / Replace / Select | بحث / استبدال / تحديد |
| Track Changes / Comments | تتبع التغييرات / التعليقات |
| Accept / Reject | قبول / رفض |
| Spelling & Grammar / Thesaurus | الإملاء والنحو / مرادفات |
| Word Count / Language | عدد الكلمات / اللغة |
| Table of Contents / Index / Bibliography | جدول المحتويات / الفهرس / قائمة المراجع |
| Citation / Cross-reference / Bookmark | استشهاد / إسناد مرجعي / إشارة مرجعية |
| Change to Display / Inline (equation) | تحويل إلى معادلة مستقلة / ضمن السطر |
| Research (kept distinct from Search) | بحث معرفي |
| Watermark / Page Color / Page Borders | علامة مائية / لون الصفحة / حدود الصفحة |
| Zoom / Ruler / Gridlines | تكبير/تصغير / المسطرة / خطوط الشبكة |
| Print Layout / Web Layout / Read Mode | تخطيط الطباعة / تخطيط ويب / وضع القراءة |
| Options / Properties / Info | خيارات / الخصائص / معلومات |
| OK / Cancel / Apply / Yes / No | موافق / إلغاء / تطبيق / نعم / لا |

## Revision history

- 2026-10-14: first version with the `ar` catalog (L10N-AR.5).
- 2026-10-14: wording fixes (Title, Citation, outlines, equation display) (L10N-AR.5).
