//! Interfața grafică.

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::text::{CCursor, CCursorRange, LayoutJob};
use egui::{
    Align, Align2, Color32, CornerRadius, FontFamily, FontId, Key, KeyboardShortcut, Layout,
    Margin, Modifiers, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind, TextBuffer,
    TextFormat, Theme, ThemePreference, Ui, Vec2, ViewportCommand, pos2, vec2,
};

use crate::ant;
use crate::antfile;
use crate::i18n::{Keywords, Lang, Strings};
use crate::lang::{self, CompileError, TokenClass};
use crate::table_edit::{Cursor as TableCursor, Drop, Grab, TypeKey, type_key};
use crate::vm::{RuntimeError, Tick, Vm};
use crate::world::{Dir, MAX_SIZE, MIN_SIZE, Move, Table, World};

pub const EXAMPLES: &[(&str, &str)] = &[
    ("Primul program", include_str!("../examples/primul.ant")),
    ("Conferința", include_str!("../examples/conferinta.ant")),
    ("SVI 99", include_str!("../examples/svi99.ant")),
    ("Margini (RO)", include_str!("../examples/Margini(Rom).ant")),
    ("Borders (EN)", include_str!("../examples/Margini(Eng).ant")),
    ("Края (RU)", include_str!("../examples/Margini(Rus).ant")),
];

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    pub lang: Lang,
    pub theme: ThemeChoice,
    pub speed: f32,
    pub instant: bool,
    pub show_grid: bool,
    pub show_trail: bool,
    /// Analiza programului în timp ce elevul scrie.
    pub live_check: bool,
    pub code: String,
    /// Tabela de pornire (se salvează în fișier împreună cu programul).
    pub table: Table,
    pub path: Option<PathBuf>,
    pub dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let first = antfile::parse(EXAMPLES[0].1);
        Settings {
            lang: Lang::Ro,
            theme: ThemeChoice::System,
            speed: 5.0,
            instant: false,
            show_grid: true,
            show_trail: true,
            live_check: true,
            code: first.code,
            table: first.table,
            path: None,
            dirty: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Idle,
    Running,
    Paused,
}

#[derive(Clone)]
enum Status {
    Ready,
    Running,
    Paused,
    Finished,
    Stopped,
    /// Rezultatul butonului „Verifică”: (erori, avertismente).
    Checked(usize, usize),
    /// Prima eroare și numărul total de erori.
    Compile(CompileError, usize),
    Runtime(RuntimeError, usize),
    File(String),
}

struct Anim {
    mv: Move,
    start: f64,
    dur: f64,
}

#[derive(Clone, Copy)]
enum Pending {
    New,
    Open,
    Example(usize),
    Quit,
}

pub fn decode_png(bytes: &[u8]) -> egui::ColorImage {
    let img = image::load_from_memory(bytes)
        .expect("imagine invalidă")
        .to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw())
}

/// Culorile interfeței pentru tema curentă.
struct Pal {
    accent: Color32,
    grid: Color32,
    border: Color32,
    field_bg: Color32,
    cell_bg: Color32,
    trail: Color32,
    path: Color32,
    tile: Color32,
    tile_border: Color32,
    tile_text: Color32,
    muted: Color32,
    ok: Color32,
    err: Color32,
    warn: Color32,
    current_line: Color32,
    error_line: Color32,
    action: Color32,
    control: Color32,
    cond: Color32,
    number: Color32,
    name: Color32,
    comment: Color32,
    text: Color32,
}

impl Pal {
    fn new(dark: bool) -> Pal {
        if dark {
            Pal {
                accent: Color32::from_rgb(0xF0, 0x8A, 0x3C),
                grid: Color32::from_rgb(0x34, 0x39, 0x44),
                border: Color32::from_rgb(0x5A, 0x62, 0x72),
                field_bg: Color32::from_rgb(0x20, 0x24, 0x2C),
                cell_bg: Color32::from_rgb(0x26, 0x2B, 0x34),
                trail: Color32::from_rgba_unmultiplied(0x5B, 0xD1, 0x8C, 0x2E),
                path: Color32::from_rgba_unmultiplied(0x5B, 0xD1, 0x8C, 0x90),
                tile: Color32::from_rgb(0x3D, 0x34, 0x24),
                tile_border: Color32::from_rgb(0xC9, 0x96, 0x3A),
                tile_text: Color32::from_rgb(0xFF, 0xE2, 0xA8),
                muted: Color32::from_rgb(0x8A, 0x92, 0xA0),
                ok: Color32::from_rgb(0x5B, 0xD1, 0x8C),
                err: Color32::from_rgb(0xFF, 0x6B, 0x6B),
                warn: Color32::from_rgb(0xE8, 0xB0, 0x4A),
                current_line: Color32::from_rgba_unmultiplied(0xF0, 0x8A, 0x3C, 0x38),
                error_line: Color32::from_rgba_unmultiplied(0xFF, 0x50, 0x50, 0x40),
                action: Color32::from_rgb(0x6C, 0xB4, 0xFF),
                control: Color32::from_rgb(0xF0, 0x8A, 0x3C),
                cond: Color32::from_rgb(0xC5, 0x92, 0xFF),
                number: Color32::from_rgb(0x5B, 0xD1, 0x8C),
                name: Color32::from_rgb(0xE8, 0xC5, 0x4A),
                comment: Color32::from_rgb(0x74, 0x7C, 0x8A),
                text: Color32::from_rgb(0xDD, 0xE1, 0xE8),
            }
        } else {
            Pal {
                accent: Color32::from_rgb(0xE0, 0x6C, 0x1E),
                grid: Color32::from_rgb(0xE3, 0xE0, 0xD8),
                border: Color32::from_rgb(0xA8, 0xA2, 0x96),
                field_bg: Color32::from_rgb(0xFF, 0xFD, 0xF8),
                cell_bg: Color32::from_rgb(0xFF, 0xFD, 0xF8),
                trail: Color32::from_rgba_unmultiplied(0x1E, 0x8E, 0x4E, 0x24),
                path: Color32::from_rgba_unmultiplied(0x1E, 0x8E, 0x4E, 0x80),
                tile: Color32::from_rgb(0xFF, 0xF1, 0xD0),
                tile_border: Color32::from_rgb(0xD0, 0x9A, 0x3C),
                tile_text: Color32::from_rgb(0x5A, 0x3A, 0x00),
                muted: Color32::from_rgb(0x6E, 0x6A, 0x62),
                ok: Color32::from_rgb(0x1E, 0x8E, 0x4E),
                err: Color32::from_rgb(0xD0, 0x2F, 0x2F),
                warn: Color32::from_rgb(0xB4, 0x6A, 0x00),
                current_line: Color32::from_rgba_unmultiplied(0xF5, 0x9E, 0x2B, 0x40),
                error_line: Color32::from_rgba_unmultiplied(0xE0, 0x30, 0x30, 0x30),
                action: Color32::from_rgb(0x1F, 0x5F, 0xD6),
                control: Color32::from_rgb(0xC2, 0x55, 0x0C),
                cond: Color32::from_rgb(0x7C, 0x3A, 0xED),
                number: Color32::from_rgb(0x15, 0x80, 0x3D),
                name: Color32::from_rgb(0x8A, 0x55, 0x00),
                comment: Color32::from_rgb(0x8C, 0x88, 0x80),
                text: Color32::from_rgb(0x22, 0x24, 0x28),
            }
        }
    }

    /// Culorile furnicii; după o eroare e roșie.
    fn ant(&self, crashed: bool) -> ant::Colors {
        let dark = self.cell_bg.r() < 0x80;
        let (body, legs) = match (crashed, dark) {
            (true, _) => (self.err, self.err.gamma_multiply(0.75)),
            (false, false) => (
                Color32::from_rgb(0xB4, 0x48, 0x1A),
                Color32::from_rgb(0x6E, 0x2A, 0x0C),
            ),
            (false, true) => (
                Color32::from_rgb(0xF0, 0x8A, 0x3C),
                Color32::from_rgb(0xD2, 0x69, 0x2A),
            ),
        };
        ant::Colors {
            body,
            legs: legs.to_opaque(),
            shine: Color32::from_white_alpha(70),
            eyes: Color32::from_rgb(0xFF, 0xF6, 0xEC),
        }
    }

    fn token(&self, c: TokenClass) -> Color32 {
        match c {
            TokenClass::Plain => self.text,
            TokenClass::Action => self.action,
            TokenClass::Control => self.control,
            TokenClass::Condition => self.cond,
            TokenClass::Number => self.number,
            TokenClass::Name => self.name,
            TokenClass::Comment => self.comment,
        }
    }
}

pub struct FurnicaApp {
    s: Settings,
    /// Tabela afișată: cea de pornire sau starea ei în timpul / după rulare.
    world: World,
    /// Tabela afișată este rezultatul unei rulări (diferă de cea de pornire).
    has_result: bool,
    vm: Option<Vm>,
    mode: Mode,
    anim: Option<Anim>,
    line: Option<usize>,
    error_line: Option<usize>,
    status: Status,
    crashed: bool,
    /// Furnica sau litera trasă acum cu mausul și unde se află mausul.
    grab: Option<Grab>,
    grab_pos: Option<Pos2>,
    /// Celula tabelei în care se scrie de la tastatură.
    table_cursor: Option<TableCursor>,
    /// Colțul tabelei pe ecran și latura unei celule (din ultimul cadru).
    field_geom: (Pos2, f32),
    pending: Option<Pending>,
    show_help: bool,
    /// Selecția din editor (în caractere; goală = doar cursorul);
    /// `None` până la primul click.
    cursor: Option<Range<usize>>,
    allow_close: bool,
    title: String,
    analysis: lang::Analysis,
    /// Textul pentru care e valabilă `analysis` (`None` = neanalizat).
    analyzed: Option<String>,
    /// Porțiunea de text la care trebuie dus cursorul (click în lista de probleme).
    jump_to: Option<Range<usize>>,
    /// Editorul primește selecția `cursor` și focusul după ce e desenat
    /// (click-ul pe un buton i-ar lua focusul în același cadru).
    focus_editor: bool,
    /// Variantele propuse în timp ce elevul scrie.
    completion: Option<Completion>,
}

/// O comandă de inserat: textul dinaintea exemplului, exemplul (rămâne
/// selectat, ca următoarea comandă să-l înlocuiască) și textul de după.
#[derive(Clone, Default)]
struct Snippet {
    before: String,
    demo: String,
    after: String,
}

impl Snippet {
    fn word(word: &str) -> Snippet {
        Snippet {
            before: word.into(),
            ..Snippet::default()
        }
    }

    /// Un bloc cu `demo` ca exemplu în interior.
    fn block(head: &str, demo: &str, end: &str) -> Snippet {
        Snippet {
            before: format!("{head}\n    "),
            demo: demo.into(),
            after: format!("\n{end}"),
        }
    }

    /// DACĂ pe mai multe rânduri; comanda de după ATUNCI e selectată.
    fn if_else(kw: &Keywords, cond: &str) -> Snippet {
        Snippet {
            before: format!("{} {cond} {}\n    ", kw.if_, kw.then),
            demo: kw.up.into(),
            after: format!("\n{}\n    {}", kw.else_, kw.down),
        }
    }

    fn text(&self) -> String {
        [&self.before, &self.demo, &self.after]
            .map(String::as_str)
            .concat()
    }

    /// Primul rând, cu „…” dacă blocul continuă.
    fn label(&self) -> String {
        let text = self.text();
        match text.split_once('\n') {
            Some((first, _)) => format!("{first} …"),
            None => text,
        }
    }
}

/// Lista de variante deschisă sub cursor.
struct Completion {
    /// Comanda scrisă pe rând (în caractere), înlocuită de varianta aleasă.
    range: Range<usize>,
    items: Vec<Snippet>,
    selected: usize,
    /// Unde a fost desenată lista (pentru click-urile cu mausul).
    rect: Option<Rect>,
}

/// O problemă de afișat în editor și în lista de probleme.
struct Diag {
    line: usize,
    span: Range<usize>,
    error: bool,
    message: String,
    hint: Option<String>,
}

const STORAGE_KEY: &str = "furnica";
const EDITOR_ID: &str = "editor";

impl FurnicaApp {
    pub fn new(cc: &eframe::CreationContext<'_>, open: Option<PathBuf>) -> FurnicaApp {
        setup_style(&cc.egui_ctx);
        let mut s: Settings = cc
            .storage
            .and_then(|st| eframe::get_value(st, STORAGE_KEY))
            .unwrap_or_default();
        if !s.table.is_consistent() {
            s.table = Table::default();
        }
        let mut app = FurnicaApp {
            world: World::new(s.table.clone()),
            s,
            has_result: false,
            vm: None,
            mode: Mode::Idle,
            anim: None,
            line: None,
            error_line: None,
            status: Status::Ready,
            crashed: false,
            grab: None,
            grab_pos: None,
            table_cursor: None,
            field_geom: (Pos2::ZERO, 1.0),
            pending: None,
            show_help: false,
            cursor: None,
            allow_close: false,
            title: String::new(),
            analysis: lang::Analysis::default(),
            analyzed: None,
            jump_to: None,
            focus_editor: false,
            completion: None,
        };
        app.apply_theme(&cc.egui_ctx);
        if let Some(p) = open {
            app.load_file(&p);
        }
        app
    }

