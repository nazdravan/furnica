//! Textele interfeței în română, rusă și engleză.

use crate::check::WarningKind;
use crate::lang::{BlockKind, ErrorKind};
use crate::vm::RuntimeError;
use crate::world::{Dir, WorldError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    Ro,
    Ru,
    En,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Ro, Lang::Ru, Lang::En];

    pub fn name(self) -> &'static str {
        match self {
            Lang::Ro => "Română",
            Lang::Ru => "Русский",
            Lang::En => "English",
        }
    }

    pub fn t(self) -> &'static Strings {
        match self {
            Lang::Ro => &RO,
            Lang::Ru => &RU,
            Lang::En => &EN,
        }
    }
}

/// Cuvintele-cheie folosite la inserarea șabloanelor și în ajutor.
pub struct Keywords {
    pub begin: &'static str,
    pub end: &'static str,
    pub up: &'static str,
    pub down: &'static str,
    pub left: &'static str,
    pub right: &'static str,
    pub repeat: &'static str,
    pub times: &'static str,
    pub end_repeat: &'static str,
    pub while_: &'static str,
    pub do_: &'static str,
    pub end_while: &'static str,
    pub if_: &'static str,
    pub then: &'static str,
    pub else_: &'static str,
    pub border: &'static str,
    pub not_border: &'static str,
    pub procedure: &'static str,
    pub end_procedure: &'static str,
    pub name: &'static str,
}

pub struct Strings {
    pub kw: Keywords,
    pub title: &'static str,
    pub new: &'static str,
    pub open: &'static str,
    pub save: &'static str,
    pub save_as: &'static str,
    pub examples: &'static str,
    pub help: &'static str,
    pub settings: &'static str,
    pub theme: &'static str,
    pub theme_system: &'static str,
    pub theme_light: &'static str,
    pub theme_dark: &'static str,
    pub run: &'static str,
    pub step_once: &'static str,
    pub pause: &'static str,
    pub stop: &'static str,
    pub reset: &'static str,
    pub check: &'static str,
    pub format: &'static str,
    pub speed: &'static str,
    pub instant: &'static str,
    pub table: &'static str,
    pub columns: &'static str,
    pub rows: &'static str,
    pub show_grid: &'static str,
    pub show_trail: &'static str,
    pub program: &'static str,
    pub untitled: &'static str,
    pub ready: &'static str,
    pub running: &'static str,
    pub paused: &'static str,
    pub finished: &'static str,
    pub stopped: &'static str,
    pub no_errors: &'static str,
    pub error_line: &'static str,
    pub position: &'static str,
    pub direction: &'static str,
    pub moves: &'static str,
    pub pushes: &'static str,
    /// În ordinea `Dir::ALL`: jos, stînga, sus, dreapta.
    pub dirs: [&'static str; 4],
    pub rotate: &'static str,
    pub clear: &'static str,
    pub keep_result: &'static str,
    pub keep_result_hint: &'static str,
    pub edit_hint: &'static str,
    pub unsaved_title: &'static str,
    pub unsaved_text: &'static str,
    pub discard: &'static str,
    pub cancel: &'static str,
    pub file_error: &'static str,
    pub ant_files: &'static str,
    pub problems: &'static str,
    pub no_problems: &'static str,
    pub live_check: &'static str,
    pub did_you_mean: &'static str,
    pub fix: &'static str,
    pub line_short: &'static str,
    pub summary: fn(usize, usize) -> String,
    pub help_text: &'static [(&'static str, &'static str)],
    pub about: &'static str,
    /// „Autor” în fereastra de ajutor, urmat de nume și de linkuri.
    pub author: &'static str,
    pub shortcuts: &'static str,
}

impl Strings {
    pub fn dir(&self, d: Dir) -> &'static str {
        self.dirs[Dir::ALL.iter().position(|x| *x == d).unwrap_or(0)]
    }

