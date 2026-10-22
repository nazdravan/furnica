//! Limbajul Furnicii: lexer, parser și compilator în instrucțiuni simple.
//!
//! Cuvintele-cheie sunt acceptate în română (cu sau fără diacritice),
//! engleză și rusă, indiferent de majuscule — ca în programul original,
//! astfel încât fișierele `.ant` vechi rulează fără modificări. Față de
//! original, blocurile se pot include oricum unul în altul.

use std::collections::HashMap;
use std::ops::Range;

use crate::world::Dir;

/// Condiția E_MARGINE (negată: NU_E_MARGINE).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cond {
    pub negated: bool,
}

/// Instrucțiuni pentru mașina virtuală (vezi `vm.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instr {
    /// Mută furnica cu o celulă.
    Act(Dir),
    /// Pune un contor nou pe stiva de repetări.
    PushCount(u32),
    /// Dacă contorul din vârf e 0 îl scoate și sare la țintă, altfel îl scade.
    CountDown(usize),
    Jump(usize),
    /// Sare la țintă dacă condiția este falsă.
    JumpUnless(Cond, usize),
    Call(usize),
    Return,
    Halt,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub code: Vec<Instr>,
    /// Linia din sursă (de la 1) pentru fiecare instrucțiune.
    pub lines: Vec<usize>,
    pub entry: usize,
}

/// Tipul unui bloc care se închide cu SFÎRȘIT <tip>.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Procedure,
    Repeat,
    Cycle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Un semn sau cuvânt-cheie pus unde nu-i este locul.
    UnexpectedToken(String),
    /// Un cuvânt care nu este nici comandă, nici procedură.
    UnknownWord(String),
    UnknownCondition(String),
    /// După DACĂ / ALTFEL sau la sfârșit lipsește comanda.
    ExpectedCommand,
    ExpectedNumber,
    ExpectedName,
    ExpectedCondition,
    /// Blocul deschis pe acest rând nu este închis.
    MissingEnd(BlockKind),
    WrongEnd {
        expected: BlockKind,
    },
    /// SFÎRȘIT <tip> fără bloc deschis.
    StrayEnd,
    /// ALTFEL fără DACĂ.
    StrayElse,
    DuplicateProcedure(String),
    ProcedureInMain,
    /// Comenzi scrise înainte de ÎNCEPUT.
    CommandOutsideMain,
    /// SFÎRȘIT fără ÎNCEPUT.
    MissingBegin,
    /// Al doilea ÎNCEPUT.
    DuplicateBegin,
    MissingMainEnd,
    TextAfterProgram,
    NumberTooLarge,
}

/// O eroare găsită înainte de rulare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompileError {
    pub line: usize,
    /// Poziția în text (octeți); poate fi goală la sfârșitul textului.
    pub span: Range<usize>,
    pub kind: ErrorKind,
    /// Cuvântul care probabil trebuia scris în loc (corectare rapidă).
    pub hint: Option<String>,
}

impl CompileError {
    fn new(line: usize, span: Range<usize>, kind: ErrorKind) -> CompileError {
        CompileError {
            line,
            span,
            kind,
            hint: None,
        }
    }
}

// ---------------------------------------------------------------- Lexer ----

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Word(String),
    Number(u64),
}

#[derive(Clone, Debug)]
struct Token {
    tok: Tok,
    /// Textul original, pentru mesaje de eroare.
    text: String,
    line: usize,
    span: Range<usize>,
}

/// Aduce un cuvânt la forma canonică: majuscule, fără diacritice.
pub fn normalize(word: &str) -> String {
    word.chars()
        .flat_map(char::to_uppercase)
        .map(|c| match c {
            'Ă' | 'Â' => 'A',
            'Î' => 'I',
            'Ș' | 'Ş' => 'S',
            'Ț' | 'Ţ' => 'T',
            'Ё' => 'Е',
            c => c,
        })
        .collect()
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn lex(src: &str, errors: &mut Vec<CompileError>) -> Vec<Token> {
    let mut out = Vec::new();
    let mut it = src.char_indices().peekable();
    let mut line = 1;
    while let Some(&(i, c)) = it.peek() {
        match c {
            '\n' => {
                line += 1;
                it.next();
            }
            c if c.is_whitespace() => {
                it.next();
            }
            '/' => {
                it.next();
                if it.peek().is_some_and(|&(_, c)| c == '/') {
                    while it.peek().is_some_and(|&(_, c)| c != '\n') {
                        it.next();
                    }
                } else {
                    errors.push(CompileError::new(
                        line,
                        i..i + 1,
                        ErrorKind::UnexpectedToken("/".into()),
                    ));
                }
            }
            c if is_word_char(c) => {
                let mut end = i;
                while let Some(&(j, c)) = it.peek() {
                    if !is_word_char(c) {
                        break;
                    }
                    end = j + c.len_utf8();
                    it.next();
                }
                let text = &src[i..end];
                let tok = if text.chars().all(|c| c.is_ascii_digit()) {
                    // Numerele prea mari sunt semnalate de REPETĂ.
                    Tok::Number(text.parse().unwrap_or(u64::MAX))
                } else {
                    Tok::Word(normalize(text))
                };
                out.push(Token {
                    tok,
                    text: text.into(),
                    line,
                    span: i..end,
                });
            }
            other => {
                it.next();
                let span = i..i + other.len_utf8();
                errors.push(CompileError::new(
                    line,
                    span,
                    ErrorKind::UnexpectedToken(other.to_string()),
                ));
            }
        }
    }
    out
}

// ------------------------------------------------------------- Keywords ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kw {
    Begin,
    End,
    Left,
    Right,
    Up,
    Down,
    Repeat,
    Times,
    While,
    Do,
    If,
    Then,
    Else,
    Procedure,
    Border,
    NotBorder,
    Not,
}