    fn t(&self) -> &'static Strings {
        self.s.lang.t()
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        ctx.set_theme(match self.s.theme {
            ThemeChoice::System => ThemePreference::System,
            ThemeChoice::Light => ThemePreference::Light,
            ThemeChoice::Dark => ThemePreference::Dark,
        });
    }

    // -------------------------------------------------------- analiză ---

    fn analyze_now(&mut self) {
        if self.analyzed.as_ref() != Some(&self.s.code) {
            self.analysis = lang::analyze(&self.s.code);
            self.analyzed = Some(self.s.code.clone());
        }
    }

    /// Ține analiza la zi cu textul (sau o șterge dacă verificarea automată e oprită).
    fn refresh_analysis(&mut self) {
        if self.analyzed.as_ref() == Some(&self.s.code) {
            return;
        }
        if self.s.live_check {
            self.analyze_now();
        } else {
            self.analysis = lang::Analysis::default();
            self.analyzed = None;
        }
    }

    fn diagnostics(&self) -> Vec<Diag> {
        let t = self.t();
        let lang = self.s.lang;
        let errors = self.analysis.errors.iter().map(|e| Diag {
            line: e.line,
            span: e.span.clone(),
            error: true,
            message: t.compile_error(lang, &e.kind),
            hint: e.hint.clone(),
        });
        let warnings = self.analysis.warnings.iter().map(|w| Diag {
            line: w.line,
            span: w.span.clone(),
            error: false,
            message: t.warning(lang, &w.kind),
            hint: None,
        });
        let mut all: Vec<Diag> = errors.chain(warnings).collect();
        all.sort_by_key(|d| (d.line, d.span.start));
        all
    }

    /// Înlocuiește cuvântul greșit cu sugestia.
    fn apply_fix(&mut self, span: Range<usize>, hint: &str) {
        if self.s.code.get(span.clone()).is_some() {
            self.s.code.replace_range(span.clone(), hint);
            self.s.dirty = true;
            self.status = Status::Ready;
            self.jump_to = Some(span.start..span.start + hint.len());
        }
    }

    // ------------------------------------------------------- execuție ---

    fn compile(&mut self) -> Option<lang::Program> {
        self.analyze_now();
        match &self.analysis.program {
            Some(p) => Some(p.clone()),
            None => {
                let first = self.analysis.errors[0].clone();
                self.status = Status::Compile(first, self.analysis.errors.len());
                None
            }
        }
    }

    fn start(&mut self, mode: Mode) -> bool {
        self.error_line = None;
        let Some(p) = self.compile() else {
            return false;
        };
        self.world = World::new(self.s.table.clone());
        self.has_result = true;
        self.vm = Some(Vm::new(p));
        self.mode = mode;
        self.anim = None;
        self.crashed = false;
        self.line = None;
        self.status = if mode == Mode::Running {
            Status::Running
        } else {
            Status::Paused
        };
        true
    }

    fn run(&mut self) {
        match self.mode {
            Mode::Idle => {
                self.start(Mode::Running);
            }
            Mode::Paused => {
                self.mode = Mode::Running;
                self.status = Status::Running;
            }
            Mode::Running => {}
        }
    }

    fn pause(&mut self) {
        if self.mode == Mode::Running {
            self.mode = Mode::Paused;
            self.status = Status::Paused;
        }
    }

    fn step_once(&mut self, now: f64) {
        match self.mode {
            Mode::Idle => {
                if self.start(Mode::Paused) {
                    self.advance(now, true);
                }
            }
            Mode::Running => self.pause(),
            Mode::Paused => {
                if self.anim.is_none() {
                    self.advance(now, true);
                }
            }
        }
    }

    fn stop(&mut self) {
        if self.mode != Mode::Idle {
            self.finish(Status::Stopped);
        }
    }

    /// Înapoi la tabela de pornire.
    fn reset(&mut self) {
        self.vm = None;
        self.mode = Mode::Idle;
        self.anim = None;
        self.line = None;
        self.error_line = None;
        self.crashed = false;
        self.world = World::new(self.s.table.clone());
        self.has_result = false;
        self.status = Status::Ready;
    }

    fn check(&mut self) {
        self.error_line = None;
        self.analyze_now();
        self.status = Status::Checked(self.analysis.errors.len(), self.analysis.warnings.len());
    }

    fn finish(&mut self, status: Status) {
        self.vm = None;
        self.mode = Mode::Idle;
        self.anim = None;
        self.line = None;
        self.status = status;
    }

    fn step_duration(&self) -> f64 {
        0.9 * 0.68f64.powf(self.s.speed as f64 - 1.0)
    }

    /// Execută până la următoarea mișcare a furnicii.
    fn advance(&mut self, now: f64, animate: bool) {
        let Some(vm) = &mut self.vm else { return };
        match vm.tick(&mut self.world, 50_000) {
            Tick::Moved(mv, line) => {
                self.line = Some(line);
                if animate {
                    self.anim = Some(Anim {
                        mv,
                        start: now,
                        dur: self.step_duration(),
                    });
                }
            }
            Tick::Busy => {}
            Tick::Finished => self.finish(Status::Finished),
            Tick::Failed(e, line) => {
                self.error_line = Some(line);
                self.crashed = matches!(e, RuntimeError::World(_));
                self.finish(Status::Runtime(e, line));
            }
        }
    }

    fn drive(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if self.anim.as_ref().is_some_and(|a| now >= a.start + a.dur) {
            self.anim = None;
        }
        if self.mode == Mode::Running && self.anim.is_none() {
            if self.s.instant {
                let deadline = std::time::Instant::now() + std::time::Duration::from_millis(12);
                while self.mode == Mode::Running && std::time::Instant::now() < deadline {
                    for _ in 0..200 {
                        self.advance(now, false);
                        if self.mode != Mode::Running {
                            break;
                        }
                    }
                }
            } else {
                self.advance(now, true);
            }
        }
        if self.mode == Mode::Running || self.anim.is_some() {
            ctx.request_repaint();
        }
    }

    // ---------------------------------------------------------- tabela ---

    /// Modifică tabela de pornire (doar când programul nu rulează).
    fn edit_table(&mut self, f: impl FnOnce(&mut Table) -> bool) {
        if self.mode != Mode::Idle {
            return;
        }
        let changed = f(&mut self.s.table);
        if changed {
            self.s.dirty = true;
        }
        if changed || self.has_result {
            self.reset();
        }
    }

    /// Tabela de după rulare devine tabela de pornire.
    fn keep_result(&mut self) {
        if self.mode == Mode::Idle && self.has_result {
            self.s.table = self.world.table.clone();
            self.s.dirty = true;
            self.reset();
        }
    }

    // --------------------------------------------------------- fișiere ---

    fn file_name(&self) -> String {
        self.s
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or_else(
                || self.t().untitled.to_string(),
                |n| n.to_string_lossy().into_owned(),
            )
    }

    fn set_file(&mut self, file: antfile::AntFile) {
        self.s.code = file.code;
        self.s.table = file.table;
        self.cursor = None;
        self.s.dirty = false;
        self.reset();
    }

    fn load_file(&mut self, path: &Path) {
        match std::fs::read(path) {
            Ok(bytes) => {
                self.set_file(antfile::parse(&antfile::decode(&bytes)));
                self.s.path = Some(path.to_path_buf());
            }
            Err(e) => self.status = Status::File(format!("{}: {e}", self.t().file_error)),
        }
    }

    fn save(&mut self, save_as: bool) -> bool {
        let path = match (&self.s.path, save_as) {
            (Some(p), false) => Some(p.clone()),
            _ => rfd::FileDialog::new()
                .add_filter(self.t().ant_files, &["ant"])
                .set_file_name(format!("{}.ant", self.file_name().trim_end_matches(".ant")))
                .save_file(),
        };
        let Some(mut path) = path else { return false };
        if path.extension().is_none() {
            path.set_extension("ant");
        }
        match std::fs::write(&path, antfile::write(&self.s.table, &self.s.code)) {
            Ok(()) => {
                self.s.path = Some(path);
                self.s.dirty = false;
                true
            }
            Err(e) => {
                self.status = Status::File(format!("{}: {e}", self.t().file_error));
                false
            }
        }
    }

    fn request(&mut self, p: Pending, ctx: &egui::Context) {
        if self.s.dirty {
            self.pending = Some(p);
        } else {
            self.perform(p, ctx);
        }
    }

    fn perform(&mut self, p: Pending, ctx: &egui::Context) {
        match p {
            Pending::New => {
                let kw = &self.t().kw;
                self.set_file(antfile::AntFile {
                    table: Table::default(),
                    code: format!("{}\n\n{}\n", kw.begin, kw.end),
                });
                self.s.path = None;
                let at = kw.begin.chars().count() + 1;
                self.cursor = Some(at..at);
            }
            Pending::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(self.t().ant_files, &["ant", "txt"])
                    .pick_file()
                {
                    self.load_file(&path);
                }
            }
            Pending::Example(i) => {
                self.set_file(antfile::parse(EXAMPLES[i].1));
                self.s.path = None;
            }
            Pending::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }

    /// Formatează programul; cursorul rămâne lângă același cuvânt, iar
    /// schimbarea se anulează cu un singur Ctrl+Z.
    fn format_code(&mut self, ctx: &egui::Context) {
        let new = lang::format_program(&self.s.code);
        if new != self.s.code {
            let sel = self.cursor.clone().unwrap_or_default();
            let map = |at| respaced_cursor(&self.s.code, &new, at);
            let cursor = map(sel.start)..map(sel.end);
            let id = egui::Id::unique(EDITOR_ID);
            let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
            // Două puncte de anulare: textul vechi și cel formatat.
            let mut undoer = state.undoer();
            let old_range = state.cursor.char_range().unwrap_or_default();
            undoer.add_undo(&(old_range, self.s.code.clone()));
            let range = CCursorRange::two(CCursor::new(cursor.start), CCursor::new(cursor.end));
            undoer.add_undo(&(range, new.clone()));
            state.set_undoer(undoer);
            state.cursor.set_char_range(Some(range));
            state.store(ctx, id);
            self.s.code = new;
            self.s.dirty = true;
            self.cursor = Some(cursor);
        }
        self.focus_editor = true;
    }

    /// Scrie comanda `before` + `demo` + `after` în program. Selecția e
    /// înlocuită de comandă; fără selecție, comanda ocupă rândul gol al
    /// cursorului sau un rând nou, deasupra ori dedesubtul lui (fără cursor:
    /// înainte de SFÎRȘIT). Procedurile de la butoane se pun înainte de ÎNCEPUT.
    /// Rândurile noi primesc indentarea blocului în care ajung, iar exemplul din
    /// interiorul comenzii (`demo`) rămâne selectat, ca următoarea comandă să-l
    /// înlocuiască. După o comandă fără exemplu, cursorul trece pe un rând nou.
    fn insert_snippet(&mut self, snippet: &Snippet, procedure: bool) {
        let Snippet {
            before,
            demo,
            after,
        } = snippet;
        let (before, demo, after) = (before.as_str(), demo.as_str(), after.as_str());
        let code = &self.s.code;
        let byte = |c: usize| code.char_indices().nth(c).map_or(code.len(), |(b, _)| b);
        let line_with = |pred: fn(&str) -> bool, last: bool| {
            let mut found = None;
            let mut at = 0;
            for l in code.split_inclusive('\n') {
                if pred(l) {
                    found = Some(at);
                    if !last {
                        break;
                    }
                }
                at += l.len();
            }
            found
        };
        let begin = line_with(lang::is_begin_line, false).filter(|_| procedure);
        // Fără cursor, comanda se adaugă înainte de SFÎRȘIT-ul programului.
        let default = line_with(lang::is_main_end_line, true).unwrap_or(code.len());
        let sel = match &self.cursor {
            Some(r) => byte(r.start)..byte(r.end),
            None => default..default,
        };
        let line_start = code[..sel.start].rfind('\n').map_or(0, |i| i + 1);
        let line_end = code[sel.start..]
            .find('\n')
            .map_or(code.len(), |i| sel.start + i);
        let (range, prefix, suffix) = if let Some(b) = begin {
            // Procedura, cu un rând gol înainte de ÎNCEPUT.
            (b..b, "", "\n\n")
        } else if !sel.is_empty() {
            (sel, "", "")
        } else if code[line_start..line_end].trim().is_empty() {
            (line_start..line_end, "", "")
        } else if code[line_start..sel.start].trim().is_empty() {
            (line_start..line_start, "", "\n")
        } else {
            (line_end..line_end, "\n", "")
        };
        let rest = &code[range.end..];
        let mut lines = rest.split('\n');
        let line_rest_blank = suffix.is_empty() && lines.next().unwrap_or("").trim().is_empty();
        // Comanda a înlocuit-o pe cea de după ATUNCI: urmează ALTFEL și comanda
        // lui, care e selectată în loc să se facă un rând nou.
        let else_follows = demo.is_empty()
            && line_rest_blank
            && lines.next().is_some_and(|l| lang::is_else(l.trim()))
            && lines.next().is_some_and(|l| {
                let l = l.trim();
                !l.is_empty() && !l.contains(char::is_whitespace)
            });
        // Rândul nou primește un cuvânt provizoriu, ca formatarea să-i dea
        // indentarea; cuvântul e scos apoi.
        let new_line = demo.is_empty() && line_rest_blank && !else_follows;
        let placeholder = if new_line { "\n_" } else { "" };

        let new = [
            &code[..range.start],
            prefix,
            before,
            demo,
            after,
            placeholder,
            suffix,
            rest,
        ]
        .concat();
        // Indentarea rândurilor atinse vine din formatare; restul rămâne.
        let first = code[..range.start].matches('\n').count() + prefix.len();
        let last = first
            + [before, demo, after, placeholder]
                .concat()
                .matches('\n')
                .count();
        let code = reindent_lines(&new, first..=last);

        let start = new[..range.start].chars().count() + prefix.len() + before.chars().count();
        let end = start + demo.chars().count();
        let (a, b) = if demo.is_empty() {
            let at = end + after.chars().count() + placeholder.len().saturating_sub(1);
            (at, at)
        } else {
            (start, end)
        };
        let mut cursor = reindented_cursor(&new, &code, a)..reindented_cursor(&new, &code, b);
        if else_follows {
            let at = code
                .char_indices()
                .nth(cursor.end)
                .map_or(code.len(), |(b, _)| b);
            let line = code[at..].split('\n').nth(2).unwrap_or("");
            let line_start: usize = code[at..]
                .split('\n')
                .take(2)
                .map(|l| l.chars().count() + 1)
                .sum::<usize>()
                + cursor.end;
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            let start = line_start + indent;
            cursor = start..start + line.trim().chars().count();
        }
        let mut code = code;
        if new_line {
            let at = code
                .char_indices()
                .nth(cursor.start)
                .map_or(code.len(), |(b, _)| b);
            code.remove(at);
        }
        self.s.code = code;
        self.s.dirty = true;
        self.cursor = Some(cursor);
        self.focus_editor = true;
    }

    /// Enter: textul de după cursor trece pe un rând nou, cu indentarea
    /// blocului; rândul părăsit e reindentat și el (SFÎRȘIT, ALTFEL se
    /// aliniază cu începutul blocului). Întoarce noua poziție a cursorului.
    fn new_line(&mut self, sel: Range<usize>) -> usize {
        let code = &self.s.code;
        let byte = |c: usize| code.char_indices().nth(c).map_or(code.len(), |(b, _)| b);
        let (a, b) = (byte(sel.start), byte(sel.end));
        let rest = code[b..].trim_start_matches([' ', '\t']);
        // Pe un rând gol, cuvântul provizoriu îi dă indentarea la formatare.
        let placeholder = if rest.split('\n').next().unwrap_or("").trim().is_empty() {
            "_"
        } else {
            ""
        };
        let new = [&code[..a], "\n", placeholder, rest].concat();
        let line = code[..a].matches('\n').count();
        let mut out = reindent_lines(&new, line..=line + 1);
        let at = reindented_cursor(&new, &out, code[..a].chars().count() + 1);
        if !placeholder.is_empty() {
            let b = out.char_indices().nth(at).map_or(out.len(), |(b, _)| b);
            out.remove(b);
        }
        self.s.code = out;
        self.cursor = Some(at..at);
        at
    }

    /// Variantele care continuă comanda `typed` scrisă la începutul rândului;
    /// `before` e textul programului de dinaintea ei.
    fn suggestions(&self, typed: &str, before: &str) -> Vec<Snippet> {
        let key = |s: &str| {
            lang::normalize(s)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        };
        let typed_key = key(typed);
        if !typed_key.chars().any(char::is_alphanumeric) {
            return Vec::new();
        }
        let kw = &self.t().kw;
        let n = typed
            .split_whitespace()
            .nth(1)
            .filter(|a| a.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or("4");
        let mut all: Vec<Snippet> = [kw.up, kw.down, kw.left, kw.right]
            .map(Snippet::word)
            .into();
        all.push(Snippet::block(
            &format!("{} {n} {}", kw.repeat, kw.times),
            kw.down,
            kw.end_repeat,
        ));
        for c in [kw.not_border, kw.border] {
            all.push(Snippet::block(
                &format!("{} {c} {}", kw.while_, kw.do_),
                kw.down,
                kw.end_while,
            ));
        }
        for c in [kw.border, kw.not_border] {
            all.push(Snippet::if_else(kw, c));
        }
        all.push(Snippet::word(kw.else_));
        all.push(Snippet::block(
            &format!("{} {}", kw.procedure, kw.name),
            kw.down,
            kw.end_procedure,
        ));
        // Procedurile se cheamă cu numele lor.
        all.extend(
            lang::procedure_names(&self.s.code)
                .iter()
                .map(|p| Snippet::word(p)),
        );
        // Sfârșitul blocului deschis e propus primul.
        let mut ends = [
            (lang::BlockKind::Repeat, kw.end_repeat),
            (lang::BlockKind::Cycle, kw.end_while),
            (lang::BlockKind::Procedure, kw.end_procedure),
        ];
        let open = lang::open_block(before);
        ends.sort_by_key(|(kind, _)| Some(*kind) != open);
        all.extend(ends.map(|(_, end)| Snippet::word(end)));
        all.push(Snippet::word(kw.begin));
        all.push(Snippet::word(kw.end));

        // Rămân variantele care adaugă ceva în afara exemplului: după
        // „DACĂ E_MARGINE ATUNCI” Enter face rând nou.
        all.retain(|s| {
            let first = key(s.text().split('\n').next().unwrap_or(""));
            first.starts_with(&typed_key) && first != typed_key && key(&s.before) != typed_key
        });
        all
    }

    /// Deschide lista de variante pentru comanda scrisă înaintea cursorului.
    fn update_completion(&mut self) {
        self.completion = None;
        let Some(cursor) = self.cursor.clone().filter(|r| r.is_empty()) else {
            return;
        };
        let code = &self.s.code;
        let end = code
            .char_indices()
            .nth(cursor.end)
            .map_or(code.len(), |(b, _)| b);
        let line_start = code[..end].rfind('\n').map_or(0, |i| i + 1);
        let typed = code[line_start..end].trim_start();
        let start = end - typed.len();
        let items = self.suggestions(typed, &code[..start]);
        if !items.is_empty() {
            self.completion = Some(Completion {
                range: code[..start].chars().count()..cursor.end,
                items,
                selected: 0,
                rect: None,
            });
        }
    }

    fn accept_completion(&mut self, i: usize) {
        if let Some(c) = self.completion.take() {
            self.cursor = Some(c.range);
            self.insert_snippet(&c.items[i], false);
        }
    }

    // ------------------------------------------------------------- UI ---

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let (f5, f6, f7, esc) = ctx.input(|i| {
            (
                i.key_pressed(Key::F5),
                i.key_pressed(Key::F6),
                i.key_pressed(Key::F7),
                i.key_pressed(Key::Escape),
            )
        });
        let shortcut = |k| {
            ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, k)))
        };
        if f5 {
            if self.mode == Mode::Running {
                self.pause()
            } else {
                self.run()
            }
        }
        if f6 {
            self.step_once(now);
        }
        if f7 {
            self.check();
        }
        if esc {
            self.stop();
        }
        if shortcut(Key::S) {
            self.save(false);
        }
        if shortcut(Key::O) {
            self.request(Pending::Open, ctx);
        }
        if shortcut(Key::N) {
            self.request(Pending::New, ctx);
        }
        let format = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::F);
        if ctx.input_mut(|i| i.consume_shortcut(&format)) && self.mode == Mode::Idle {
            self.format_code(ctx);
        }

        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if let Some(p) = dropped.into_iter().next()
            && !self.s.dirty
        {
            self.load_file(&p);
        }

        if ctx.input(|i| i.viewport().close_requested()) && self.s.dirty && !self.allow_close {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.pending = Some(Pending::Quit);
        }

        let title = format!(
            "{} — {}{}",
            self.t().title,
            self.file_name(),
            if self.s.dirty { " •" } else { "" }
        );
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        let t = self.t();
        let ctx = ui.ctx().clone();
        egui::Panel::top("top")
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(Margin::symmetric(12, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    logo(ui, 32.0);
                    let strong = ui.visuals().strong_text_color();
                    ui.label(
                        RichText::new(t.title)
                            .size(20.0)
                            .family(bold())
                            .color(strong),
                    );
                    ui.add_space(12.0);

                    if ui
                        .button(format!("🗋  {}", t.new))
                        .on_hover_text("Ctrl+N")
                        .clicked()
                    {
                        self.request(Pending::New, &ctx);
                    }
                    if ui
                        .button(format!("🗁  {}", t.open))
                        .on_hover_text("Ctrl+O")
                        .clicked()
                    {
                        self.request(Pending::Open, &ctx);
                    }
                    if ui
                        .button(format!("💾  {}", t.save))
                        .on_hover_text("Ctrl+S")
                        .clicked()
                    {
                        self.save(false);
                    }
                    if ui.button(t.save_as.to_string()).clicked() {
                        self.save(true);
                    }
                    ui.menu_button(format!("📚  {}", t.examples), |ui| {
                        for (i, (name, _)) in EXAMPLES.iter().enumerate() {
                            if ui.button(*name).clicked() {
                                self.request(Pending::Example(i), &ctx);
                            }
                        }
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(format!("❓  {}", t.help)).clicked() {
                            self.show_help = !self.show_help;
                        }
                        ui.menu_button(format!("⚙  {}", t.settings), |ui| self.settings_menu(ui));
                        ui.menu_button(format!("🌐  {}", self.s.lang.name()), |ui| {
                            for l in Lang::ALL {
                                ui.selectable_value(&mut self.s.lang, l, l.name());
                            }
                        });
                    });
                });
            });
    }

    fn settings_menu(&mut self, ui: &mut Ui) {
        let t = self.t();
        ui.set_min_width(220.0);
        ui.label(RichText::new(t.theme).strong());
        let before = self.s.theme;
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.s.theme, ThemeChoice::System, t.theme_system);
            ui.selectable_value(
                &mut self.s.theme,
                ThemeChoice::Light,
                format!("☀ {}", t.theme_light),
            );
            ui.selectable_value(
                &mut self.s.theme,
                ThemeChoice::Dark,
                format!("🌙 {}", t.theme_dark),
            );
        });
        if before != self.s.theme {
            self.apply_theme(ui.ctx());
        }
        ui.separator();
        ui.label(RichText::new(t.table).strong());
        let (mut c, mut r) = (self.s.table.width, self.s.table.height);
        ui.add_enabled_ui(self.mode == Mode::Idle, |ui| {
            ui.add(egui::Slider::new(&mut c, MIN_SIZE..=MAX_SIZE).text(t.columns));
            ui.add(egui::Slider::new(&mut r, MIN_SIZE..=MAX_SIZE).text(t.rows));
        });
        if (c, r) != (self.s.table.width, self.s.table.height) {
            self.edit_table(|tb| {
                *tb = tb.resized(c, r);
                true
            });
        }
        ui.checkbox(&mut self.s.show_grid, t.show_grid);
        ui.checkbox(&mut self.s.show_trail, t.show_trail);
        ui.separator();
        ui.checkbox(&mut self.s.live_check, t.live_check);
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        let t = self.t();
        let pal = Pal::new(ui.visuals().dark_mode);
        egui::Panel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(Margin::symmetric(14, 6)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (icon, color, text) = self.status_text(&pal);
                    ui.label(RichText::new(format!("{icon}  {text}")).color(color));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let w = &self.world;
                        let (x, y) = w.pos();
                        for (i, text) in [
                            format!("{}: {}", t.pushes, w.pushes),
                            format!("{}: {}", t.moves, w.step_count),
                            format!("{}: {}", t.direction, t.dir(w.dir())),
                            format!("{}: ({}, {})", t.position, x + 1, y + 1),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            if i > 0 {
                                ui.separator();
                            }
                            ui.label(RichText::new(text).color(pal.muted));
                        }
                    });
                });
            });
    }

    fn status_text(&self, pal: &Pal) -> (&'static str, Color32, String) {
        let t = self.t();
        let lang = self.s.lang;
        match &self.status {
            Status::Ready => ("●", pal.muted, t.ready.into()),
            Status::Running => ("▶", pal.accent, t.running.into()),
            Status::Paused => ("⏸", pal.accent, t.paused.into()),
            Status::Finished => ("✔", pal.ok, t.finished.into()),
            Status::Stopped => ("⏹", pal.muted, t.stopped.into()),
            Status::Checked(0, 0) => ("✔", pal.ok, t.no_errors.into()),
            Status::Checked(0, w) => ("✔", pal.ok, (t.summary)(0, *w)),
            Status::Checked(e, w) => ("✖", pal.err, (t.summary)(*e, *w)),
            Status::Compile(e, total) => {
                let more = if *total > 1 {
                    format!("  ({})", (t.summary)(*total, 0))
                } else {
                    String::new()
                };
                (
                    "✖",
                    pal.err,
                    format!(
                        "{} {}: {}{more}",
                        t.error_line,
                        e.line,
                        t.compile_error(lang, &e.kind)
                    ),
                )
            }
            Status::Runtime(e, line) => (
                "✖",
                pal.err,
                format!("{} {line}: {}", t.error_line, t.runtime_error(lang, *e)),
            ),
            Status::File(msg) => ("✖", pal.err, msg.clone()),
        }
    }

    fn editor_panel(&mut self, ui: &mut Ui) {
        let t = self.t();
        let kw = &t.kw;
        let pal = Pal::new(ui.visuals().dark_mode);
        let editable = self.mode == Mode::Idle;

        egui::Panel::left("editor_panel")
            .default_size(430.0)
            .size_range(300.0..=900.0)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(Margin::same(12)),
            )
            .show(ui, |ui| {
                let (title, format) = ui
                    .horizontal(|ui| {
                        let title = ui.label(RichText::new(t.program).size(16.0).family(bold()));
                        ui.label(
                            RichText::new(format!(
                                "{}{}",
                                self.file_name(),
                                if self.s.dirty { " •" } else { "" }
                            ))
                            .color(pal.muted),
                        );
                        let format = ui
                            .with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add_enabled(
                                    editable,
                                    egui::Button::new(format!("⇥  {}", t.format)),
                                )
                                .on_hover_text("Ctrl+Shift+F")
                                .clicked()
                            })
                            .inner;
                        (title.id, format)
                    })
                    .inner;
                if format {
                    self.format_code(ui.ctx());
                }
                ui.add_space(6.0);

                // Butoane pentru inserarea comenzilor: (etichetă, culoare, comandă, e procedură).
                let snippets: [(String, Color32, Snippet, bool); 8] = [
                    (kw.up.into(), pal.action, Snippet::word(kw.up), false),
                    (kw.down.into(), pal.action, Snippet::word(kw.down), false),
                    (kw.left.into(), pal.action, Snippet::word(kw.left), false),
                    (kw.right.into(), pal.action, Snippet::word(kw.right), false),
                    (
                        format!("{} N", kw.repeat),
                        pal.control,
                        Snippet::block(
                            &format!("{} 4 {}", kw.repeat, kw.times),
                            kw.down,
                            kw.end_repeat,
                        ),
                        false,
                    ),
                    (
                        kw.while_.into(),
                        pal.control,
                        Snippet::block(
                            &format!("{} {} {}", kw.while_, kw.not_border, kw.do_),
                            kw.down,
                            kw.end_while,
                        ),
                        false,
                    ),
                    (
                        kw.if_.into(),
                        pal.control,
                        Snippet::if_else(kw, kw.border),
                        false,
                    ),
                    (
                        kw.procedure.into(),
                        pal.control,
                        Snippet::block(
                            &format!("{} {}", kw.procedure, kw.name),
                            kw.down,
                            kw.end_procedure,
                        ),
                        true,
                    ),
                ];
                // Deplasările pe primul rând, structurile de control pe al doilea.
                ui.add_enabled_ui(editable, |ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    for row in [&snippets[..4], &snippets[4..]] {
                        ui.horizontal_wrapped(|ui| {
                            for (label, color, snippet, procedure) in row {
                                let b = egui::Button::new(
                                    RichText::new(label)
                                        .color(*color)
                                        .family(FontFamily::Monospace),
                                );
                                if ui.add(b).on_hover_text(snippet.text()).clicked() {
                                    self.insert_snippet(snippet, *procedure);
                                }
                            }
                        });
                    }
                });
                ui.add_space(8.0);
                self.problems_panel(ui, &pal);

                let frame_fill = if ui.visuals().dark_mode {
                    Color32::from_rgb(0x17, 0x1A, 0x20)
                } else {
                    Color32::WHITE
                };
                egui::Frame::new()
                    .fill(frame_fill)
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .corner_radius(10)
                    .inner_margin(Margin::same(4))
                    .show(ui, |ui| {
                        egui::ScrollArea::both()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                self.code_editor(ui, &pal, editable, title);
                            });
                    });
            });
    }

    fn problems_panel(&mut self, ui: &mut Ui, pal: &Pal) {
        let t = self.t();
        let diags = self.diagnostics();
        let (errors, warnings) = (self.analysis.errors.len(), self.analysis.warnings.len());
        let mut fix = None;
        // Aproximativ o treime din înălțimea disponibilă sub butoane.
        let height = (ui.available_height() * 0.32).clamp(90.0, 280.0);
        egui::Panel::bottom("problems")
            .resizable(false)
            .exact_size(height)
            .frame(egui::Frame::new().inner_margin(Margin {
                left: 0,
                right: 0,
                top: 8,
                bottom: 0,
            }))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(t.problems).size(15.0).family(bold()));
                    if errors > 0 {
                        ui.label(RichText::new(format!("✖ {errors}")).color(pal.err));
                    }
                    if warnings > 0 {
                        ui.label(RichText::new(format!("⚠ {warnings}")).color(pal.warn));
                    }
                });
                ui.add_space(2.0);
                if self.analyzed.is_none() {
                    ui.label(RichText::new(format!("F7 — {}", t.check)).color(pal.muted));
                    return;
                }
                if diags.is_empty() {
                    ui.label(RichText::new(format!("✔  {}", t.no_problems)).color(pal.ok));
                    return;
                }
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for d in &diags {
                            ui.horizontal_top(|ui| {
                                let (icon, color) = if d.error {
                                    ("✖", pal.err)
                                } else {
                                    ("⚠", pal.warn)
                                };
                                ui.label(RichText::new(icon).color(color));
                                ui.vertical(|ui| {
                                    let text = RichText::new(format!(
                                        "{} {}: {}",
                                        t.line_short, d.line, d.message
                                    ))
                                    .color(pal.text);
                                    let r = ui
                                        .add(egui::Label::new(text).wrap().sense(Sense::click()))
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                                    if r.clicked() {
                                        self.jump_to = Some(d.span.clone());
                                    }
                                    if let Some(h) = &d.hint {
                                        let b = ui
                                            .small_button(
                                                RichText::new(format!("{} → {h}", t.fix))
                                                    .color(pal.ok),
                                            )
                                            .on_hover_text(format!("{} „{h}”?", t.did_you_mean));
                                        if b.clicked() {
                                            fix = Some((d.span.clone(), h.clone()));
                                        }
                                    }
                                });
                            });
                            ui.add_space(2.0);
                        }
                    });
            });
        if let Some((span, hint)) = fix {
            self.apply_fix(span, &hint);
        }
    }

    /// Pune selecția `self.cursor` în starea editorului.
    fn store_cursor(&self, ctx: &egui::Context) {
        if let Some(sel) = &self.cursor {
            let id = egui::Id::unique(EDITOR_ID);
            let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
            state.cursor.set_char_range(Some(CCursorRange::two(
                CCursor::new(sel.start),
                CCursor::new(sel.end),
            )));
            state.store(ctx, id);
        }
    }

    /// Lista de variante sub comanda scrisă: se deschide când elevul scrie și
    /// se închide când cursorul pleacă sau editorul pierde focusul.
    fn completion_list(&mut self, ui: &Ui, out: &egui::text_edit::TextEditOutput, edited: bool) {
        if out.response.changed() && !edited {
            self.update_completion();
        }
        let Some(c) = &mut self.completion else {
            return;
        };
        let on_list = c
            .rect
            .zip(ui.input(|i| i.pointer.hover_pos()))
            .is_some_and(|(r, p)| r.contains(p));
        let focused = out.response.response.has_focus() || self.focus_editor;
        if self.cursor != Some(c.range.end..c.range.end) || (!focused && !on_list) {
            self.completion = None;
            return;
        }

        let at = out.galley.pos_from_cursor(CCursor::new(c.range.start));
        let pos = out.galley_pos + at.left_bottom().to_vec2() + vec2(0.0, 2.0);
        let mut clicked = None;
        let area = egui::Area::new(out.response.response.id.with("completion"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    // Fiecare variantă pe un singur rând, oricât de îngust e panoul.
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    for (i, s) in c.items.iter().enumerate() {
                        let text = RichText::new(s.label()).family(FontFamily::Monospace);
                        if ui.selectable_label(i == c.selected, text).clicked() {
                            clicked = Some(i);
                        }
                    }
                });
            });
        c.rect = Some(area.response.rect);
        if let Some(i) = clicked {
            self.accept_completion(i);
        }
    }

    fn code_editor(&mut self, ui: &mut Ui, pal: &Pal, editable: bool, label: egui::Id) {
        let mono = FontId::monospace(15.0);
        let gutter = 44.0;
        let mut layouter = |ui: &Ui, buf: &dyn TextBuffer, _wrap: f32| {
            let text = buf.as_str();
            let mut job = LayoutJob::default();
            for (range, class) in lang::highlight(text) {
                job.append(
                    &text[range],
                    0.0,
                    TextFormat {
                        font_id: mono.clone(),
                        color: pal.token(class),
                        italics: class == TokenClass::Comment,
                        ..Default::default()
                    },
                );
            }
            job.wrap.max_width = f32::INFINITY;
            ui.fonts_mut(|f| f.layout_job(job))
        };

        let id = egui::Id::unique(EDITOR_ID);
        let mut scroll_to = None;
        if let Some(span) = self.jump_to.take() {
            let a = char_index(&self.s.code, span.start);
            let b = char_index(&self.s.code, span.end);
            self.cursor = Some(a..b);
            self.focus_editor = true;
            scroll_to = Some(a);
        }

        let mut edited = false;
        // Tastele listei de variante: ↑/↓ aleg, Enter/Tab inserează, Esc închide.
        let focused = ui.memory(|m| m.has_focus(id));
        if !editable {
            self.completion = None;
        }
        // La Esc egui scoate deja focusul din editor, la începutul cadrului.
        let esc = ui.input(|i| i.key_pressed(Key::Escape));
        if (focused || esc)
            && let Some(c) = &mut self.completion
        {
            let n = c.items.len();
            let (up, down, accept, close) = ui.input_mut(|i| {
                (
                    i.consume_key(Modifiers::NONE, Key::ArrowUp),
                    i.consume_key(Modifiers::NONE, Key::ArrowDown),
                    i.consume_key(Modifiers::NONE, Key::Enter)
                        || i.consume_key(Modifiers::NONE, Key::Tab),
                    i.consume_key(Modifiers::NONE, Key::Escape),
                )
            });
            if up {
                c.selected = (c.selected + n - 1) % n;
            }
            if down {
                c.selected = (c.selected + 1) % n;
            }
            let selected = c.selected;
            if close {
                self.completion = None;
                self.focus_editor = true;
            } else if accept {
                self.accept_completion(selected);
                self.store_cursor(ui.ctx());
                edited = true;
            }
        }

        // Enter e tratat aici, ca rândul nou să primească indentarea blocului.
        if editable
            && ui.memory(|m| m.has_focus(id))
            && let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), id)
            && let Some(range) = state.cursor.char_range()
            && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter))
        {
            let r = range.as_sorted_char_range();
            let at = self.new_line(r.start.into()..r.end.into());
            state
                .cursor
                .set_char_range(Some(CCursorRange::one(CCursor::new(at))));
            state.store(ui.ctx(), id);
            edited = true;
        }

        let bg = ui.painter().add(Shape::Noop);
        let out = egui::TextEdit::multiline(&mut self.s.code)
            .id(id)
            .font(mono.clone())
            .code_editor()
            .frame(egui::Frame::NONE.inner_margin(Margin {
                left: gutter as i8,
                right: 8,
                top: 6,
                bottom: 6,
            }))
            .lock_focus(true)
            .interactive(editable)
            .desired_width(f32::INFINITY)
            .desired_rows(30)
            .layouter(&mut layouter)
            .show(ui);
        let _ = out.response.response.clone().labelled_by(label);

        if out.response.changed() || edited {
            self.s.dirty = true;
            if !matches!(self.status, Status::Ready) {
                self.status = Status::Ready;
                self.error_line = None;
            }
        }
        if let Some(r) = out.cursor_range {
            let r = r.as_sorted_char_range();
            self.cursor = Some(r.start.into()..r.end.into());
        }
        if std::mem::take(&mut self.focus_editor) {
            // Fără focus, egui strânge selecția la cursor: o pune la loc.
            self.store_cursor(ui.ctx());
            out.response.response.request_focus();
        }
        self.completion_list(ui, &out, edited);

        let diags = if self.analyzed.as_ref() == Some(&self.s.code) {
            self.diagnostics()
        } else {
            Vec::new()
        };
        let error_lines: std::collections::HashSet<usize> =
            diags.iter().filter(|d| d.error).map(|d| d.line).collect();
        let warning_lines: std::collections::HashSet<usize> =
            diags.iter().filter(|d| !d.error).map(|d| d.line).collect();

        // Numerele rândurilor și evidențierea rândului curent / cu eroare.
        let painter = ui.painter().clone();
        let full = out.response.rect;
        let mut line = 1;
        let mut first_row = true;
        let mut highlight = Vec::new();
        for row in &out.galley.rows {
            let r = row.rect().translate(out.galley_pos.to_vec2());
            if first_row {
                let marked = error_lines.contains(&line) || Some(line) == self.error_line;
                let color = if Some(line) == self.line || marked {
                    pal.text
                } else {
                    pal.muted
                };
                painter.text(
                    pos2(full.left() + gutter - 12.0, r.center().y),
                    Align2::RIGHT_CENTER,
                    line.to_string(),
                    FontId::monospace(12.5),
                    color,
                );
                if Some(line) == self.line {
                    painter.text(
                        pos2(full.left() + 4.0, r.center().y),
                        Align2::LEFT_CENTER,
                        "▶",
                        FontId::proportional(11.0),
                        pal.accent,
                    );
                } else if marked {
                    painter.circle_filled(pos2(full.left() + 9.0, r.center().y), 4.0, pal.err);
                } else if warning_lines.contains(&line) {
                    let c = pos2(full.left() + 9.0, r.center().y);
                    let tri = vec![c + vec2(0.0, -4.5), c + vec2(4.5, 3.5), c + vec2(-4.5, 3.5)];
                    painter.add(Shape::convex_polygon(tri, pal.warn, Stroke::NONE));
                }
            }
            let fill = if Some(line) == self.error_line || error_lines.contains(&line) {
                Some(pal.error_line)
            } else if Some(line) == self.line {
                Some(pal.current_line)
            } else {
                None
            };
            if let Some(fill) = fill {
                let rect = Rect::from_x_y_ranges(
                    full.left()..=full.right().max(ui.clip_rect().right()),
                    r.y_range(),
                );
                highlight.push(Shape::rect_filled(rect, 3.0, fill));
            }
            first_row = row.ends_with_newline;
            if row.ends_with_newline {
                line += 1;
            }
        }
        ui.painter().set(bg, Shape::Vec(highlight));

        // Sublinierea ondulată a cuvintelor greșite, cu explicație la hover.
        let gp = out.galley_pos.to_vec2();
        let hover = out.response.response.hover_pos();
        let mut tooltip = None;
        for d in &diags {
            let a = char_index(&self.s.code, d.span.start);
            let b = char_index(&self.s.code, d.span.end).max(a);
            let ra = out.galley.pos_from_cursor(CCursor::new(a)).translate(gp);
            let rb = out.galley.pos_from_cursor(CCursor::new(b)).translate(gp);
            let same_row = (ra.center().y - rb.center().y).abs() < 1.0;
            let x1 = if same_row && b > a {
                rb.left()
            } else {
                ra.left() + 10.0
            };
            let color = if d.error { pal.err } else { pal.warn };
            wavy_underline(&painter, ra.left(), x1, ra.bottom() - 1.0, color);
            let hit =
                Rect::from_x_y_ranges(ra.left() - 2.0..=x1 + 2.0, ra.top()..=ra.bottom() + 2.0);
            if hover.is_some_and(|p| hit.contains(p)) {
                tooltip = Some(d);
            }
        }
        if let Some(d) = tooltip {
            let t = self.t();
            let _ = out.response.response.clone().on_hover_ui_at_pointer(|ui| {
                ui.label(RichText::new(&d.message).color(if d.error { pal.err } else { pal.warn }));
                if let Some(h) = &d.hint {
                    ui.label(format!("{} „{h}”?", t.did_you_mean));
                }
            });
        }
        if let Some(a) = scroll_to {
            let r = out.galley.pos_from_cursor(CCursor::new(a)).translate(gp);
            ui.scroll_to_rect(r.expand(40.0), None);
        }

        // Derulează la rândul care se execută.
        if let (Some(cur), Mode::Running | Mode::Paused) = (self.line, self.mode) {
            let mut l = 1;
            for row in &out.galley.rows {
                if l == cur {
                    let r = row.rect().translate(out.galley_pos.to_vec2());
                    ui.scroll_to_rect(r.expand(30.0), None);
                    break;
                }
                if row.ends_with_newline {
                    l += 1;
                }
            }
        }
    }

    fn controls(&mut self, ui: &mut Ui) {
        let t = self.t();
        let pal = Pal::new(ui.visuals().dark_mode);
        let now = ui.input(|i| i.time);
        // Se rupe pe două rânduri când fereastra e îngustă.
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().button_padding = vec2(12.0, 7.0);
            let (icon, label) = if self.mode == Mode::Running {
                ("⏸", t.pause)
            } else {
                ("▶", t.run)
            };
            let b = egui::Button::new(
                RichText::new(format!("{icon}  {label}"))
                    .color(Color32::WHITE)
                    .strong(),
            )
            .fill(pal.accent);
            if ui.add(b).on_hover_text("F5").clicked() {
                if self.mode == Mode::Running {
                    self.pause();
                } else {
                    self.run();
                }
            }
            if ui
                .button(format!("⏭  {}", t.step_once))
                .on_hover_text("F6")
                .clicked()
            {
                self.step_once(now);
            }
            if ui
                .add_enabled(
                    self.mode != Mode::Idle,
                    egui::Button::new(format!("⏹  {}", t.stop)),
                )
                .on_hover_text("Esc")
                .clicked()
            {
                self.stop();
            }
            if ui.button(format!("↺  {}", t.reset)).clicked() {
                self.reset();
            }
            if ui
                .button(format!("✔  {}", t.check))
                .on_hover_text("F7")
                .clicked()
            {
                self.check();
            }

            ui.add_space(10.0);
            let label = ui.label(RichText::new(t.speed).color(pal.muted));
            ui.spacing_mut().slider_width = 90.0;
            let slider = ui.add_enabled(
                !self.s.instant,
                egui::Slider::new(&mut self.s.speed, 1.0..=10.0)
                    .step_by(1.0)
                    .show_value(false),
            );
            let _ = slider.labelled_by(label.id);
            ui.toggle_value(&mut self.s.instant, format!("⚡ {}", t.instant));
        });
    }

    /// Uneltele pentru editarea tabelei de pornire.
    fn table_tools(&mut self, ui: &mut Ui) {
        let t = self.t();
        let pal = Pal::new(ui.visuals().dark_mode);
        ui.add_enabled_ui(self.mode == Mode::Idle, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(t.table).strong())
                    .on_hover_text(t.edit_hint);
                ui.add_space(4.0);
                if ui.button(format!("↻ {}", t.rotate)).clicked() {
                    self.edit_table(|tb| {
                        tb.dir = tb.dir.clockwise();
                        true
                    });
                }
                if ui.button(format!("🗑 {}", t.clear)).clicked() {
                    self.edit_table(|tb| {
                        let had = tb.letters().next().is_some();
                        tb.clear_letters();
                        had
                    });
                }
                if self.has_result
                    && ui
                        .button(RichText::new(format!("📌 {}", t.keep_result)).color(pal.ok))
                        .on_hover_text(t.keep_result_hint)
                        .clicked()
                {
                    self.keep_result();
                }
            });
        });
    }

    fn field(&mut self, ui: &mut Ui) {
        let pal = Pal::new(ui.visuals().dark_mode);
        let editable = self.mode == Mode::Idle;
        let sense = if editable {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        };
        let (resp, painter) = ui.allocate_painter(ui.available_size(), sense);
        let area = resp.rect;
        let (width, height) = (self.world.table.width, self.world.table.height);
        let pad = 34.0;
        let cell = ((area.width() - 2.0 * pad) / width as f32)
            .min((area.height() - 2.0 * pad) / height as f32)
            .clamp(8.0, 84.0);
        let size = vec2(cell * width as f32, cell * height as f32);
        let origin = area.center() - size / 2.0;
        let field_rect = Rect::from_min_size(origin, size);
        self.field_geom = (origin, cell);
        let rect_of = |(x, y): (i32, i32)| {
            Rect::from_min_size(
                origin + vec2(x as f32 * cell, y as f32 * cell),
                Vec2::splat(cell),
            )
        };
        let cell_at = |p: Pos2| {
            field_rect.contains(p).then(|| {
                let v = (p - origin) / cell;
                ((v.x as i32).min(width - 1), (v.y as i32).min(height - 1))
            })
        };

        // Editarea tabelei cu mausul și tastatura.
        let hover_cell = resp.hover_pos().and_then(cell_at);
        if editable {
            self.table_input(ui, &resp, &cell_at);
        } else {
            self.grab = None;
            self.table_cursor = None;
        }

        let w = &self.world;
        let table = &w.table;
        painter.rect_filled(field_rect.expand(14.0), 14.0, pal.field_bg);
        painter.rect_filled(field_rect, 2.0, pal.cell_bg);

        if self.s.show_trail {
            for y in 0..height {
                for x in 0..width {
                    if w.visited((x, y)) {
                        painter.rect_filled(rect_of((x, y)), 0.0, pal.trail);
                    }
                }
            }
        }
        if self.s.show_grid {
            let stroke = Stroke::new(1.0, pal.grid);
            for i in 1..width {
                let x = origin.x + i as f32 * cell;
                painter.line_segment(
                    [pos2(x, field_rect.top()), pos2(x, field_rect.bottom())],
                    stroke,
                );
            }
            for j in 1..height {
                let y = origin.y + j as f32 * cell;
                painter.line_segment(
                    [pos2(field_rect.left(), y), pos2(field_rect.right(), y)],
                    stroke,
                );
            }
        }
        painter.rect_stroke(
            field_rect,
            2.0,
            Stroke::new(2.0, pal.border),
            StrokeKind::Middle,
        );

        // Numerele coloanelor și rândurilor, de la 1, ca în fișiere.
        let label_font = FontId::proportional(11.0);
        let every = if cell < 18.0 { 5 } else { 1 };
        for i in (0..width).filter(|i| i % every == 0 || every == 1) {
            painter.text(
                rect_of((i, 0)).center_top() - vec2(0.0, 12.0),
                Align2::CENTER_CENTER,
                (i + 1).to_string(),
                label_font.clone(),
                pal.muted,
            );
        }
        for j in (0..height).filter(|j| j % every == 0 || every == 1) {
            painter.text(
                rect_of((0, j)).left_center() - vec2(14.0, 0.0),
                Align2::CENTER_CENTER,
                (j + 1).to_string(),
                label_font.clone(),
                pal.muted,
            );
        }

        // (mișcarea, progresul netezit, progresul liniar)
        let progress = self.anim.as_ref().map(|a| {
            let now = ui.input(|i| i.time);
            let raw = ((now - a.start) / a.dur).clamp(0.0, 1.0) as f32;
            (a.mv, ease(raw), raw)
        });
        let lerp = |a: (i32, i32), b: (i32, i32), p: f32| {
            let (a, b) = (rect_of(a).center(), rect_of(b).center());
            a + (b - a) * p
        };
        let ant_pos = match progress {
            Some((mv, p, _)) => lerp(mv.from, mv.to, p),
            None => rect_of(w.pos()).center(),
        };

        // Drumul furnicii.
        if self.s.show_trail && w.path.len() > 1 {
            let mut points: Vec<Pos2> = w.path.iter().map(|&p| rect_of(p).center()).collect();
            if progress.is_some()
                && let Some(last) = points.last_mut()
            {
                *last = ant_pos;
            }
            painter.add(Shape::line(
                points,
                Stroke::new((cell * 0.06).clamp(1.5, 4.0), pal.path),
            ));
        }

        // Literele; cea împinsă acum alunecă spre noua celulă.
        let sliding =
            progress.and_then(|(mv, p, _)| mv.pushed.map(|(c, dest)| (c, mv.to, dest, p)));
        for (p, c) in table.letters() {
            if sliding.is_some_and(|s| s.2 == p) {
                continue;
            }
            // Litera trasă rămâne doar ca o urmă pe locul ei.
            let lifted = matches!(self.grab, Some(Grab::Letter(_, from)) if from == p);
            letter_tile(
                &painter,
                rect_of(p),
                c,
                &pal,
                if lifted { 0.3 } else { 1.0 },
            );
        }
        if let Some((c, from, to, p)) = sliding {
            let r = Rect::from_center_size(lerp(from, to, p), Vec2::splat(cell));
            letter_tile(&painter, r, c, &pal, 1.0);
        }

        // Unde s-a lovit furnica.
        if self.crashed {
            let (dx, dy) = w.dir().delta();
            let (x, y) = w.pos();
            let ahead = (x + dx, y + dy);
            let stroke = Stroke::new(3.0, pal.err);
            if table.inside(ahead) {
                painter.rect_stroke(rect_of(ahead).shrink(2.0), 6.0, stroke, StrokeKind::Inside);
            } else {
                let r = rect_of(w.pos());
                let edge = match w.dir() {
                    Dir::Up => [r.left_top(), r.right_top()],
                    Dir::Down => [r.left_bottom(), r.right_bottom()],
                    Dir::Left => [r.left_top(), r.left_bottom()],
                    Dir::Right => [r.right_top(), r.right_bottom()],
                };
                painter.line_segment(edge, Stroke::new(5.0, pal.err));
            }
        }

        // Celula în care se scrie de la tastatură.
        if let Some(cur) = self.table_cursor {
            let r = rect_of(cur.cell).shrink(1.5);
            painter.rect_filled(r, 5.0, pal.accent.gamma_multiply(0.15));
            painter.rect_stroke(r, 5.0, Stroke::new(2.5, pal.accent), StrokeKind::Inside);
        }

        // Unde ajunge ce e tras: verde dacă se poate, roșu dacă nu.
        let dragging = self.grab.zip(self.grab_pos);
        if let Some((g, pos)) = dragging {
            let target = cell_at(pos);
            let color = match g.outcome(table, target) {
                Drop::Move | Drop::Swap => Some(pal.ok),
                Drop::Refused | Drop::Delete => Some(pal.err),
                Drop::Nothing => None,
            };
            if let (Some(t), Some(color)) = (target, color) {
                let r = rect_of(t).shrink(1.5);
                painter.rect_filled(r, 5.0, color.gamma_multiply(0.15));
                painter.rect_stroke(r, 5.0, Stroke::new(2.5, color), StrokeKind::Inside);
            }
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if editable && let Some(c) = hover_cell {
            let r = rect_of(c);
            painter.rect_stroke(
                r.shrink(1.0),
                4.0,
                Stroke::new(1.5, pal.accent.gamma_multiply(0.6)),
                StrokeKind::Inside,
            );
            let icon = if Grab::at(table, c).is_some() {
                egui::CursorIcon::Grab
            } else {
                egui::CursorIcon::Text
            };
            ui.ctx().set_cursor_icon(icon);
        }

        // Furnica: se întoarce în prima parte a mișcării și pășește cât trece la celula următoare.
        let (angle, gait) = match progress {
            Some((mv, _, raw)) => (
                ant::turn(ant::angle(mv.dir_before), ant::angle(mv.dir), raw / 0.35),
                raw,
            ),
            None => (ant::angle(w.dir()), 0.0),
        };
        let mut colors = pal.ant(self.crashed);
        if self.grab == Some(Grab::Ant) {
            colors = colors.faded(0.3);
        }
        ant::paint(&painter, ant_pos, cell * 0.95, angle, gait, colors);

        // Ce e tras plutește sub maus, puțin mai mare, ca ridicat de pe tabelă.
        if let Some((g, pos)) = dragging {
            match g {
                Grab::Ant => {
                    let colors = pal.ant(false);
                    ant::paint(&painter, pos, cell * 1.1, angle, 0.25, colors);
                }
                Grab::Letter(c, _) => {
                    let r = Rect::from_center_size(pos, Vec2::splat(cell * 1.1));
                    let delete = g.outcome(table, cell_at(pos)) == Drop::Delete;
                    letter_tile(&painter, r, c, &pal, if delete { 0.55 } else { 1.0 });
                    if delete {
                        let r = r.shrink(cell * 0.09);
                        painter.rect_stroke(r, 8.0, Stroke::new(2.5, pal.err), StrokeKind::Inside);
                    }
                }
            }
        }

        if editable {
            painter.text(
                pos2(field_rect.center().x, field_rect.bottom() + 26.0),
                Align2::CENTER_CENTER,
                self.t().edit_hint,
                FontId::proportional(12.5),
                pal.muted,
            );
        }
    }

    /// Tragerea furnicii și a literelor, click-urile și scrierea literelor pe tabelă.
    fn table_input(
        &mut self,
        ui: &Ui,
        resp: &egui::Response,
        cell_at: &dyn Fn(Pos2) -> Option<(i32, i32)>,
    ) {
        let pointer = resp.interact_pointer_pos().or(resp.hover_pos());
        // Se editează tabela de pornire, deci rezultatul unei rulări dispare.
        if (resp.drag_started() || resp.clicked() || resp.secondary_clicked()) && self.has_result {
            self.reset();
        }
        if resp.drag_started() {
            let from = ui.input(|i| i.pointer.press_origin()).and_then(cell_at);
            self.grab = from.and_then(|c| Grab::at(&self.s.table, c));
            self.grab_pos = None;
            if self.grab.is_some() {
                self.table_cursor = None;
            }
        }
        if self.grab.is_some()
            && let Some(p) = pointer
        {
            self.grab_pos = Some(p);
        }
        // La eliberare mausul poate să nu mai aibă poziție: contează ultima cunoscută.
        if resp.drag_stopped()
            && let Some(g) = self.grab.take()
        {
            let target = self.grab_pos.take().and_then(cell_at);
            self.edit_table(|t| g.drop(t, target));
        }
        if resp.clicked() {
            match pointer.and_then(cell_at) {
                Some(c) if c == self.s.table.ant => {
                    self.table_cursor = None;
                    self.edit_table(|t| {
                        t.dir = t.dir.clockwise();
                        true
                    });
                }
                Some(c) => {
                    self.table_cursor = Some(TableCursor::at(c));
                    resp.request_focus();
                }
                None => self.table_cursor = None,
            }
        }
        if resp.secondary_clicked()
            && let Some(c) = pointer.and_then(cell_at)
        {
            self.edit_table(|t| t.set(c, None));
        }

        if !resp.has_focus() && !resp.clicked() {
            self.table_cursor = None;
        }
        let Some(mut cur) = self.table_cursor else {
            return;
        };
        // Săgețile mută celula selectată, nu focusul la alt buton.
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                resp.id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    ..Default::default()
                },
            )
        });
        let mut keys = Vec::new();
        for e in ui.input(|i| i.events.clone()) {
            match e {
                egui::Event::Text(text) => keys.extend(text.chars().map(TypeKey::Char)),
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if !modifiers.command => keys.push(match key {
                    Key::Backspace => TypeKey::Backspace,
                    Key::Delete => TypeKey::Delete,
                    Key::ArrowLeft => TypeKey::Left,
                    Key::ArrowRight => TypeKey::Right,
                    Key::ArrowUp => TypeKey::Up,
                    Key::ArrowDown => TypeKey::Down,
                    Key::Enter => TypeKey::Enter,
                    Key::Escape => {
                        self.table_cursor = None;
                        resp.surrender_focus();
                        return;
                    }
                    _ => continue,
                }),
                _ => {}
            }
        }
        if keys.is_empty() {
            return;
        }
        let mut changed = false;
        for k in keys {
            changed |= type_key(&mut self.s.table, &mut cur, k);
        }
        self.table_cursor = Some(cur);
        if changed {
            self.s.dirty = true;
            self.reset();
        }
    }

    fn help_window(&mut self, ctx: &egui::Context) {
        let t = self.t();
        let pal = Pal::new(ctx.global_style().visuals.dark_mode);
        let mut open = self.show_help;
        egui::Window::new(format!("❓ {}", t.help))
            .open(&mut open)
            .default_size([560.0, 600.0])
            .default_pos(ctx.content_rect().right_top() + vec2(-600.0, 70.0))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (syntax, desc) in t.help_text {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(*syntax)
                                .family(FontFamily::Monospace)
                                .color(pal.control)
                                .size(14.5),
                        );
                        ui.label(*desc);
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    egui::Grid::new("kw_table")
                        .striped(true)
                        .spacing([18.0, 6.0])
                        .show(ui, |ui| {
                            for l in Lang::ALL {
                                ui.label(RichText::new(l.name()).strong());
                            }
                            ui.end_row();
                            let rows: [fn(&crate::i18n::Keywords) -> &'static str; 18] = [
                                |k| k.begin,
                                |k| k.end,
                                |k| k.up,
                                |k| k.down,
                                |k| k.left,
                                |k| k.right,
                                |k| k.repeat,
                                |k| k.times,
                                |k| k.end_repeat,
                                |k| k.while_,
                                |k| k.do_,
                                |k| k.end_while,
                                |k| k.if_,
                                |k| k.then,
                                |k| k.else_,
                                |k| k.border,
                                |k| k.not_border,
                                |k| k.procedure,
                            ];
                            for f in rows {
                                for l in Lang::ALL {
                                    ui.label(
                                        RichText::new(f(&l.t().kw)).family(FontFamily::Monospace),
                                    );
                                }
                                ui.end_row();
                            }
                        });
                    ui.add_space(10.0);
                    ui.separator();
                    ui.label(RichText::new(t.shortcuts).color(pal.muted));
                    ui.add_space(6.0);
                    ui.label(RichText::new(t.about).color(pal.muted));
                    ui.label(
                        RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                            .color(pal.muted)
                            .small(),
                    );
                });
            });
        self.show_help = open;
    }

    fn unsaved_dialog(&mut self, ctx: &egui::Context) {
        let Some(p) = self.pending else { return };
        let t = self.t();
        let mut choice = None;
        egui::Modal::new(egui::Id::unique("unsaved")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading(t.unsaved_title);
            ui.add_space(6.0);
            ui.label(t.unsaved_text);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(format!("💾  {}", t.save)).clicked() {
                    choice = Some(0);
                }
                if ui.button(t.discard).clicked() {
                    choice = Some(1);
                }
                if ui.button(t.cancel).clicked() {
                    choice = Some(2);
                }
            });
        });
        match choice {
            Some(0) => {
                self.pending = None;
                if self.save(false) {
                    self.perform(p, ctx);
                }
            }
            Some(1) => {
                self.pending = None;
                self.s.dirty = false;
                self.perform(p, ctx);
            }
            Some(_) => self.pending = None,
            None => {}
        }
    }
}