    fn block_end(&self, b: BlockKind) -> &'static str {
        match b {
            BlockKind::Procedure => self.kw.end_procedure,
            BlockKind::Repeat => self.kw.end_repeat,
            BlockKind::Cycle => self.kw.end_while,
        }
    }

    fn block_start(&self, b: BlockKind) -> &'static str {
        match b {
            BlockKind::Procedure => self.kw.procedure,
            BlockKind::Repeat => self.kw.repeat,
            BlockKind::Cycle => self.kw.while_,
        }
    }

    pub fn compile_error(&self, lang: Lang, e: &ErrorKind) -> String {
        use ErrorKind::*;
        let k = &self.kw;
        match lang {
            Lang::Ro => match e {
                UnexpectedToken(t) => format!("„{t}” nu are ce căuta aici"),
                UnknownWord(t) => {
                    format!("Cuvânt necunoscut „{t}” (nu este nici comandă, nici procedură)")
                }
                UnknownCondition(t) => format!(
                    "Condiție necunoscută „{t}” (se poate {} sau {})",
                    k.border, k.not_border
                ),
                ExpectedCommand => "Aici lipsește o comandă".into(),
                ExpectedNumber => format!("După {} trebuie un număr", k.repeat),
                ExpectedName => "Lipsește numele procedurii".into(),
                ExpectedCondition => {
                    format!("Lipsește condiția ({} sau {})", k.border, k.not_border)
                }
                MissingEnd(b) => format!(
                    "„{}” nu este închis cu „{}”",
                    self.block_start(*b),
                    self.block_end(*b)
                ),
                WrongEnd { expected } => format!("Aici trebuia „{}”", self.block_end(*expected)),
                StrayEnd => format!("„{}” nu închide niciun bloc", k.end),
                StrayElse => format!(
                    "„{}” fără „{}” (după {} poate fi o singură comandă)",
                    k.else_, k.if_, k.then
                ),
                DuplicateProcedure(n) => format!("Procedura „{n}” este definită de două ori"),
                ProcedureInMain => format!(
                    "Procedurile se scriu înainte de „{}”, nu în programul principal",
                    k.begin
                ),
                CommandOutsideMain => {
                    format!(
                        "Comenzile trebuie scrise între „{}” și „{}”",
                        k.begin, k.end
                    )
                }
                MissingBegin => format!("„{}” fără „{}”", k.end, k.begin),
                DuplicateBegin => format!("„{}” poate fi doar o dată", k.begin),
                MissingMainEnd => format!("„{}” nu este închis cu „{}”", k.begin, k.end),
                TextAfterProgram => format!("Text în plus după „{}”", k.end),
                NumberTooLarge => "Numărul este prea mare".into(),
            },
            Lang::Ru => match e {
                UnexpectedToken(t) => format!("«{t}» здесь не к месту"),
                UnknownWord(t) => {
                    format!("Неизвестное слово «{t}» (это не команда и не процедура)")
                }
                UnknownCondition(t) => format!(
                    "Неизвестное условие «{t}» (можно {} или {})",
                    k.border, k.not_border
                ),
                ExpectedCommand => "Здесь не хватает команды".into(),
                ExpectedNumber => format!("После {} нужно число", k.repeat),
                ExpectedName => "Не указано имя процедуры".into(),
                ExpectedCondition => format!("Нет условия ({} или {})", k.border, k.not_border),
                MissingEnd(b) => format!(
                    "«{}» не закрыт словами «{}»",
                    self.block_start(*b),
                    self.block_end(*b)
                ),
                WrongEnd { expected } => {
                    format!("Здесь должно быть «{}»", self.block_end(*expected))
                }
                StrayEnd => format!("«{}» не закрывает ни одного блока", k.end),
                StrayElse => format!(
                    "«{}» без «{}» (после {} может быть только одна команда)",
                    k.else_, k.if_, k.then
                ),
                DuplicateProcedure(n) => format!("Процедура «{n}» описана дважды"),
                ProcedureInMain => format!(
                    "Процедуры пишутся до «{}», а не в основной программе",
                    k.begin
                ),
                CommandOutsideMain => {
                    format!("Команды нужно писать между «{}» и «{}»", k.begin, k.end)
                }
                MissingBegin => format!("«{}» без «{}»", k.end, k.begin),
                DuplicateBegin => format!("«{}» может быть только один раз", k.begin),
                MissingMainEnd => format!("«{}» не закрыто словом «{}»", k.begin, k.end),
                TextAfterProgram => format!("Лишний текст после «{}»", k.end),
                NumberTooLarge => "Слишком большое число".into(),
            },
            Lang::En => match e {
                UnexpectedToken(t) => format!("\"{t}\" does not belong here"),
                UnknownWord(t) => {
                    format!("Unknown word \"{t}\" (neither a command nor a procedure)")
                }
                UnknownCondition(t) => format!(
                    "Unknown condition \"{t}\" (use {} or {})",
                    k.border, k.not_border
                ),
                ExpectedCommand => "A command is missing here".into(),
                ExpectedNumber => format!("{} must be followed by a number", k.repeat),
                ExpectedName => "Missing procedure name".into(),
                ExpectedCondition => {
                    format!("Missing condition ({} or {})", k.border, k.not_border)
                }
                MissingEnd(b) => format!(
                    "\"{}\" is not closed with \"{}\"",
                    self.block_start(*b),
                    self.block_end(*b)
                ),
                WrongEnd { expected } => format!("Expected \"{}\" here", self.block_end(*expected)),
                StrayEnd => format!("\"{}\" does not close any block", k.end),
                StrayElse => format!(
                    "\"{}\" without \"{}\" ({} takes a single command)",
                    k.else_, k.if_, k.then
                ),
                DuplicateProcedure(n) => format!("Procedure \"{n}\" is defined twice"),
                ProcedureInMain => format!(
                    "Procedures are written before \"{}\", not in the main program",
                    k.begin
                ),
                CommandOutsideMain => {
                    format!(
                        "Commands must be written between \"{}\" and \"{}\"",
                        k.begin, k.end
                    )
                }
                MissingBegin => format!("\"{}\" without \"{}\"", k.end, k.begin),
                DuplicateBegin => format!("\"{}\" may appear only once", k.begin),
                MissingMainEnd => format!("\"{}\" is not closed with \"{}\"", k.begin, k.end),
                TextAfterProgram => format!("Extra text after \"{}\"", k.end),
                NumberTooLarge => "Number is too large".into(),
            },
        }
    }

    pub fn warning(&self, lang: Lang, w: &WarningKind) -> String {
        use WarningKind::*;
        let k = &self.kw;
        match lang {
            Lang::Ro => match w {
                UnusedProcedure(n) => format!("Procedura „{n}” nu este folosită niciodată"),
                EmptyBlock => "Blocul nu conține nicio comandă".into(),
                RepeatZero => format!(
                    "{} 0 {}: comenzile nu se execută niciodată",
                    k.repeat, k.times
                ),
                LoopWithoutMovement => format!(
                    "Ciclul {} nu mută furnica și poate rula la nesfârșit",
                    k.while_
                ),
                EndlessRecursion(n) => {
                    format!("Procedura „{n}” se apelează pe ea însăși fără oprire")
                }
                EmptyProgram => "Programul principal nu conține nicio comandă".into(),
            },
            Lang::Ru => match w {
                UnusedProcedure(n) => format!("Процедура «{n}» нигде не используется"),
                EmptyBlock => "В блоке нет ни одной команды".into(),
                RepeatZero => format!("{} 0 {}: команды никогда не выполнятся", k.repeat, k.times),
                LoopWithoutMovement => format!(
                    "Цикл {} не двигает муравья и может не закончиться",
                    k.while_
                ),
                EndlessRecursion(n) => format!("Процедура «{n}» вызывает себя без остановки"),
                EmptyProgram => "В основной программе нет команд".into(),
            },
            Lang::En => match w {
                UnusedProcedure(n) => format!("Procedure \"{n}\" is never used"),
                EmptyBlock => "The block contains no commands".into(),
                RepeatZero => format!("{} 0 {}: the commands never run", k.repeat, k.times),
                LoopWithoutMovement => format!(
                    "The {} loop does not move the ant and may never end",
                    k.while_
                ),
                EndlessRecursion(n) => format!("Procedure \"{n}\" calls itself without stopping"),
                EmptyProgram => "The main program has no commands".into(),
            },
        }
    }

    pub fn runtime_error(&self, lang: Lang, e: RuntimeError) -> &'static str {
        use RuntimeError::*;
        use WorldError::*;
        match (lang, e) {
            (Lang::Ro, World(OutOfField)) => "Furnica nu poate ieși din tabelă!",
            (Lang::Ro, World(PushOut)) => "Furnica nu poate împinge litera peste marginea tabelei!",
            (Lang::Ro, World(TooHeavy)) => "Furnica nu poate împinge două litere deodată!",
            (Lang::Ro, StackOverflow) => "Stiva este plină (procedura se apelează la nesfârșit)",
            (Lang::Ru, World(OutOfField)) => "Муравей не может выйти за край таблицы!",
            (Lang::Ru, World(PushOut)) => "Муравей не может вытолкнуть букву за край таблицы!",
            (Lang::Ru, World(TooHeavy)) => "Муравей не может толкать две буквы сразу!",
            (Lang::Ru, StackOverflow) => "Переполнение стека (процедура вызывает себя бесконечно)",
            (Lang::En, World(OutOfField)) => "The ant cannot leave the table!",
            (Lang::En, World(PushOut)) => "The ant cannot push the letter off the table!",
            (Lang::En, World(TooHeavy)) => "The ant cannot push two letters at once!",
            (Lang::En, StackOverflow) => "Stack overflow (a procedure calls itself forever)",
        }
    }
}