fn keyword(w: &str) -> Option<Kw> {
    Some(match w {
        "INCEPUT" | "BEGIN" | "НАЧАЛО" => Kw::Begin,
        "SFIRSIT" | "SFARSIT" | "SFIRSITUL" | "SFARSITUL" | "END" | "КОНЕЦ" => Kw::End,
        "STINGA" | "STANGA" | "LEFT" | "ВЛЕВО" => Kw::Left,
        "DREAPTA" | "RIGHT" | "ВПРАВО" => Kw::Right,
        "SUS" | "UP" | "ВВЕРХ" => Kw::Up,
        "JOS" | "DOWN" | "ВНИЗ" => Kw::Down,
        "REPETA" | "REPEAT" | "ПОВТОРИ" => Kw::Repeat,
        "ORI" | "TIMES" | "РАЗ" | "РАЗА" => Kw::Times,
        "CIT" | "CAT" | "WHILE" | "ПОКА" => Kw::While,
        // În rusa.lng originală „СДЕЛАЙ” începe cu un C latin.
        "EXECUTA" | "DO" | "СДЕЛАЙ" | "CДЕЛАЙ" | "ДЕЛАЙ" => Kw::Do,
        "DACA" | "IF" | "ЕСЛИ" => Kw::If,
        "ATUNCI" | "THEN" | "ТОГДА" => Kw::Then,
        "ALTFEL" | "ELSE" | "ИНАЧЕ" => Kw::Else,
        "PROCEDURA" | "PROCEDURE" | "ПРОЦЕДУРА" => Kw::Procedure,
        "E_MARGINE" | "BORDER" | "IS_BORDER" | "КРАЙ" => Kw::Border,
        "NU_E_MARGINE" | "NOT_BORDER" | "НЕ_КРАЙ" => Kw::NotBorder,
        "NU" | "NOT" | "НЕ" => Kw::Not,
        _ => return None,
    })
}

/// Cuvintele care pot apărea în locul unei comenzi (pentru sugestii).
pub(crate) const COMMAND_WORDS: &[&str] = &[
    "SUS",
    "JOS",
    "STINGA",
    "DREAPTA",
    "REPETA",
    "CIT",
    "EXECUTA",
    "DACA",
    "ATUNCI",
    "ALTFEL",
    "PROCEDURA",
    "INCEPUT",
    "SFIRSIT",
    "UP",
    "DOWN",
    "LEFT",
    "RIGHT",
    "REPEAT",
    "WHILE",
    "IF",
    "THEN",
    "ELSE",
    "PROCEDURE",
    "BEGIN",
    "END",
    "ВВЕРХ",
    "ВНИЗ",
    "ВЛЕВО",
    "ВПРАВО",
    "ПОВТОРИ",
    "ПОКА",
    "СДЕЛАЙ",
    "ЕСЛИ",
    "ТОГДА",
    "ИНАЧЕ",
    "ПРОЦЕДУРА",
    "НАЧАЛО",
    "КОНЕЦ",
];

pub(crate) const CONDITION_WORDS: &[&str] = &[
    "E_MARGINE",
    "NU_E_MARGINE",
    "BORDER",
    "NOT_BORDER",
    "КРАЙ",
    "НЕ_КРАЙ",
];

/// Cuvintele care pot urma după SFÎRȘIT.
const END_KIND_WORDS: &[&str] = &[
    "CIT",
    "REPETA",
    "PROCEDURA",
    "WHILE",
    "REPEAT",
    "PROCEDURE",
    "ПОКА",
    "ПОВТОРИ",
    "ПРОЦЕДУРА",
];

fn end_kind(kw: Kw) -> Option<BlockKind> {
    Some(match kw {
        Kw::Procedure => BlockKind::Procedure,
        Kw::Repeat => BlockKind::Repeat,
        Kw::While => BlockKind::Cycle,
        _ => return None,
    })
}

fn dir_of(kw: Kw) -> Option<Dir> {
    Some(match kw {
        Kw::Up => Dir::Up,
        Kw::Down => Dir::Down,
        Kw::Left => Dir::Left,
        Kw::Right => Dir::Right,
        _ => return None,
    })
}

// ------------------------------------------------------------------- AST ---

#[derive(Debug)]
pub(crate) struct Stmt {
    pub kind: StmtKind,
    pub line: usize,
    /// Poziția cuvântului care începe comanda.
    pub span: Range<usize>,
}

#[derive(Debug)]
pub(crate) enum StmtKind {
    Move(Dir),
    Repeat {
        n: u32,
        n_span: Range<usize>,
        body: Vec<Stmt>,
    },
    While(Cond, Vec<Stmt>),
    If(Cond, Box<Stmt>, Option<Box<Stmt>>),
    /// Apelul unei proceduri: numele normalizat, textul original și poziția numelui.
    Call(String, String, Range<usize>),
    /// Loc rămas gol din cauza unei erori.
    Invalid,
}

#[derive(Debug)]
pub(crate) struct ProcDef {
    pub name: String,
    pub text: String,
    pub body: Vec<Stmt>,
    pub line: usize,
    pub span: Range<usize>,
}

#[derive(Debug)]
pub(crate) struct Ast {
    pub procs: Vec<ProcDef>,
    pub main: Vec<Stmt>,
    /// Poziția lui ÎNCEPUT (dacă există).
    pub main_at: Option<(usize, Range<usize>)>,
}

// ---------------------------------------------------------------- Parser ---

/// Ce urmează după SFÎRȘIT pe același rând.
enum EndWord {
    /// Nimic: sfârșitul programului.
    Bare,
    Kind(BlockKind),
    /// Un cuvânt necunoscut, probabil tipul blocului scris greșit.
    Unknown(Token),
}