impl eframe::App for FurnicaApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.shortcuts(ctx);
        self.refresh_analysis();
        self.drive(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.top_bar(ui);
        self.status_bar(ui);
        self.editor_panel(ui);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(Margin::same(14)),
            )
            .show(ui, |ui| {
                self.controls(ui);
                ui.add_space(4.0);
                self.table_tools(ui);
                ui.add_space(8.0);
                self.field(ui);
            });

        self.help_window(&ctx);
        self.unsaved_dialog(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, &self.s);
    }
}

/// O literă pe tabelă, desenată ca o piesă de joc.
fn letter_tile(painter: &egui::Painter, cell: Rect, c: char, pal: &Pal, alpha: f32) {
    let r = cell.shrink(cell.width() * 0.08);
    painter.rect(
        r,
        r.width() * 0.18,
        pal.tile.gamma_multiply(alpha),
        Stroke::new(
            (r.width() * 0.05).clamp(1.0, 2.5),
            pal.tile_border.gamma_multiply(alpha),
        ),
        StrokeKind::Inside,
    );
    painter.text(
        r.center(),
        Align2::CENTER_CENTER,
        c,
        FontId::new(r.height() * 0.62, bold()),
        pal.tile_text.gamma_multiply(alpha),
    );
}

/// Sigla: o insignă portocalie cu o furnică deschisă la culoare, puțin înclinată.
fn logo(ui: &mut Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, size * 0.28, Color32::from_rgb(0xE8, 0x74, 0x22));
    // Lumina de sus a insignei.
    let top = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.center().y));
    p.rect_filled(
        top,
        CornerRadius {
            nw: (size * 0.28) as u8,
            ne: (size * 0.28) as u8,
            sw: 0,
            se: 0,
        },
        Color32::from_rgba_unmultiplied(0xFF, 0xB0, 0x40, 0x60),
    );
    let colors = ant::Colors {
        body: Color32::from_rgb(0xFF, 0xF6, 0xEC),
        legs: Color32::from_rgb(0xFF, 0xF6, 0xEC),
        shine: Color32::TRANSPARENT,
        eyes: Color32::from_rgb(0xE8, 0x74, 0x22),
    };
    ant::paint(p, rect.center(), size * 0.78, 0.5, 0.0, colors);
}