pub static RO: Strings = Strings {
    kw: Keywords {
        begin: "ÎNCEPUT",
        end: "SFÎRȘIT",
        up: "SUS",
        down: "JOS",
        left: "STÎNGA",
        right: "DREAPTA",
        repeat: "REPETĂ",
        times: "ORI",
        end_repeat: "SFÎRȘIT REPETĂ",
        while_: "CÎT",
        do_: "EXECUTĂ",
        end_while: "SFÎRȘIT CÎT",
        if_: "DACĂ",
        then: "ATUNCI",
        else_: "ALTFEL",
        border: "E_MARGINE",
        not_border: "NU_E_MARGINE",
        procedure: "PROCEDURA",
        end_procedure: "SFÎRȘIT PROCEDURA",
        name: "nume",
    },
    title: "Furnica",
    new: "Nou",
    open: "Deschide…",
    save: "Salvează",
    save_as: "Salvează ca…",
    examples: "Exemple",
    help: "Ajutor",
    settings: "Setări",
    theme: "Tema",
    theme_system: "Ca sistemul",
    theme_light: "Luminoasă",
    theme_dark: "Întunecată",
    run: "Execută",
    step_once: "Pas cu pas",
    pause: "Pauză",
    stop: "Stop",
    reset: "Resetare",
    check: "Verifică",
    format: "Formatează",
    speed: "Viteza",
    instant: "Instant",
    table: "Tabela",
    columns: "Coloane",
    rows: "Rânduri",
    show_grid: "Arată grila",
    show_trail: "Arată drumul furnicii",
    program: "Program",
    untitled: "fără nume",
    ready: "Gata de lucru",
    running: "Se execută…",
    paused: "Pauză",
    finished: "Executarea completă",
    stopped: "Executarea a fost oprită",
    no_errors: "Programul nu conține erori",
    error_line: "Eroare în rândul",
    position: "Poziția",
    direction: "Direcția",
    moves: "Mișcări",
    pushes: "Litere împinse",
    dirs: ["↓ jos", "← stînga", "↑ sus", "→ dreapta"],
    rotate: "Rotire",
    clear: "Golește",
    keep_result: "Păstrează rezultatul",
    keep_result_hint: "Tabela de după executare devine tabela de pornire",
    edit_hint: "Trage furnica sau literele cu mausul · click pe furnică o rotește · click pe o celulă și scrie literele · click dreapta șterge",
    unsaved_title: "Modificări nesalvate",
    unsaved_text: "Programul are modificări nesalvate. Le salvați?",
    discard: "Nu salva",
    cancel: "Anulare",
    file_error: "Nu pot deschide/salva fișierul",
    ant_files: "Programe Furnica",
    problems: "Probleme",
    no_problems: "Nicio problemă găsită",
    live_check: "Verifică programul în timp ce scriu",
    did_you_mean: "Poate ați vrut",
    fix: "Corectează",
    line_short: "rândul",
    summary: |e, w| match (e, w) {
        (0, 0) => "Programul nu conține erori".into(),
        (0, w) => format!("Fără erori · avertismente: {w}"),
        (e, 0) => format!("Erori: {e}"),
        (e, w) => format!("Erori: {e} · avertismente: {w}"),
    },
    help_text: &[
        (
            "Tabela",
            "Furnica se mișcă pe celulele unei tabele în care sunt litere. Când merge spre o literă, o împinge cu o celulă. Nu poate împinge două litere deodată și nici o literă peste margine. Celulele prin care a trecut rămân colorate.",
        ),
        (
            "Editarea tabelei",
            "Furnica și literele se mută trăgându-le cu mausul; o literă lăsată peste alta face schimb cu ea, iar una trasă în afara tabelei se șterge. Un click pe furnică o rotește. Pentru litere noi dați click pe o celulă și scrieți de la tastatură, ca într-un text: cursorul trece singur la celula următoare; Backspace, Delete, săgețile și Enter funcționează ca de obicei, Esc încheie. Click dreapta șterge o literă. Tabela se salvează împreună cu programul.",
        ),
        (
            "SUS  JOS  STÎNGA  DREAPTA",
            "Mută furnica cu o celulă în direcția respectivă.",
        ),
        (
            "ÎNCEPUT … SFÎRȘIT",
            "Programul principal. Procedurile se scriu înaintea lui.",
        ),
        (
            "REPETĂ N ORI … SFÎRȘIT REPETĂ",
            "Repetă comenzile dintre ele de N ori.",
        ),
        (
            "CÎT condiție EXECUTĂ … SFÎRȘIT CÎT",
            "Repetă comenzile cât timp condiția este adevărată.",
        ),
        (
            "DACĂ condiție ATUNCI comandă1 ALTFEL comandă2",
            "Execută comanda1 dacă condiția e adevărată, altfel comanda2 (partea ALTFEL poate lipsi).",
        ),
        (
            "E_MARGINE   NU_E_MARGINE",
            "E_MARGINE este adevărat când furnica privește spre marginea tabelei. Furnica privește în direcția în care s-a mișcat ultima dată.",
        ),
        (
            "PROCEDURA nume … SFÎRȘIT PROCEDURA",
            "Definește o procedură; se apelează scriind doar numele ei. Numele poate conține litere, cifre și _.",
        ),
        (
            "// comentariu",
            "Textul de după // până la sfârșitul rândului este ignorat.",
        ),
        (
            "Verificare",
            "Programul este verificat în timp ce scrieți: greșelile sunt subliniate cu roșu, avertismentele cu galben. Lista „Probleme” arată toate greșelile; un click duce la rândul respectiv, iar „Corectează” înlocuiește cuvântul greșit.",
        ),
        (
            "Limbă",
            "Cuvintele-cheie pot fi scrise în română (cu sau fără diacritice), engleză sau rusă, cu litere mari sau mici.",
        ),
    ],
    about: "Furnica — executant pentru învățarea algoritmilor. Rescris în Rust după programul original „Furnica” (Ant v2.00, 1999).",
    author: "Autor",
    shortcuts: "F5 — execută · F6 — pas cu pas · Esc — stop · F7 — verifică · Ctrl+Shift+F — formatează · Ctrl+S/O/N — fișiere",
};