/// Parser care nu se oprește la prima greșeală: o notează, sare peste restul
/// rândului și continuă, ca analizorul să poată arăta toate erorile odată.
struct Parser<'a> {
    toks: Vec<Token>,
    pos: usize,
    end_line: usize,
    end_pos: usize,
    errors: &'a mut Vec<CompileError>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.pos)
    }

    fn kw_at(&self, i: usize) -> Option<Kw> {
        match self.toks.get(i)?.tok {
            Tok::Word(ref w) => keyword(w),
            _ => None,
        }
    }

    fn peek_kw(&self) -> Option<Kw> {
        self.kw_at(self.pos)
    }

    fn eat_kw(&mut self, kw: Kw) -> bool {
        let found = self.peek_kw() == Some(kw);
        self.pos += usize::from(found);
        found
    }

    /// Rândul și poziția cuvântului curent (sau sfârșitul textului).
    fn here(&self) -> (usize, Range<usize>) {
        self.peek()
            .map_or((self.end_line, self.end_pos..self.end_pos), |t| {
                (t.line, t.span.clone())
            })
    }

    fn error(&mut self, line: usize, span: Range<usize>, kind: ErrorKind, hint: Option<String>) {
        self.errors.push(CompileError {
            line,
            span,
            kind,
            hint,
        });
    }

    fn error_here(&mut self, kind: ErrorKind) {
        let (line, span) = self.here();
        self.error(line, span, kind, None);
    }

    /// Cuvinte la care se oprește un bloc.
    fn at_block_end(&self) -> bool {
        self.peek().is_none() || matches!(self.peek_kw(), Some(Kw::End | Kw::Procedure | Kw::Begin))
    }

    /// După o eroare: sare peste cuvântul greșit și restul rândului,
    /// dar nu peste cuvintele care închid blocuri.
    fn recover(&mut self, before: usize) {
        if self.pos == before && !self.at_block_end() {
            self.pos += 1;
        }
        let Some(line) = self.pos.checked_sub(1).map(|i| self.toks[i].line) else {
            return;
        };
        while self.peek().is_some_and(|t| t.line == line) && !self.at_block_end() {
            self.pos += 1;
        }
    }

    /// `[NU] E_MARGINE` sau `NU_E_MARGINE`, pe rândul cuvântului `line`.
    fn cond(&mut self, line: usize) -> Cond {
        let not = self.eat_kw(Kw::Not);
        let negated = match self.peek_kw() {
            Some(Kw::Border) => false,
            Some(Kw::NotBorder) => true,
            _ => {
                match self.peek().cloned() {
                    Some(Token {
                        tok: Tok::Word(w),
                        text,
                        line: l,
                        span,
                    }) if keyword(&w).is_none() && l == line => {
                        let hint = crate::check::suggest(&w, &text, CONDITION_WORDS);
                        self.error(l, span, ErrorKind::UnknownCondition(text), hint);
                        self.pos += 1;
                    }
                    _ => self.error_here(ErrorKind::ExpectedCondition),
                }
                return Cond { negated: not };
            }
        };
        self.pos += 1;
        Cond {
            negated: negated != not,
        }
    }

    /// Ce urmează după cuvântul SFÎRȘIT de la poziția curentă.
    fn end_word(&self) -> EndWord {
        let line = self.toks[self.pos].line;
        match self.toks.get(self.pos + 1) {
            Some(t) if t.line == line => match &t.tok {
                Tok::Word(w) => match keyword(w) {
                    Some(kw) => end_kind(kw).map_or(EndWord::Bare, EndWord::Kind),
                    None => EndWord::Unknown(t.clone()),
                },
                Tok::Number(_) => EndWord::Bare,
            },
            _ => EndWord::Bare,
        }
    }

    /// Tipul de bloc scris greșit după SFÎRȘIT: îl raportează cu o sugestie.
    fn bad_end_word(&mut self, t: Token) {
        let hint = match &t.tok {
            Tok::Word(w) => crate::check::suggest(w, &t.text, END_KIND_WORDS),
            Tok::Number(_) => None,
        };
        self.error(t.line, t.span, ErrorKind::UnknownWord(t.text), hint);
    }

    /// Citește `SFÎRȘIT <tip>` la capătul unui bloc.
    fn end(&mut self, kind: BlockKind, open_line: usize, open_span: &Range<usize>) {
        if self.peek_kw() != Some(Kw::End) {
            self.error(
                open_line,
                open_span.clone(),
                ErrorKind::MissingEnd(kind),
                None,
            );
            return;
        }
        match self.end_word() {
            EndWord::Kind(found) => {
                if found != kind {
                    let t = &self.toks[self.pos + 1];
                    let (line, span) = (t.line, t.span.clone());
                    self.error(line, span, ErrorKind::WrongEnd { expected: kind }, None);
                }
                self.pos += 2;
            }
            EndWord::Unknown(t) => {
                self.bad_end_word(t);
                self.pos += 2;
            }
            // Un SFÎRȘIT simplu încheie programul principal; o procedură nu-l poate conține.
            EndWord::Bare if kind == BlockKind::Procedure => {
                self.error_here(ErrorKind::WrongEnd { expected: kind });
                self.pos += 1;
            }
            EndWord::Bare => {
                self.error(
                    open_line,
                    open_span.clone(),
                    ErrorKind::MissingEnd(kind),
                    None,
                );
            }
        }
    }

    /// SFÎRȘIT <tip> rătăcit: îl raportează și îl sare împreună cu tipul lui.
    fn stray_end(&mut self) {
        match self.end_word() {
            EndWord::Unknown(t) => self.bad_end_word(t),
            _ => self.error_here(ErrorKind::StrayEnd),
        }
        self.pos += 2;
    }

    /// Instrucțiuni până la SFÎRȘIT, PROCEDURA sau ÎNCEPUT.
    fn block(&mut self) -> Vec<Stmt> {
        let mut out = Vec::new();
        while !self.at_block_end() {
            let before = self.pos;
            match self.stmt() {
                Some(s) => out.push(s),
                None => self.recover(before),
            }
        }
        out
    }

    /// Comanda de după ATUNCI sau ALTFEL.
    fn branch(&mut self) -> Box<Stmt> {
        let (line, span) = self.here();
        let before = self.pos;
        let stmt = if self.at_block_end() {
            self.error_here(ErrorKind::ExpectedCommand);
            None
        } else {
            self.stmt()
        };
        Box::new(stmt.unwrap_or_else(|| {
            self.recover(before);
            Stmt {
                kind: StmtKind::Invalid,
                line,
                span,
            }
        }))
    }

    /// Numele de după EXECUTĂ, pe același rând (apel scris ca în Cangurul).
    fn call_after_do(&self, line: usize) -> Option<Token> {
        let t = self.toks.get(self.pos + 1)?;
        match &t.tok {
            Tok::Word(w) if keyword(w).is_none() && t.line == line => Some(t.clone()),
            _ => None,
        }
    }

    fn stmt(&mut self) -> Option<Stmt> {
        let Some(t) = self.peek().cloned() else {
            self.error_here(ErrorKind::ExpectedCommand);
            return None;
        };
        let Token { line, span, .. } = t;
        let kw = match &t.tok {
            Tok::Word(w) => match keyword(w) {
                Some(kw) => kw,
                // Orice alt cuvânt este numele unei proceduri; existența ei o verifică `check`.
                None => {
                    self.pos += 1;
                    return Some(Stmt {
                        kind: StmtKind::Call(w.clone(), t.text, span.clone()),
                        line,
                        span,
                    });
                }
            },
            Tok::Number(_) => {
                self.error(line, span, ErrorKind::UnexpectedToken(t.text), None);
                return None;
            }
        };
        if let Some(d) = dir_of(kw) {
            self.pos += 1;
            return Some(Stmt {
                kind: StmtKind::Move(d),
                line,
                span,
            });
        }
        let kind = match kw {
            Kw::End | Kw::Procedure | Kw::Begin => {
                self.error_here(ErrorKind::ExpectedCommand);
                return None;
            }
            Kw::Else => {
                self.error(line, span, ErrorKind::StrayElse, None);
                return None;
            }
            Kw::Do => match self.call_after_do(line) {
                Some(name) => {
                    self.pos += 2;
                    let Tok::Word(w) = name.tok else {
                        unreachable!()
                    };
                    StmtKind::Call(w, name.text, name.span)
                }
                None => {
                    self.error(line, span, ErrorKind::UnexpectedToken(t.text), None);
                    return None;
                }
            },
            Kw::Times | Kw::Then | Kw::Not | Kw::Border | Kw::NotBorder => {
                self.error(line, span, ErrorKind::UnexpectedToken(t.text), None);
                return None;
            }
            Kw::Left | Kw::Right | Kw::Up | Kw::Down => unreachable!(),
            Kw::Repeat => {
                self.pos += 1;
                let (n, n_span) = self.repeat_count(line, &span);
                self.eat_kw(Kw::Times);
                let body = self.block();
                self.end(BlockKind::Repeat, line, &span);
                StmtKind::Repeat { n, n_span, body }
            }
            Kw::While => {
                self.pos += 1;
                let c = self.cond(line);
                self.eat_kw(Kw::Do);
                let body = self.block();
                self.end(BlockKind::Cycle, line, &span);
                StmtKind::While(c, body)
            }
            Kw::If => {
                self.pos += 1;
                let c = self.cond(line);
                self.eat_kw(Kw::Then);
                let then = self.branch();
                let other = self.eat_kw(Kw::Else).then(|| self.branch());
                StmtKind::If(c, then, other)
            }
        };
        Some(Stmt { kind, line, span })
    }

    /// Numărul de după REPETĂ.
    fn repeat_count(&mut self, line: usize, span: &Range<usize>) -> (u32, Range<usize>) {
        if let Some(Token {
            tok: Tok::Number(n),
            span: n_span,
            ..
        }) = self.peek().cloned()
        {
            self.pos += 1;
            let n = u32::try_from(n).unwrap_or_else(|_| {
                self.error(line, n_span.clone(), ErrorKind::NumberTooLarge, None);
                1
            });
            return (n, n_span);
        }
        self.error_here(ErrorKind::ExpectedNumber);
        // Sare peste ce a scris elevul în loc de număr, până la ORI.
        while let Some(Token {
            tok: Tok::Word(w),
            line: l,
            ..
        }) = self.peek()
            && *l == line
            && keyword(w).is_none()
        {
            self.pos += 1;
        }
        // „REPETĂ de 3 ORI”: numărul de după cuvântul în plus e folosit.
        match self.peek().cloned() {
            Some(Token {
                tok: Tok::Number(n),
                span: n_span,
                line: l,
                ..
            }) if l == line => {
                self.pos += 1;
                (u32::try_from(n).unwrap_or(1), n_span)
            }
            _ => (1, span.clone()),
        }
    }

    fn procedure(&mut self, procs: &mut Vec<ProcDef>) {
        let Token { line, span, .. } = self.toks[self.pos].clone();
        self.pos += 1;
        let name = match self.peek().cloned() {
            Some(Token {
                tok: Tok::Word(w),
                text,
                span: name_span,
                line: name_line,
            }) if keyword(&w).is_none() && name_line == line => {
                self.pos += 1;
                if procs.iter().any(|p| p.name == w) {
                    self.error(
                        line,
                        name_span.clone(),
                        ErrorKind::DuplicateProcedure(text.clone()),
                        None,
                    );
                }
                Some((w, text, name_span))
            }
            _ => {
                self.error(line, span.clone(), ErrorKind::ExpectedName, None);
                None
            }
        };
        let body = self.block();
        self.end(BlockKind::Procedure, line, &span);
        if let Some((name, text, span)) = name
            && !procs.iter().any(|p| p.name == name)
        {
            procs.push(ProcDef {
                name,
                text,
                body,
                line,
                span,
            });
        }
    }

    /// Conținutul dintre ÎNCEPUT și SFÎRȘIT.
    fn main_body(&mut self, begin: &Token, procs: &mut Vec<ProcDef>) -> Vec<Stmt> {
        let mut body = Vec::new();
        loop {
            body.extend(self.block());
            let Some(t) = self.peek().cloned() else {
                self.error(
                    begin.line,
                    begin.span.clone(),
                    ErrorKind::MissingMainEnd,
                    None,
                );
                return body;
            };
            match self.peek_kw() {
                Some(Kw::End) => match self.end_word() {
                    EndWord::Bare => {
                        self.pos += 1;
                        return body;
                    }
                    _ => self.stray_end(),
                },
                Some(Kw::Begin) => {
                    self.error(t.line, t.span, ErrorKind::DuplicateBegin, None);
                    self.pos += 1;
                }
                Some(Kw::Procedure) => {
                    self.error(t.line, t.span, ErrorKind::ProcedureInMain, None);
                    self.procedure(procs);
                }
                _ => unreachable!("blocul se oprește doar la SFÎRȘIT, ÎNCEPUT sau PROCEDURA"),
            }
        }
    }

    fn program(&mut self) -> Ast {
        let mut procs = Vec::new();
        let mut main: Option<Vec<Stmt>> = None;
        let mut main_at = None;
        let mut loose: Vec<Stmt> = Vec::new();
        let mut reported_after_main = false;

        while let Some(t) = self.peek().cloned() {
            if main.is_some() && !reported_after_main {
                self.error(t.line, t.span.clone(), ErrorKind::TextAfterProgram, None);
                reported_after_main = true;
            }
            match self.peek_kw() {
                Some(Kw::Begin) => {
                    self.pos += 1;
                    let body = self.main_body(&t, &mut procs);
                    if main.is_none() {
                        main = Some(body);
                        main_at = Some((t.line, t.span.clone()));
                    }
                }
                Some(Kw::Procedure) => self.procedure(&mut procs),
                Some(Kw::End) => match self.end_word() {
                    EndWord::Bare => {
                        if main.is_none() {
                            self.error(t.line, t.span.clone(), ErrorKind::MissingBegin, None);
                            main = Some(std::mem::take(&mut loose));
                        }
                        self.pos += 1;
                    }
                    _ => self.stray_end(),
                },
                _ => {
                    let before = self.pos;
                    match self.stmt() {
                        Some(s) if main.is_none() => loose.push(s),
                        Some(_) => {}
                        None => self.recover(before),
                    }
                }
            }
        }

        let main = match main {
            Some(m) => {
                // Programul fără ÎNCEPUT/SFÎRȘIT e permis, dar comenzile libere înainte de ÎNCEPUT nu.
                if let Some(s) = loose.first() {
                    self.error(s.line, s.span.clone(), ErrorKind::CommandOutsideMain, None);
                }
                m
            }
            None => loose,
        };
        Ast {
            procs,
            main,
            main_at,
        }
    }
}