/// Indexul caracterului (pentru cursorul editorului) la o poziție în octeți.
fn char_index(s: &str, byte: usize) -> usize {
    let mut b = byte.min(s.len());
    while !s.is_char_boundary(b) {
        b -= 1;
    }
    s[..b].chars().count()
}

/// Poziția cursorului (în caractere) după o formatare care schimbă doar
/// spațiile și rândurile: rămâne lângă același caracter vizibil.
fn respaced_cursor(old: &str, new: &str, at: usize) -> usize {
    let visible = old.chars().take(at).filter(|c| !c.is_whitespace()).count();
    // Lipit de textul din stânga rămâne după el; altfel trece la textul următor.
    let after_text = at > 0 && old.chars().nth(at - 1).is_some_and(|c| !c.is_whitespace());
    let mut seen = 0;
    for (i, c) in new.chars().enumerate() {
        if seen == visible && (after_text || !c.is_whitespace()) {
            return i;
        }
        if !c.is_whitespace() {
            seen += 1;
        }
    }
    new.chars().count()
}

/// Pune pe rândurile `lines` indentarea dată de formatare; restul rămâne.
fn reindent_lines(code: &str, lines: std::ops::RangeInclusive<usize>) -> String {
    let formatted = lang::format(code);
    let out: Vec<&str> = code
        .split('\n')
        .zip(formatted.split('\n'))
        .enumerate()
        .map(|(i, (old, f))| if lines.contains(&i) { f } else { old })
        .collect();
    out.join("\n")
}