pub static RU: Strings = Strings {
    kw: Keywords {
        begin: "НАЧАЛО",
        end: "КОНЕЦ",
        up: "ВВЕРХ",
        down: "ВНИЗ",
        left: "ВЛЕВО",
        right: "ВПРАВО",
        repeat: "ПОВТОРИ",
        times: "РАЗ",
        end_repeat: "КОНЕЦ ПОВТОРИ",
        while_: "ПОКА",
        do_: "СДЕЛАЙ",
        end_while: "КОНЕЦ ПОКА",
        if_: "ЕСЛИ",
        then: "ТОГДА",
        else_: "ИНАЧЕ",
        border: "КРАЙ",
        not_border: "НЕ_КРАЙ",
        procedure: "ПРОЦЕДУРА",
        end_procedure: "КОНЕЦ ПРОЦЕДУРА",
        name: "имя",
    },
    title: "Муравей",
    new: "Новый",
    open: "Открыть…",
    save: "Сохранить",
    save_as: "Сохранить как…",
    examples: "Примеры",
    help: "Помощь",
    settings: "Настройки",
    theme: "Тема",
    theme_system: "Как в системе",
    theme_light: "Светлая",
    theme_dark: "Тёмная",
    run: "Выполнить",
    step_once: "По шагам",
    pause: "Пауза",
    stop: "Стоп",
    reset: "Сброс",
    check: "Проверить",
    format: "Форматировать",
    speed: "Скорость",
    instant: "Мгновенно",
    table: "Таблица",
    columns: "Столбцы",
    rows: "Строки",
    show_grid: "Показывать сетку",
    show_trail: "Показывать путь муравья",
    program: "Программа",
    untitled: "без имени",
    ready: "Готов к работе",
    running: "Выполняется…",
    paused: "Пауза",
    finished: "Выполнение завершено",
    stopped: "Выполнение остановлено",
    no_errors: "Программа не содержит ошибок",
    error_line: "Ошибка в строке",
    position: "Позиция",
    direction: "Направление",
    moves: "Ходов",
    pushes: "Сдвинуто букв",
    dirs: ["↓ вниз", "← влево", "↑ вверх", "→ вправо"],
    rotate: "Поворот",
    clear: "Очистить",
    keep_result: "Оставить результат",
    keep_result_hint: "Таблица после выполнения станет начальной",
    edit_hint: "Перетаскивайте муравья и буквы мышью · щелчок по муравью поворачивает его · щёлкните клетку и печатайте буквы · правая кнопка стирает",
    unsaved_title: "Несохранённые изменения",
    unsaved_text: "В программе есть несохранённые изменения. Сохранить?",
    discard: "Не сохранять",
    cancel: "Отмена",
    file_error: "Не удалось открыть/сохранить файл",
    ant_files: "Программы Муравья",
    problems: "Проблемы",
    no_problems: "Проблем не найдено",
    live_check: "Проверять программу во время набора",
    did_you_mean: "Возможно, вы имели в виду",
    fix: "Исправить",
    line_short: "строка",
    summary: |e, w| match (e, w) {
        (0, 0) => "Программа не содержит ошибок".into(),
        (0, w) => format!("Ошибок нет · предупреждений: {w}"),
        (e, 0) => format!("Ошибок: {e}"),
        (e, w) => format!("Ошибок: {e} · предупреждений: {w}"),
    },
    help_text: &[
        (
            "Таблица",
            "Муравей ходит по клеткам таблицы, в которой стоят буквы. Если впереди буква, он сдвигает её на одну клетку. Две буквы сразу или букву за край таблицы он сдвинуть не может. Клетки, по которым он прошёл, остаются закрашенными.",
        ),
        (
            "Редактирование таблицы",
            "Муравья и буквы перетаскивают мышью; буква, брошенная на другую, меняется с ней местами, а вынесенная за таблицу стирается. Щелчок по муравью поворачивает его. Чтобы поставить новые буквы, щёлкните клетку и печатайте, как текст: курсор сам переходит к следующей клетке; Backspace, Delete, стрелки и Enter работают как обычно, Esc — конец ввода. Правая кнопка стирает букву. Таблица сохраняется вместе с программой.",
        ),
        (
            "ВВЕРХ  ВНИЗ  ВЛЕВО  ВПРАВО",
            "Перемещает муравья на одну клетку.",
        ),
        (
            "НАЧАЛО … КОНЕЦ",
            "Основная программа. Процедуры пишутся перед ней.",
        ),
        ("ПОВТОРИ N РАЗ … КОНЕЦ ПОВТОРИ", "Повторяет команды N раз."),
        (
            "ПОКА условие СДЕЛАЙ … КОНЕЦ ПОКА",
            "Повторяет команды, пока условие истинно.",
        ),
        (
            "ЕСЛИ условие ТОГДА команда1 ИНАЧЕ команда2",
            "Выполняет команду1, если условие истинно, иначе команду2 (ИНАЧЕ можно не писать).",
        ),
        (
            "КРАЙ   НЕ_КРАЙ",
            "КРАЙ истинно, когда муравей смотрит на край таблицы. Муравей смотрит туда, куда двигался в последний раз.",
        ),
        (
            "ПРОЦЕДУРА имя … КОНЕЦ ПРОЦЕДУРА",
            "Описание процедуры; вызывается по имени. Имя может содержать буквы, цифры и _.",
        ),
        (
            "// комментарий",
            "Текст после // до конца строки пропускается.",
        ),
        (
            "Проверка",
            "Программа проверяется во время набора: ошибки подчёркнуты красным, предупреждения — жёлтым. В списке «Проблемы» видны все ошибки; щелчок переходит к строке, а «Исправить» заменяет неверное слово.",
        ),
        (
            "Язык",
            "Ключевые слова можно писать по-русски, по-румынски или по-английски, большими или маленькими буквами.",
        ),
    ],
    about: "Муравей — исполнитель для изучения алгоритмов. Переписан на Rust по мотивам программы «Furnica» (Ant v2.00, 1999).",
    author: "Автор",
    shortcuts: "F5 — выполнить · F6 — по шагам · Esc — стоп · F7 — проверить · Ctrl+Shift+F — форматировать · Ctrl+S/O/N — файлы",
};