/// Rezultatul analizei: toate erorile, avertismentele și programul (dacă nu are erori).
#[derive(Debug, Default)]
pub struct Analysis {
    pub errors: Vec<CompileError>,
    pub warnings: Vec<crate::check::Warning>,
    pub program: Option<Program>,
}

/// Analizează textul: găsește toate greșelile și, dacă nu sunt, îl compilează.
pub fn analyze(src: &str) -> Analysis {
    let mut errors = Vec::new();
    let toks = lex(src, &mut errors);
    let mut p = Parser {
        toks,
        pos: 0,
        end_line: src.lines().count().max(1),
        end_pos: src.trim_end().len(),
        errors: &mut errors,
    };
    let ast = p.program();
    let warnings = crate::check::check(&ast, &mut errors);

    errors.sort_by_key(|e| (e.span.start, e.line));
    errors.dedup_by(|a, b| a.span == b.span && a.line == b.line);
    let program = errors.is_empty().then(|| generate(&ast, src));
    Analysis {
        errors,
        warnings,
        program,
    }
}

/// Compilează textul unui program; întoarce prima eroare.
#[cfg(test)]
pub fn compile(src: &str) -> Result<Program, CompileError> {
    let mut a = analyze(src);
    a.program.ok_or_else(|| a.errors.swap_remove(0))
}