/// Poziția cursorului (în caractere) după reindentare: rămâne pe același
/// caracter din rând; din spațiile de la început trece la primul cuvânt.
fn reindented_cursor(old: &str, new: &str, at: usize) -> usize {
    let indent = |l: &str| l.chars().take_while(|c| c.is_whitespace()).count();
    let (mut old_start, mut new_start) = (0, 0);
    for (o, n) in old.split('\n').zip(new.split('\n')) {
        let (old_len, new_len) = (o.chars().count(), n.chars().count());
        if at <= old_start + old_len {
            let col = (at - old_start).saturating_sub(indent(o)) + indent(n);
            return new_start + col.min(new_len);
        }
        old_start += old_len + 1;
        new_start += new_len + 1;
    }
    new.chars().count()
}

fn wavy_underline(painter: &egui::Painter, x0: f32, x1: f32, y: f32, color: Color32) {
    let mut points = Vec::new();
    let mut x = x0;
    while x <= x1 {
        let phase = ((x - x0) / 2.0) as i32 % 2;
        points.push(pos2(x, y + if phase == 0 { -1.2 } else { 1.2 }));
        x += 2.0;
    }
    if points.len() >= 2 {
        painter.add(Shape::line(points, Stroke::new(1.3, color)));
    }
}

fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