pub static EN: Strings = Strings {
    kw: Keywords {
        begin: "BEGIN",
        end: "END",
        up: "UP",
        down: "DOWN",
        left: "LEFT",
        right: "RIGHT",
        repeat: "REPEAT",
        times: "TIMES",
        end_repeat: "END REPEAT",
        while_: "WHILE",
        do_: "DO",
        end_while: "END WHILE",
        if_: "IF",
        then: "THEN",
        else_: "ELSE",
        border: "BORDER",
        not_border: "NOT_BORDER",
        procedure: "PROCEDURE",
        end_procedure: "END PROCEDURE",
        name: "name",
    },
    title: "Ant",
    new: "New",
    open: "Open…",
    save: "Save",
    save_as: "Save as…",
    examples: "Examples",
    help: "Help",
    settings: "Settings",
    theme: "Theme",
    theme_system: "System",
    theme_light: "Light",
    theme_dark: "Dark",
    run: "Run",
    step_once: "Step by step",
    pause: "Pause",
    stop: "Stop",
    reset: "Reset",
    check: "Check",
    format: "Format",
    speed: "Speed",
    instant: "Instant",
    table: "Table",
    columns: "Columns",
    rows: "Rows",
    show_grid: "Show grid",
    show_trail: "Show the ant's path",
    program: "Program",
    untitled: "untitled",
    ready: "Ready",
    running: "Running…",
    paused: "Paused",
    finished: "Execution complete",
    stopped: "Execution stopped",
    no_errors: "The program has no errors",
    error_line: "Error on line",
    position: "Position",
    direction: "Direction",
    moves: "Moves",
    pushes: "Letters pushed",
    dirs: ["↓ down", "← left", "↑ up", "→ right"],
    rotate: "Rotate",
    clear: "Clear",
    keep_result: "Keep result",
    keep_result_hint: "The table after the run becomes the starting table",
    edit_hint: "Drag the ant or the letters with the mouse · click the ant to turn it · click a cell and type letters · right click erases",
    unsaved_title: "Unsaved changes",
    unsaved_text: "The program has unsaved changes. Save them?",
    discard: "Don't save",
    cancel: "Cancel",
    file_error: "Could not open/save the file",
    ant_files: "Ant programs",
    problems: "Problems",
    no_problems: "No problems found",
    live_check: "Check the program while I type",
    did_you_mean: "Did you mean",
    fix: "Fix",
    line_short: "line",
    summary: |e, w| match (e, w) {
        (0, 0) => "The program has no errors".into(),
        (0, w) => format!("No errors · warnings: {w}"),
        (e, 0) => format!("Errors: {e}"),
        (e, w) => format!("Errors: {e} · warnings: {w}"),
    },
    help_text: &[
        (
            "Table",
            "The ant moves on the cells of a table that holds letters. When it walks into a letter, it pushes it one cell further. It cannot push two letters at once, nor push a letter off the table. The cells it walked through stay coloured.",
        ),
        (
            "Editing the table",
            "Drag the ant and the letters with the mouse; a letter dropped on another swaps places with it, and one dragged off the table is deleted. Clicking the ant turns it. To add letters, click a cell and type as in a text: the cursor moves on to the next cell by itself; Backspace, Delete, the arrows and Enter work as usual, Esc finishes. A right click erases a letter. The table is saved together with the program.",
        ),
        (
            "UP  DOWN  LEFT  RIGHT",
            "Moves the ant one cell in that direction.",
        ),
        (
            "BEGIN … END",
            "The main program. Procedures are written before it.",
        ),
        (
            "REPEAT N TIMES … END REPEAT",
            "Repeats the commands N times.",
        ),
        (
            "WHILE condition DO … END WHILE",
            "Repeats the commands while the condition is true.",
        ),
        (
            "IF condition THEN command1 ELSE command2",
            "Runs command1 if the condition is true, otherwise command2 (ELSE is optional).",
        ),
        (
            "BORDER   NOT_BORDER",
            "BORDER is true when the ant faces the edge of the table. The ant faces the direction it last moved in.",
        ),
        (
            "PROCEDURE name … END PROCEDURE",
            "Defines a procedure; call it by writing its name. Names may contain letters, digits and _.",
        ),
        (
            "// comment",
            "Text after // to the end of the line is ignored.",
        ),
        (
            "Checking",
            "The program is checked while you type: errors are underlined in red, warnings in yellow. The \"Problems\" list shows every mistake; click one to jump to its line, and \"Fix\" replaces the misspelled word.",
        ),
        (
            "Language",
            "Keywords may be written in English, Romanian or Russian, in upper or lower case.",
        ),
    ],
    about: "Ant — an executor for learning algorithms. Rewritten in Rust after the original \"Furnica\" (Ant v2.00, 1999).",
    author: "Author",
    shortcuts: "F5 — run · F6 — step · Esc — stop · F7 — check · Ctrl+Shift+F — format · Ctrl+S/O/N — files",
};