// -------------------------------------------------------------- Compiler ---

struct Compiler<'a> {
    code: Vec<Instr>,
    lines: Vec<usize>,
    procs: &'a HashMap<String, usize>,
    /// (index instrucțiune Call, nume procedură) — completate la final.
    calls: Vec<(usize, String)>,
}

impl Compiler<'_> {
    fn emit(&mut self, i: Instr, line: usize) -> usize {
        self.code.push(i);
        self.lines.push(line);
        self.code.len() - 1
    }

    fn patch(&mut self, at: usize, target: usize) {
        match &mut self.code[at] {
            Instr::CountDown(t) | Instr::Jump(t) | Instr::JumpUnless(_, t) => *t = target,
            _ => unreachable!(),
        }
    }

    fn stmts(&mut self, body: &[Stmt]) {
        body.iter().for_each(|s| self.stmt(s));
    }

    fn stmt(&mut self, s: &Stmt) {
        let line = s.line;
        match &s.kind {
            StmtKind::Move(d) => {
                self.emit(Instr::Act(*d), line);
            }
            StmtKind::Repeat { n, body, .. } => {
                self.emit(Instr::PushCount(*n), line);
                let top = self.emit(Instr::CountDown(0), line);
                self.stmts(body);
                self.emit(Instr::Jump(top), line);
                let end = self.code.len();
                self.patch(top, end);
            }
            StmtKind::While(c, body) => {
                let top = self.emit(Instr::JumpUnless(*c, 0), line);
                self.stmts(body);
                self.emit(Instr::Jump(top), line);
                let end = self.code.len();
                self.patch(top, end);
            }
            StmtKind::If(c, then, other) => {
                let test = self.emit(Instr::JumpUnless(*c, 0), line);
                self.stmt(then);
                if let Some(other) = other {
                    let skip = self.emit(Instr::Jump(0), line);
                    let else_at = self.code.len();
                    self.patch(test, else_at);
                    self.stmt(other);
                    let end = self.code.len();
                    self.patch(skip, end);
                } else {
                    let end = self.code.len();
                    self.patch(test, end);
                }
            }
            StmtKind::Call(name, ..) => {
                debug_assert!(self.procs.contains_key(name));
                let at = self.emit(Instr::Call(0), line);
                self.calls.push((at, name.clone()));
            }
            StmtKind::Invalid => {}
        }
    }
}

/// Generează codul pentru un program fără erori.
fn generate(ast: &Ast, src: &str) -> Program {
    let last_line = src.lines().count().max(1);
    let index: HashMap<String, usize> = ast
        .procs
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.clone(), i))
        .collect();
    let mut c = Compiler {
        code: Vec::new(),
        lines: Vec::new(),
        procs: &index,
        calls: Vec::new(),
    };

    c.stmts(&ast.main);
    c.emit(Instr::Halt, last_line);

    let mut starts = Vec::new();
    for p in &ast.procs {
        starts.push(c.code.len());
        c.stmts(&p.body);
        c.emit(Instr::Return, p.line);
    }
    for (at, name) in std::mem::take(&mut c.calls) {
        c.code[at] = Instr::Call(starts[index[&name]]);
    }
    Program {
        code: c.code,
        lines: c.lines,
        entry: 0,
    }
}

/// Primul cuvânt al rândului (fără comentariu) și dacă mai urmează altele.
fn first_keyword(line: &str) -> Option<(Kw, bool)> {
    let code = line.split("//").next().unwrap_or("");
    let mut words = code.split_whitespace();
    let kw = keyword(&normalize(words.next()?))?;
    Some((kw, words.next().is_some()))
}

/// Rândul cu ÎNCEPUT (înaintea lui se inserează procedurile).
pub fn is_begin_line(line: &str) -> bool {
    matches!(first_keyword(line), Some((Kw::Begin, _)))
}

/// Rândul care conține doar SFÎRȘIT, adică sfârșitul programului principal.
pub fn is_main_end_line(line: &str) -> bool {
    matches!(first_keyword(line), Some((Kw::End, false)))
}

// ------------------------------------------------------- Syntax colouring ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenClass {
    Plain,
    /// SUS, JOS, STÎNGA, DREAPTA.
    Action,
    Control,
    Condition,
    Number,
    /// Numele procedurilor.
    Name,
    Comment,
}

/// Împarte textul în segmente colorate pentru editor (nu dă erori).
pub fn highlight(src: &str) -> Vec<(Range<usize>, TokenClass)> {
    let mut out = Vec::new();
    let mut it = src.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        let mut end = i + c.len_utf8();
        let class = if c == '/' && it.peek().is_some_and(|&(_, c)| c == '/') {
            while let Some(&(j, c)) = it.peek() {
                if c == '\n' {
                    break;
                }
                end = j + c.len_utf8();
                it.next();
            }
            TokenClass::Comment
        } else if is_word_char(c) {
            while let Some(&(j, c)) = it.peek() {
                if !is_word_char(c) {
                    break;
                }
                end = j + c.len_utf8();
                it.next();
            }
            let word = &src[i..end];
            if word.chars().all(|c| c.is_ascii_digit()) {
                TokenClass::Number
            } else {
                match keyword(&normalize(word)) {
                    Some(kw) if dir_of(kw).is_some() => TokenClass::Action,
                    Some(Kw::Border | Kw::NotBorder | Kw::Not) => TokenClass::Condition,
                    Some(_) => TokenClass::Control,
                    None => TokenClass::Name,
                }
            }
        } else {
            TokenClass::Plain
        };
        out.push((i..end, class));
    }
    out
}