fn setup_style(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let data = |b: &'static [u8]| Arc::new(egui::FontData::from_static(b));
    fonts.font_data.insert(
        "noto".into(),
        data(include_bytes!("../assets/fonts/NotoSans-Regular.ttf")),
    );
    fonts.font_data.insert(
        "noto-bold".into(),
        data(include_bytes!("../assets/fonts/NotoSans-Bold.ttf")),
    );
    fonts.font_data.insert(
        "fira".into(),
        data(include_bytes!("../assets/fonts/FiraCode-Regular.ttf")),
    );
    let prop = fonts.families.entry(FontFamily::Proportional).or_default();
    prop.insert(0, "noto".into());
    // Fira Code are săgețile și simbolurile care lipsesc din Noto Sans.
    prop.push("fira".into());
    let mut bold_family = vec!["noto-bold".to_string()];
    bold_family.extend(prop.iter().cloned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "fira".into());
    fonts.families.insert(bold(), bold_family);
    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|style| {
        use egui::TextStyle::*;
        style.text_styles.insert(Body, FontId::proportional(14.5));
        style.text_styles.insert(Button, FontId::proportional(14.5));
        style.text_styles.insert(Heading, FontId::new(19.0, bold()));
        style.text_styles.insert(Small, FontId::proportional(12.0));
        style.text_styles.insert(Monospace, FontId::monospace(14.5));
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(10.0, 5.0);
        style.spacing.interact_size.y = 28.0;
        style.spacing.slider_width = 140.0;
        let r = CornerRadius::same(8);
        let v = &mut style.visuals;
        v.widgets.noninteractive.corner_radius = r;
        v.widgets.inactive.corner_radius = r;
        v.widgets.hovered.corner_radius = r;
        v.widgets.active.corner_radius = r;
        v.widgets.open.corner_radius = r;
        v.window_corner_radius = CornerRadius::same(12);
        v.menu_corner_radius = CornerRadius::same(10);
    });

    ctx.style_mut_of(Theme::Light, |s| {
        let v = &mut s.visuals;
        v.panel_fill = Color32::from_rgb(0xF7, 0xF5, 0xF1);
        v.extreme_bg_color = Color32::from_rgb(0xEE, 0xEB, 0xE4);
        v.window_fill = Color32::from_rgb(0xFF, 0xFE, 0xFB);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xEA, 0xE6, 0xDE);
        v.widgets.inactive.bg_fill = Color32::from_rgb(0xEA, 0xE6, 0xDE);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xE0, 0xDB, 0xD0);
        v.selection.bg_fill = Color32::from_rgb(0xF9, 0xC8, 0x9B);
        v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0xC2, 0x55, 0x0C));
        v.hyperlink_color = Color32::from_rgb(0xC2, 0x55, 0x0C);
    });
    ctx.style_mut_of(Theme::Dark, |s| {
        let v = &mut s.visuals;
        v.panel_fill = Color32::from_rgb(0x1C, 0x1F, 0x26);
        v.extreme_bg_color = Color32::from_rgb(0x14, 0x16, 0x1B);
        v.window_fill = Color32::from_rgb(0x22, 0x26, 0x2E);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x2C, 0x31, 0x3B);
        v.widgets.inactive.bg_fill = Color32::from_rgb(0x2C, 0x31, 0x3B);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x38, 0x3E, 0x4A);
        v.selection.bg_fill = Color32::from_rgb(0x8A, 0x4A, 0x1A);
        v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0xF0, 0x8A, 0x3C));
        v.hyperlink_color = Color32::from_rgb(0xF0, 0x8A, 0x3C);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::WorldError;

    #[test]
    fn examples_are_clean_and_finish() {
        for (name, src) in EXAMPLES {
            let f = antfile::parse(src);
            let a = lang::analyze(&f.code);
            assert!(
                a.errors.is_empty() && a.warnings.is_empty(),
                "{name}: {:?} {:?}",
                a.errors,
                a.warnings
            );
            let mut world = World::new(f.table);
            let mut vm = Vm::new(a.program.unwrap());
            let end = crate::vm::run_to_end(&mut vm, &mut world, 100_000);
            assert_eq!(end, Tick::Finished, "{name}");
        }
    }

    #[test]
    fn borders_example_returns_to_the_corner() {
        for (name, src) in &EXAMPLES[3..6] {
            let f = antfile::parse(src);
            let mut world = World::new(f.table);
            let mut vm = Vm::new(lang::analyze(&f.code).program.unwrap());
            crate::vm::run_to_end(&mut vm, &mut world, 1000);
            assert_eq!((world.pos(), world.step_count), ((0, 0), 36), "{name}");
        }
    }

    /// Teste de interfață fără fereastră: apasă butoanele reale și,
    /// dacă `FURNICA_SHOTS` e setat, salvează capturi PNG în acel director.
    mod ui {
        use super::*;
        use egui_kittest::Harness;
        use egui_kittest::kittest::Queryable;

        fn harness<'a>() -> Harness<'a, FurnicaApp> {
            Harness::builder()
                .with_size(vec2(1280.0, 800.0))
                .wgpu()
                .build_eframe(|cc| FurnicaApp::new(cc, None))
        }

        fn shot(h: &mut Harness<'_, FurnicaApp>, name: &str) {
            if let Ok(dir) = std::env::var("FURNICA_SHOTS") {
                h.render()
                    .unwrap()
                    .save(format!("{dir}/{name}.png"))
                    .unwrap();
            }
        }

        fn run_until_idle(h: &mut Harness<'_, FurnicaApp>) {
            for _ in 0..20_000 {
                h.step();
                if h.state().mode == Mode::Idle && h.state().anim.is_none() {
                    break;
                }
            }
            h.step();
        }

        fn load(h: &mut Harness<'_, FurnicaApp>, example: usize) {
            h.state_mut().set_file(antfile::parse(EXAMPLES[example].1));
            h.run_steps(2);
        }

        #[test]
        fn run_original_example_and_keep_result() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            h.run_steps(3);
            shot(&mut h, "1-start");

            load(&mut h, 1);
            h.get_by_label("▶  Execută").click();
            h.run_steps(40);
            assert_eq!(h.state().mode, Mode::Running);
            assert!(h.state().line.is_some());
            shot(&mut h, "2-running");

            h.state_mut().s.instant = true;
            run_until_idle(&mut h);
            assert!(matches!(h.state().status, Status::Finished));
            assert_eq!(h.state().world.table.row(0).trim(), "conferinta");
            shot(&mut h, "3-finished");

            // Tabela de pornire nu s-a schimbat, până la „Păstrează rezultatul”.
            assert_eq!(h.state().s.table.row(0).trim(), "");
            h.get_by_label("📌 Păstrează rezultatul").click();
            h.run_steps(2);
            assert_eq!(h.state().s.table.row(0).trim(), "conferinta");
            assert!(h.state().s.dirty && !h.state().has_result);
        }

        #[test]
        fn snippet_buttons_insert_in_the_right_place() {
            let mut h = harness();
            h.state_mut().s.code = "ÎNCEPUT\nSUS\nSFÎRȘIT\n".into();
            h.run_steps(2);
            h.get_by_label("STÎNGA").click();
            h.run_steps(2);
            h.get_by_label("REPETĂ N").click();
            h.run_steps(2);
            assert_eq!(
                h.state().s.code,
                "ÎNCEPUT\nSUS\nSTÎNGA\nREPETĂ 4 ORI\n    JOS\nSFÎRȘIT REPETĂ\nSFÎRȘIT\n"
            );
            // Cursorul rămâne în editor după click pe buton.
            let editor = egui::Id::unique(EDITOR_ID);
            assert!(h.ctx.memory(|m| m.has_focus(editor)));
            // Exemplul din interior e selectat și e înlocuit de următoarea
            // comandă; după DACĂ se completează comanda de după ATUNCI, apoi
            // cea de după ALTFEL.
            for b in ["SUS", "DACĂ", "STÎNGA", "DREAPTA"] {
                h.get_by_label(b).click();
                h.run_steps(2);
            }
            assert_eq!(
                h.state().s.code,
                "ÎNCEPUT\nSUS\nSTÎNGA\nREPETĂ 4 ORI\n    SUS\n    DACĂ E_MARGINE ATUNCI\n        STÎNGA\n    ALTFEL\n        DREAPTA\n    \nSFÎRȘIT REPETĂ\nSFÎRȘIT\n"
            );
            // După comandă, cursorul e pe un rând nou, cu indentarea blocului.
            let at = h.state().s.code.find("    \nSFÎRȘIT REPETĂ").unwrap() + 4;
            let at = h.state().s.code[..at].chars().count();
            assert_eq!(h.state().cursor, Some(at..at));
            // Procedurile ajung înainte de ÎNCEPUT.
            h.get_by_label("PROCEDURA").click();
            h.run_steps(2);
            assert!(
                h.state()
                    .s
                    .code
                    .starts_with("PROCEDURA nume\n    JOS\nSFÎRȘIT PROCEDURA\n\nÎNCEPUT\n"),
                "{}",
                h.state().s.code
            );
            assert!(h.state().s.dirty);
            assert!(lang::analyze(&h.state().s.code).errors.is_empty());
        }

        /// Pune cursorul la `at` (în caractere) și focusul în editor.
        fn focus_at(h: &mut Harness<'_, FurnicaApp>, at: usize) {
            h.state_mut().cursor = Some(at..at);
            h.state_mut().focus_editor = true;
            h.run_steps(2);
        }

        fn completion_labels(h: &Harness<'_, FurnicaApp>) -> Vec<String> {
            h.state()
                .completion
                .as_ref()
                .map(|c| c.items.iter().map(Snippet::label).collect())
                .unwrap_or_default()
        }

        #[test]
        fn enter_keeps_block_indentation() {
            let mut h = harness();
            let src = "ÎNCEPUT\nREPETĂ 2 ORI\nSFÎRȘIT\n";
            h.state_mut().s.code = src.into();
            focus_at(&mut h, "ÎNCEPUT\nREPETĂ 2 ORI".chars().count());
            let typed = |h: &mut Harness<'_, FurnicaApp>, text: &str| {
                h.event(egui::Event::Text(text.into()));
                h.run_steps(1);
                h.key_press(Key::Enter);
                h.run_steps(1);
            };
            typed(&mut h, "");
            typed(&mut h, "JOS");
            assert_eq!(
                h.state().s.code,
                "ÎNCEPUT\nREPETĂ 2 ORI\n    JOS\n    \nSFÎRȘIT\n"
            );
            // SFÎRȘIT REPETĂ se aliniază cu REPETĂ după Enter.
            typed(&mut h, "SFÎRȘIT REPETĂ");
            assert_eq!(
                h.state().s.code,
                "ÎNCEPUT\nREPETĂ 2 ORI\n    JOS\nSFÎRȘIT REPETĂ\n\nSFÎRȘIT\n"
            );
        }

        #[test]
        fn completion_while_typing() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            h.state_mut().s.code = "ÎNCEPUT\n\nSFÎRȘIT\n".into();
            focus_at(&mut h, "ÎNCEPUT\n".chars().count());

            h.event(egui::Event::Text("cit".into()));
            h.run_steps(2);
            assert_eq!(
                completion_labels(&h),
                ["CÎT NU_E_MARGINE EXECUTĂ …", "CÎT E_MARGINE EXECUTĂ …"]
            );
            shot(&mut h, "10-completion");
            h.key_press(Key::ArrowDown);
            h.run_steps(1);
            h.key_press(Key::Enter);
            h.run_steps(2);
            assert_eq!(
                h.state().s.code,
                "ÎNCEPUT\nCÎT E_MARGINE EXECUTĂ\n    JOS\nSFÎRȘIT CÎT\nSFÎRȘIT\n"
            );
            assert!(h.state().completion.is_none());

            // „s” înlocuiește JOS selectat; sfârșitul ciclului e propus
            // înaintea celorlalte.
            h.event(egui::Event::Text("s".into()));
            h.run_steps(2);
            assert_eq!(
                completion_labels(&h),
                [
                    "SUS",
                    "STÎNGA",
                    "SFÎRȘIT CÎT",
                    "SFÎRȘIT REPETĂ",
                    "SFÎRȘIT PROCEDURA",
                    "SFÎRȘIT"
                ]
            );
            h.key_press(Key::Tab);
            h.run_steps(2);
            assert_eq!(
                h.state().s.code,
                "ÎNCEPUT\nCÎT E_MARGINE EXECUTĂ\n    SUS\n    \nSFÎRȘIT CÎT\nSFÎRȘIT\n"
            );

            // Esc închide lista, iar cursorul rămâne în editor.
            h.event(egui::Event::Text("JO".into()));
            h.run_steps(2);
            assert_eq!(completion_labels(&h), ["JOS"]);
            h.key_press(Key::Escape);
            h.run_steps(1);
            assert!(h.state().completion.is_none());
            h.key_press(Key::Enter);
            h.run_steps(1);

            // După o condiție scrisă întreagă, Enter face rând nou.
            h.event(egui::Event::Text("DACĂ E_MARGINE ATUNCI".into()));
            h.run_steps(2);
            assert!(h.state().completion.is_none());
            h.key_press(Key::Enter);
            h.run_steps(1);
            assert_eq!(
                h.state().s.code,
                "ÎNCEPUT\nCÎT E_MARGINE EXECUTĂ\n    SUS\n    JO\n    DACĂ E_MARGINE ATUNCI\n        \nSFÎRȘIT CÎT\nSFÎRȘIT\n"
            );
        }

        #[test]
        fn completion_click_calls_procedure() {
            let mut h = harness();
            let src = "PROCEDURA pina_jos\n    JOS\nSFÎRȘIT PROCEDURA\n\nÎNCEPUT\n\nSFÎRȘIT\n";
            h.state_mut().s.code = src.into();
            focus_at(&mut h, src.chars().count() - "\nSFÎRȘIT\n".chars().count());
            h.event(egui::Event::Text("pin".into()));
            h.run_steps(2);
            h.get_by_label("pina_jos").click();
            h.run_steps(3);
            assert!(h.state().s.code.ends_with("ÎNCEPUT\npina_jos\n\nSFÎRȘIT\n"));
            let editor = egui::Id::unique(EDITOR_ID);
            assert!(h.ctx.memory(|m| m.has_focus(editor)));
        }

        #[test]
        fn format_button_and_undo() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            let src = "ÎNCEPUT\nREPETĂ 4 ORI\nJOS\nDACĂ E_MARGINE ATUNCI SUS ALTFEL JOS\nSFÎRȘIT REPETĂ\nSFÎRȘIT\n";
            let formatted = "ÎNCEPUT\nREPETĂ 4 ORI\n    JOS\n    DACĂ E_MARGINE ATUNCI\n        SUS\n    ALTFEL\n        JOS\nSFÎRȘIT REPETĂ\nSFÎRȘIT\n";
            h.state_mut().s.code = src.into();
            // Cursorul după SUS rămâne după SUS, acum pe rândul lui.
            let at = src[..src.find(" ALTFEL").unwrap()].chars().count();
            h.state_mut().cursor = Some(at..at);
            h.run_steps(2);
            h.get_by_label("⇥  Formatează").click();
            h.run_steps(2);
            assert_eq!(h.state().s.code, formatted);
            let at = formatted[..formatted.find("\n    ALTFEL").unwrap()]
                .chars()
                .count();
            assert_eq!(h.state().cursor, Some(at..at));
            let editor = egui::Id::unique(EDITOR_ID);
            assert!(h.ctx.memory(|m| m.has_focus(editor)));
            shot(&mut h, "9-formatted");

            // Formatarea se anulează cu un singur Ctrl+Z.
            h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
            h.run_steps(2);
            assert_eq!(h.state().s.code, src);

            h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::F);
            h.run_steps(2);
            assert_eq!(h.state().s.code, formatted);
        }

        #[test]
        fn live_analysis_and_quick_fix() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            h.state_mut().s.code = "PROCEDURA PinaJos\nCÎT NU_E_MARGINE EXECUTĂ\n    JSO\nSFÎRȘIT CÎT\nSFÎRȘIT PROCEDURA\n\nÎNCEPUT\nDRAPTA\nPinaJso\nSFÎRȘIT\n".into();
            h.run_steps(3);
            // Analiza rulează singură, fără „Verifică”.
            let errs: Vec<usize> = h.state().analysis.errors.iter().map(|e| e.line).collect();
            assert_eq!(errs, vec![3, 8, 9]);
            // „PinaJos” nu e folosită: apelul e scris greșit.
            assert_eq!(h.state().analysis.warnings.len(), 1);
            shot(&mut h, "7-problems");

            // Rularea e blocată și arată prima eroare.
            h.get_by_label("▶  Execută").click();
            h.run_steps(2);
            assert!(matches!(h.state().status, Status::Compile(_, 3)));
            assert_eq!(h.state().mode, Mode::Idle);

            for fix in [
                "Corectează → JOS",
                "Corectează → DREAPTA",
                "Corectează → PinaJos",
            ] {
                h.get_by_label(fix).click();
                h.run_steps(3);
                // Cuvântul corectat rămâne selectat în editor.
                let editor = egui::Id::unique(EDITOR_ID);
                assert!(h.ctx.memory(|m| m.has_focus(editor)));
                let range = egui::text_edit::TextEditState::load(&h.ctx, editor)
                    .and_then(|s| s.cursor.char_range())
                    .unwrap()
                    .as_sorted_char_range();
                let selected: String = h
                    .state()
                    .s
                    .code
                    .chars()
                    .skip(range.start.0)
                    .take(range.end.0 - range.start.0)
                    .collect();
                assert_eq!(Some(selected.as_str()), fix.split(' ').next_back());
            }
            let a = &h.state().analysis;
            assert!(a.errors.is_empty() && a.warnings.is_empty(), "{a:?}");
            shot(&mut h, "8-fixed");
        }

        #[test]
        fn errors_and_dark_theme() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Dark);
            h.state_mut().s.code = "ÎNCEPUT\nSUS\nZBOR\nSFÎRȘIT\n".into();
            h.run_steps(2);
            h.get_by_label("✔  Verifică").click();
            h.run_steps(3);
            assert!(matches!(h.state().status, Status::Checked(1, 0)));
            assert_eq!(h.state().analysis.errors[0].line, 3);
            shot(&mut h, "4-compile-error");

            // Litera ajunge la margine și nu mai poate fi împinsă.
            let mut table = Table::default();
            table.set((0, 5), Some('X'));
            h.state_mut().set_file(antfile::AntFile {
                table,
                code: "ÎNCEPUT\nCÎT NU_E_MARGINE EXECUTĂ\n    JOS\nSFÎRȘIT CÎT\nSFÎRȘIT\n".into(),
            });
            h.state_mut().s.instant = true;
            h.get_by_label("▶  Execută").click();
            run_until_idle(&mut h);
            assert!(matches!(
                h.state().status,
                Status::Runtime(RuntimeError::World(WorldError::PushOut), 3)
            ));
            assert!(h.state().crashed);
            assert_eq!(h.state().world.pos(), (0, 8));
            shot(&mut h, "5-runtime-error");
        }

        #[test]
        fn russian_step_by_step() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Dark);
            h.state_mut().s.lang = Lang::Ru;
            load(&mut h, 5);
            for _ in 0..12 {
                h.get_by_label("⏭  По шагам").click();
                h.run_steps(2);
            }
            assert_eq!(h.state().mode, Mode::Paused);
            assert_eq!(h.state().world.step_count, 12);
            h.get_by_label("❓  Помощь").click();
            h.run_steps(3);
            shot(&mut h, "6-russian-help");
        }

        /// Centrul celulei pe ecran, după geometria din ultimul cadru.
        fn at(h: &Harness<'_, FurnicaApp>, (x, y): (i32, i32)) -> Pos2 {
            let (origin, cell) = h.state().field_geom;
            origin + vec2((x as f32 + 0.5) * cell, (y as f32 + 0.5) * cell)
        }

        /// Trage cu mausul de la o celulă la un punct, prin câțiva pași intermediari.
        fn drag(h: &mut Harness<'_, FurnicaApp>, from: (i32, i32), to: Pos2) {
            let start = at(h, from);
            h.hover_at(start);
            h.run_steps(1);
            h.drag_at(start);
            h.run_steps(1);
            for i in 1..=4 {
                h.hover_at(start + (to - start) * (i as f32 / 4.0));
                h.run_steps(1);
            }
            h.drop_at(to);
            h.run_steps(2);
        }

        fn click(h: &mut Harness<'_, FurnicaApp>, p: Pos2, button: egui::PointerButton) {
            h.hover_at(p);
            h.run_steps(1);
            for pressed in [true, false] {
                h.event(egui::Event::PointerButton {
                    pos: p,
                    button,
                    pressed,
                    modifiers: Modifiers::NONE,
                });
                h.run_steps(1);
            }
        }

        fn empty_table(h: &mut Harness<'_, FurnicaApp>) {
            h.state_mut().set_file(antfile::AntFile {
                table: Table::default(),
                code: "ÎNCEPUT\nSFÎRȘIT\n".into(),
            });
            h.run_steps(2);
        }

        #[test]
        fn ant_is_dragged_and_turned() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Light);
            empty_table(&mut h);
            h.state_mut().s.table.set((2, 2), Some('A'));
            h.state_mut().reset();
            h.run_steps(1);

            // Pe o literă nu se poate: furnica rămâne pe loc.
            let target = at(&h, (2, 2));
            drag(&mut h, (0, 0), target);
            assert_eq!(h.state().s.table.ant, (0, 0));

            let target = at(&h, (4, 6));
            drag(&mut h, (0, 0), target);
            assert_eq!(h.state().s.table.ant, (4, 6));
            assert_eq!(h.state().world.pos(), (4, 6));
            assert!(h.state().s.dirty);

            // Un click pe furnică o rotește.
            let p = at(&h, (4, 6));
            click(&mut h, p, egui::PointerButton::Primary);
            assert_eq!(h.state().s.table.dir, Dir::Left);

            // Imaginea din timpul tragerii: furnica plutește peste o celulă liberă.
            let start = at(&h, (4, 6));
            let over = at(&h, (7, 3));
            h.hover_at(start);
            h.run_steps(1);
            h.drag_at(start);
            h.run_steps(1);
            h.hover_at(start + (over - start) * 0.5);
            h.run_steps(1);
            h.hover_at(over);
            h.run_steps(1);
            assert_eq!(h.state().grab, Some(Grab::Ant));
            shot(&mut h, "9-dragging-ant");
            h.drop_at(over);
            h.run_steps(2);
            assert_eq!(h.state().s.table.ant, (7, 3));
        }

        #[test]
        fn letters_are_typed_moved_and_deleted() {
            let mut h = harness();
            h.ctx.set_theme(ThemePreference::Dark);
            empty_table(&mut h);

            // Click pe o celulă, apoi se scrie ca într-un text.
            let p = at(&h, (2, 3));
            click(&mut h, p, egui::PointerButton::Primary);
            assert!(h.state().table_cursor.is_some());
            h.event(egui::Event::Text("ABCX".into()));
            h.run_steps(1);
            h.key_press(Key::Backspace);
            h.run_steps(1);
            h.key_press(Key::Enter);
            h.run_steps(1);
            h.event(egui::Event::Text("DE".into()));
            h.run_steps(2);
            assert_eq!(h.state().s.table.row(3), "  ABC     ");
            assert_eq!(h.state().s.table.row(4), "  DE      ");
            assert!(
                lang::analyze(&h.state().s.code).errors.is_empty(),
                "editorul nu primește literele"
            );
            shot(&mut h, "10-typing-letters");

            // Esc încheie scrierea.
            h.key_press(Key::Escape);
            h.run_steps(2);
            assert!(h.state().table_cursor.is_none());

            // Tragerea mută litera; peste altă literă face schimb cu ea.
            let target = at(&h, (6, 6));
            drag(&mut h, (2, 3), target);
            assert_eq!(h.state().s.table.get((6, 6)), Some('A'));
            assert_eq!(h.state().s.table.get((2, 3)), None);
            let target = at(&h, (3, 4));
            drag(&mut h, (3, 3), target);
            assert_eq!(h.state().s.table.row(3), "   EC     ");
            assert_eq!(h.state().s.table.row(4), "  DB      ");

            // Trasă în afara tabelei, litera se șterge; click dreapta la fel.
            let (origin, _) = h.state().field_geom;
            drag(&mut h, (6, 6), origin - vec2(20.0, -40.0));
            assert_eq!(h.state().s.table.get((6, 6)), None);
            let p = at(&h, (2, 4));
            click(&mut h, p, egui::PointerButton::Secondary);
            assert_eq!(h.state().s.table.row(4), "   B      ");

            let saved = antfile::write(&h.state().s.table, &h.state().s.code);
            assert_eq!(antfile::parse(&saved).table, h.state().s.table);

            h.get_by_label("🗑 Golește").click();
            h.run_steps(2);
            assert_eq!(h.state().s.table.letters().count(), 0);
        }
    }
}