// ----------------------------------------------------------- Completare ----

/// Cuvântul este ALTFEL (în oricare limbă).
pub fn is_else(word: &str) -> bool {
    keyword(&normalize(word)) == Some(Kw::Else)
}

/// Numele procedurilor definite în program, așa cum sunt scrise.
pub fn procedure_names(src: &str) -> Vec<String> {
    let toks = lex(src, &mut Vec::new());
    let mut out: Vec<String> = Vec::new();
    for w in toks.windows(2) {
        if let (Tok::Word(a), Tok::Word(b)) = (&w[0].tok, &w[1].tok)
            && keyword(a) == Some(Kw::Procedure)
            && keyword(b).is_none()
            && w[0].line == w[1].line
            && !out.iter().any(|n| normalize(n) == *b)
        {
            out.push(w[1].text.clone());
        }
    }
    out
}

/// După SFÎRȘIT, pe același rând, urmează tipul blocului (sau un tip scris
/// greșit): SFÎRȘIT închide atunci un bloc, nu programul.
fn ends_block(toks: &[Token], i: usize) -> bool {
    toks.get(i + 1).is_some_and(|t| {
        t.line == toks[i].line
            && matches!(&t.tok, Tok::Word(w) if keyword(w).is_none_or(|k| end_kind(k).is_some()))
    })
}

/// Blocul cel mai interior rămas deschis la sfârșitul textului.
pub fn open_block(src: &str) -> Option<BlockKind> {
    let toks = lex(src, &mut Vec::new());
    let mut stack = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if let Tok::Word(w) = &toks[i].tok {
            match keyword(w) {
                Some(Kw::Begin) => stack.clear(),
                Some(Kw::Procedure) => {
                    stack.clear();
                    stack.push(BlockKind::Procedure);
                }
                Some(Kw::Repeat) => stack.push(BlockKind::Repeat),
                Some(Kw::While) => stack.push(BlockKind::Cycle),
                Some(Kw::End) if ends_block(&toks, i) => {
                    stack.pop();
                    i += 1;
                }
                Some(Kw::End) => stack.clear(),
                _ => {}
            }
        }
        i += 1;
    }
    stack.last().copied()
}

// ------------------------------------------------------------ Formatare ----

#[derive(Clone, Copy)]
enum Frame {
    /// REPETĂ, CÎT sau PROCEDURA, până la SFÎRȘIT <tip>.
    Block,
    /// Comanda de după DACĂ (`then`) sau ALTFEL; `started` după primul ei cuvânt.
    Branch { then: bool, started: bool },
    /// DACĂ și-a terminat comanda; poate urma ALTFEL.
    AwaitElse,
}

/// Urmărește blocurile deschise pentru a afla indentarea fiecărui rând.
#[derive(Default)]
struct Indenter {
    stack: Vec<Frame>,
    /// După DACĂ / CÎT urmează condiția (cu NU în față).
    expect_cond: bool,
    /// După condiția lui CÎT poate urma EXECUTĂ.
    expect_do: bool,
    /// După SFÎRȘIT urmează tipul blocului.
    after_end: bool,
}

impl Indenter {
    fn depth(&self) -> usize {
        self.stack
            .iter()
            .filter(|f| !matches!(f, Frame::AwaitElse))
            .count()
    }

    fn start_command(&mut self) {
        if let Some(Frame::Branch { started, .. }) = self.stack.last_mut() {
            *started = true;
        }
    }

    /// O comandă s-a terminat: închide ramurile DACĂ / ALTFEL din care făcea parte.
    fn complete(&mut self) {
        while let Some(Frame::Branch {
            then,
            started: true,
        }) = self.stack.last().copied()
        {
            self.stack.pop();
            if then {
                self.stack.push(Frame::AwaitElse);
                return;
            }
        }
    }

    /// Prelucrează cuvântul `toks[i]`; întoarce indentarea rândului dacă el începe rândul.
    fn token(&mut self, toks: &[Token], i: usize) -> usize {
        let tok = &toks[i].tok;
        let kw = match tok {
            Tok::Word(w) => keyword(w),
            Tok::Number(_) => None,
        };
        if std::mem::take(&mut self.after_end) {
            self.complete();
            return self.depth();
        }
        if kw != Some(Kw::Else) && matches!(self.stack.last(), Some(Frame::AwaitElse)) {
            self.stack.pop();
            self.complete();
        }
        if std::mem::take(&mut self.expect_cond) {
            match (tok, kw) {
                (_, Some(Kw::Not)) => {
                    self.expect_cond = true;
                    return self.depth();
                }
                (Tok::Word(_), Some(Kw::Border | Kw::NotBorder) | None) => return self.depth(),
                _ => {}
            }
        }
        if std::mem::take(&mut self.expect_do) && kw == Some(Kw::Do) {
            return self.depth();
        }
        let depth = self.depth();
        match (tok, kw) {
            (_, Some(Kw::Begin)) => {
                self.stack.clear();
                0
            }
            (_, Some(Kw::Procedure)) => {
                self.stack.clear();
                self.stack.push(Frame::Block);
                0
            }
            (_, Some(Kw::End)) if ends_block(toks, i) => {
                while let Some(f) = self.stack.pop() {
                    if matches!(f, Frame::Block) {
                        break;
                    }
                }
                self.after_end = true;
                self.depth()
            }
            // SFÎRȘIT singur: sfârșitul programului.
            (_, Some(Kw::End)) => {
                self.stack.clear();
                0
            }
            (_, Some(Kw::Repeat)) => {
                self.start_command();
                self.stack.push(Frame::Block);
                depth
            }
            (_, Some(Kw::While)) => {
                self.start_command();
                self.stack.push(Frame::Block);
                self.expect_cond = true;
                self.expect_do = true;
                depth
            }
            (_, Some(Kw::If)) => {
                self.start_command();
                self.stack.push(Frame::Branch {
                    then: true,
                    started: false,
                });
                self.expect_cond = true;
                depth
            }
            (_, Some(Kw::Else)) => {
                if matches!(self.stack.last(), Some(Frame::AwaitElse)) {
                    self.stack.pop();
                    self.stack.push(Frame::Branch {
                        then: false,
                        started: false,
                    });
                }
                depth
            }
            // EXECUTĂ <nume>: numele procedurii încheie comanda.
            (_, Some(Kw::Do)) => {
                self.start_command();
                depth
            }
            // Deplasările și apelurile de proceduri.
            (Tok::Word(_), Some(Kw::Up | Kw::Down | Kw::Left | Kw::Right) | None) => {
                self.start_command();
                self.complete();
                depth
            }
            _ => depth,
        }
    }
}

/// Reindentează programul: câte 4 spații pentru fiecare REPETĂ, CÎT, PROCEDURA
/// deschise și pentru comanda de pe rândul de după DACĂ / ALTFEL; între ÎNCEPUT
/// și SFÎRȘIT nu se indentează. Restul textului (cuvintele, rândurile goale,
/// comentariile) rămâne neschimbat.
pub fn format(src: &str) -> String {
    let toks = lex(src, &mut Vec::new());
    let mut ind = Indenter::default();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    for (n, line) in src.split('\n').enumerate() {
        if n > 0 {
            out.push('\n');
        }
        let mut indent = None;
        while i < toks.len() && toks[i].line == n + 1 {
            let d = ind.token(&toks, i);
            indent.get_or_insert(d);
            i += 1;
        }
        let text = line.trim();
        if !text.is_empty() {
            let depth = indent.unwrap_or_else(|| ind.depth());
            out.extend(std::iter::repeat_n("    ", depth));
            out.push_str(text);
        }
    }
    out
}

/// Formatarea completă (butonul „Formatează”): pune pe rânduri separate
/// comanda de după DACĂ … ATUNCI, ALTFEL și comanda de după ALTFEL, apoi
/// reindentează.
pub fn format_program(src: &str) -> String {
    let toks = lex(src, &mut Vec::new());
    let kw_at = |i: usize| match toks.get(i).map(|t| &t.tok) {
        Some(Tok::Word(w)) => keyword(w),
        _ => None,
    };
    // Pozițiile (octeți) unde începe un rând nou.
    let mut breaks = Vec::new();
    let mut break_before = |i: usize| {
        if let Some(t) = toks.get(i)
            && i > 0
            && toks[i - 1].line == t.line
        {
            breaks.push(t.span.start);
        }
    };
    for i in 0..toks.len() {
        match kw_at(i) {
            Some(Kw::If) => {
                let mut j = i + 1;
                while kw_at(j) == Some(Kw::Not) {
                    j += 1;
                }
                if matches!(toks.get(j).map(|t| &t.tok), Some(Tok::Word(_)))
                    && matches!(kw_at(j), Some(Kw::Border | Kw::NotBorder) | None)
                {
                    j += 1;
                }
                if kw_at(j) == Some(Kw::Then) {
                    j += 1;
                }
                break_before(j);
            }
            Some(Kw::Else) => {
                break_before(i);
                break_before(i + 1);
            }
            _ => {}
        }
    }
    let mut out = String::with_capacity(src.len() + breaks.len() * 8);
    let mut last = 0;
    for b in breaks {
        out.push_str(&src[last..b]);
        out.push('\n');
        last = b;
    }
    out.push_str(&src[last..]);
    format(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_nested_blocks() {
        let src = "// x\nPROCEDURA p\nREPETĂ 2 ORI\nCÎT NU_E_MARGINE EXECUTĂ\nJOS\nSFÎRȘIT CÎT\nSFÎRȘIT REPETĂ\nSFÎRȘIT PROCEDURA\n\nÎNCEPUT\n  p\nREPETĂ 3 ORI\n\tSUS  \nSFÎRȘIT REPETĂ\nSFÎRȘIT\n";
        assert_eq!(
            format(src),
            "// x\nPROCEDURA p\n    REPETĂ 2 ORI\n        CÎT NU_E_MARGINE EXECUTĂ\n            JOS\n        SFÎRȘIT CÎT\n    SFÎRȘIT REPETĂ\nSFÎRȘIT PROCEDURA\n\nÎNCEPUT\np\nREPETĂ 3 ORI\n    SUS\nSFÎRȘIT REPETĂ\nSFÎRȘIT\n"
        );
        // REPEAT după END este tipul blocului, nu o comandă nouă.
        assert_eq!(
            format(
                "BEGIN\nREPEAT 2 TIMES\nWHILE NOT_BORDER DO\nDOWN\nEND WHILE\nEND REPEAT\nUP\nEND"
            ),
            "BEGIN\nREPEAT 2 TIMES\n    WHILE NOT_BORDER DO\n        DOWN\n    END WHILE\nEND REPEAT\nUP\nEND"
        );
    }

    #[test]
    fn format_program_splits_if_else() {
        let src = "PROCEDURA p\nSUS\nSFÎRȘIT PROCEDURA\nÎNCEPUT\nDACĂ E_MARGINE ATUNCI SUS ALTFEL JOS\ndaca nu e_margine atunci stinga // gata\nDACĂ E_MARGINE ATUNCI DACĂ NU_E_MARGINE ATUNCI SUS ALTFEL JOS\nREPETĂ 2 ORI\nDACĂ E_MARGINE ATUNCI\nEXECUTĂ p\nALTFEL\nJOS\nSFÎRȘIT REPETĂ\nSFÎRȘIT\n";
        let out = format_program(src);
        assert_eq!(
            out,
            "PROCEDURA p\n    SUS\nSFÎRȘIT PROCEDURA\nÎNCEPUT\nDACĂ E_MARGINE ATUNCI\n    SUS\nALTFEL\n    JOS\ndaca nu e_margine atunci\n    stinga // gata\nDACĂ E_MARGINE ATUNCI\n    DACĂ NU_E_MARGINE ATUNCI\n        SUS\n    ALTFEL\n        JOS\nREPETĂ 2 ORI\n    DACĂ E_MARGINE ATUNCI\n        EXECUTĂ p\n    ALTFEL\n        JOS\nSFÎRȘIT REPETĂ\nSFÎRȘIT\n"
        );
        assert_eq!(compile(src).unwrap().code, compile(&out).unwrap().code);
    }

    #[test]
    fn completion_context() {
        let src = "PROCEDURA Patrat\nSUS\nSFÎRȘIT PROCEDURA\nprocedura Linie\nREPEAT 2\nCIT NU_E_MARGINE EXECUTA\nJOS\nEND WHILE\n";
        assert_eq!(procedure_names(src), vec!["Patrat", "Linie"]);
        assert_eq!(open_block(src), Some(BlockKind::Repeat));
        assert_eq!(open_block("ÎNCEPUT\nREPETĂ 2 ORI\nJOS\nEND REPEAT\n"), None);
        assert_eq!(open_block("PROCEDURA P\n"), Some(BlockKind::Procedure));
        assert_eq!(open_block("REPETĂ 2 ORI\nSFÎRȘIT\n"), None);
        assert!(is_else("altfel") && is_else("ИНАЧЕ") && !is_else("SUS"));
    }

    #[test]
    fn diacritics_and_case() {
        assert_eq!(normalize("Repetă"), "REPETA");
        assert_eq!(normalize("sfîrșit"), "SFIRSIT");
        assert_eq!(normalize("Stânga"), "STANGA");
        assert_eq!(normalize("вверх"), "ВВЕРХ");
    }

    #[test]
    fn simple_program() {
        let p = compile("inceput\nsus\njos\nstinga\ndreapta\nsfirsit").unwrap();
        assert_eq!(
            p.code,
            vec![
                Instr::Act(Dir::Up),
                Instr::Act(Dir::Down),
                Instr::Act(Dir::Left),
                Instr::Act(Dir::Right),
                Instr::Halt
            ]
        );
        assert_eq!(&p.lines[..4], &[2, 3, 4, 5]);
    }

    #[test]
    fn all_languages() {
        let codes: Vec<Vec<Instr>> = [
            "procedura p\ncit nu_e_margine executa\njos\nsfirsit cit\nsfirsit procedura\n\
             inceput\nrepeta 2 ori\ndreapta\nsfirsit repeta\np\n\
             daca e_margine atunci sus altfel jos\nsfirsit",
            "PROCEDURE p\nWHILE NOT_BORDER DO\nDOWN\nEND WHILE\nEND PROCEDURE\n\
             BEGIN\nREPEAT 2 TIMES\nRIGHT\nEND REPEAT\np\n\
             IF BORDER THEN UP ELSE DOWN\nEND",
            "ПРОЦЕДУРА p\nПОКА НЕ_КРАЙ CДЕЛАЙ\nВНИЗ\nКОНЕЦ ПОКА\nКОНЕЦ ПРОЦЕДУРА\n\
             НАЧАЛО\nПОВТОРИ 2 РАЗ\nВПРАВО\nКОНЕЦ ПОВТОРИ\np\n\
             ЕСЛИ КРАЙ ТОГДА ВВЕРХ ИНАЧЕ ВНИЗ\nКОНЕЦ",
            "PROCEDURA p\nCÎT NU E_MARGINE EXECUTĂ\nJOS\nSFÎRȘIT CÎT\nSFÎRȘIT PROCEDURA\n\
             ÎNCEPUT\nREPETĂ 2 ORI\nDREAPTA\nSFÎRȘIT REPETĂ\np\n\
             DACĂ E_MARGINE ATUNCI SUS ALTFEL JOS\nSFÎRȘIT",
        ]
        .iter()
        .map(|src| compile(src).unwrap_or_else(|e| panic!("{src}: {e:?}")).code)
        .collect();
        assert!(codes.iter().all(|c| *c == codes[0]), "{codes:?}");
        assert!(
            codes[0]
                .iter()
                .any(|i| matches!(i, Instr::JumpUnless(Cond { negated: true }, _)))
        );
    }

    #[test]
    fn nesting_is_allowed() {
        // În original REPETĂ nu putea conține alt REPETĂ.
        assert!(
            compile(
                "inceput\nrepeta 2 ori\nrepeta 3 ori\nsus\nsfirsit repeta\nsfirsit repeta\nsfirsit"
            )
            .is_ok()
        );
    }

    #[test]
    fn errors_have_lines() {
        let e = compile("inceput\nsus\nzbor\nsfirsit").unwrap_err();
        assert_eq!(e.line, 3);
        assert_eq!(e.kind, ErrorKind::UnknownWord("zbor".into()));

        let e = compile("inceput\nrepeta 3 ori\nsus\nsfirsit cit\nsfirsit").unwrap_err();
        assert_eq!(
            e.kind,
            ErrorKind::WrongEnd {
                expected: BlockKind::Repeat
            }
        );

        let e = compile("inceput\nrepeta 3 ori\nsus\nsfirsit").unwrap_err();
        assert_eq!(
            (e.line, e.kind),
            (2, ErrorKind::MissingEnd(BlockKind::Repeat))
        );
    }

    #[test]
    fn without_begin_end() {
        assert!(compile("sus jos stinga").is_ok());
    }

    #[test]
    fn execute_name_is_a_call() {
        assert!(
            compile("procedura p\nsus\nsfirsit procedura\ninceput\nexecuta p\nsfirsit").is_ok()
        );
    }
}

/// Programe la întâmplare: analizorul nu trebuie să cadă și nici să se blocheze.
#[cfg(test)]
mod fuzz {
    use super::*;
    use crate::vm::{Vm, run_to_end};
    use crate::world::{Table, World};

    const WORDS: &[&str] = &[
        "INCEPUT",
        "SFIRSIT",
        "SUS",
        "JOS",
        "STINGA",
        "DREAPTA",
        "REPETA",
        "3",
        "0",
        "ORI",
        "CIT",
        "NU_E_MARGINE",
        "E_MARGINE",
        "NU",
        "EXECUTA",
        "DACA",
        "ATUNCI",
        "ALTFEL",
        "PROCEDURA",
        "p",
        "q",
        "CIT",
        "REPETA",
        "PROCEDURA",
        "x",
        "\n",
        "\n",
        "\n",
        "//",
        "*",
        "БУКВА",
    ];

    #[test]
    fn random_programs_do_not_panic() {
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut compiled = 0;
        for _ in 0..3000 {
            let n = next() % 40;
            let src: Vec<&str> = (0..n)
                .map(|_| WORDS[(next() % WORDS.len() as u64) as usize])
                .collect();
            let src = src.join(" ");
            let a = analyze(&src);
            assert_eq!(a.program.is_some(), a.errors.is_empty(), "{src}");
            for e in &a.errors {
                assert!(src.get(e.span.clone()).is_some(), "{src}: {e:?}");
            }
            // Formatarea nu schimbă programul și a doua oară nu mai are ce face.
            let formatted = format_program(&src);
            assert_eq!(format_program(&formatted), formatted, "{src}");
            if let Some(p) = &a.program {
                let after = analyze(&formatted).program;
                assert_eq!(after.map(|q| q.code), Some(p.code.clone()), "{src}");
            }
            if let Some(p) = a.program {
                compiled += 1;
                let mut world = World::new(Table::default());
                run_to_end(&mut Vm::new(p), &mut world, 500);
            }
        }
        assert!(compiled > 50, "prea puține programe corecte: {compiled}");
    }
}
