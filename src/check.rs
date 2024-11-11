//! Analiza programului înainte de rulare: proceduri necunoscute, avertismente
//! despre cicluri care pot rula la nesfârșit și sugestii pentru cuvinte greșite.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use crate::lang::{Ast, COMMAND_WORDS, CompileError, ErrorKind, Stmt, StmtKind, normalize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WarningKind {
    UnusedProcedure(String),
    EmptyBlock,
    RepeatZero,
    /// CÎT fără nicio mișcare în corp: poate rula la nesfârșit.
    LoopWithoutMovement,
    /// Procedura se apelează pe ea însăși fără nicio condiție.
    EndlessRecursion(String),
    EmptyProgram,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
    pub line: usize,
    pub span: Range<usize>,
    pub kind: WarningKind,
}

/// Cuvinte pe care elevii le scriu des în loc de comenzi.
const SYNONYMS: &[(&str, &str)] = &[
    ("START", "INCEPUT"),
    ("INCEPE", "INCEPUT"),
    ("STOP", "SFIRSIT"),
    ("FINAL", "SFIRSIT"),
    ("STINGHA", "STINGA"),
    ("DREAPA", "DREAPTA"),
    ("DREPTA", "DREAPTA"),
    ("REPETAM", "REPETA"),
    ("ATUNCEA", "ATUNCI"),
];

/// Distanța de editare; două litere inversate („CTI” în loc de „CIT”) contează ca o greșeală.
fn distance(a: &[char], b: &[char]) -> usize {
    let mut d = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    d[0] = (0..=b.len()).collect();
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j - 1] + cost)
                .min(d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

/// Scrie sugestia cu aceleași litere (mici / mari) ca originalul.
fn match_case(original: &str, word: &str) -> String {
    let mut chars = original.chars().filter(|c| c.is_alphabetic());
    let first_upper = chars.next().is_some_and(char::is_uppercase);
    let rest_lower = chars.all(|c| !c.is_uppercase());
    match (first_upper, rest_lower) {
        (false, _) => word.to_lowercase(),
        (true, true) => {
            let lower = word.to_lowercase();
            let mut c = lower.chars();
            c.next()
                .map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
        }
        (true, false) => word.to_string(),
    }
}

/// Cel mai apropiat cuvânt din listă de un cuvânt scris greșit.
pub fn suggest(normalized: &str, original: &str, candidates: &[&str]) -> Option<String> {
    if let Some((_, to)) = SYNONYMS.iter().find(|(from, _)| *from == normalized)
        && candidates.contains(to)
    {
        return Some(match_case(original, to));
    }
    let w: Vec<char> = normalized.chars().collect();
    let max = match w.len() {
        0..=4 => 1,
        5..=8 => 2,
        _ => 3,
    };
    candidates
        .iter()
        .map(|c| (distance(&w, &c.chars().collect::<Vec<_>>()), *c))
        .filter(|&(d, _)| d > 0 && d <= max)
        .min_by_key(|&(d, _)| d)
        .map(|(_, c)| match_case(original, c))
}

/// Comenzile din corp mută furnica (sau apelează o procedură)?
fn moves(body: &[Stmt]) -> bool {
    body.iter().any(|s| match &s.kind {
        StmtKind::Move(_) | StmtKind::Call(..) | StmtKind::Invalid => true,
        StmtKind::Repeat { body, .. } | StmtKind::While(_, body) => moves(body),
        StmtKind::If(_, a, b) => {
            moves(std::slice::from_ref(a))
                || b.as_deref().is_some_and(|b| moves(std::slice::from_ref(b)))
        }
    })
}

/// Apelurile care se fac sigur, fără nicio condiție (pentru recursivitate infinită).
fn unconditional_calls<'a>(body: &'a [Stmt], out: &mut Vec<&'a str>) {
    for s in body {
        match &s.kind {
            StmtKind::Call(name, ..) => out.push(name),
            StmtKind::Repeat { n, body, .. } if *n > 0 => unconditional_calls(body, out),
            _ => {}
        }
    }
}

struct Checker<'a> {
    procs: HashMap<&'a str, usize>,
    proc_texts: Vec<&'a str>,
    errors: &'a mut Vec<CompileError>,
    warnings: Vec<Warning>,
    calls: Vec<HashSet<usize>>,
}

impl<'a> Checker<'a> {
    fn warn(&mut self, line: usize, span: &Range<usize>, kind: WarningKind) {
        self.warnings.push(Warning {
            line,
            span: span.clone(),
            kind,
        });
    }

    /// Sugestia pentru un cuvânt necunoscut: o comandă sau numele unei proceduri.
    fn hint(&self, name: &str, text: &str) -> Option<String> {
        let names: Vec<String> = self.proc_texts.iter().map(|t| normalize(t)).collect();
        let mut candidates: Vec<&str> = names.iter().map(String::as_str).collect();
        candidates.extend(COMMAND_WORDS);
        let hint = suggest(name, text, &candidates)?;
        // Numele procedurii se propune exact cum a fost scris la definiție.
        Some(
            self.proc_texts
                .iter()
                .find(|t| normalize(t) == normalize(&hint))
                .map_or(hint, |t| t.to_string()),
        )
    }

    /// Verifică un corp; `owner` este procedura care îl conține (`procs.len()` = programul principal).
    fn body(&mut self, body: &'a [Stmt], owner: usize) {
        for s in body {
            match &s.kind {
                StmtKind::Call(name, text, span) => match self.procs.get(name.as_str()) {
                    Some(&i) => {
                        self.calls[owner].insert(i);
                    }
                    None => {
                        let hint = self.hint(name, text);
                        self.errors.push(CompileError {
                            line: s.line,
                            span: span.clone(),
                            kind: ErrorKind::UnknownWord(text.clone()),
                            hint,
                        });
                    }
                },
                StmtKind::Repeat { n, n_span, body } => {
                    if *n == 0 {
                        self.warn(s.line, n_span, WarningKind::RepeatZero);
                    } else if body.is_empty() {
                        self.warn(s.line, &s.span, WarningKind::EmptyBlock);
                    }
                    self.body(body, owner);
                }
                StmtKind::While(_, body) => {
                    if !moves(body) {
                        self.warn(s.line, &s.span, WarningKind::LoopWithoutMovement);
                    }
                    self.body(body, owner);
                }
                StmtKind::If(_, a, b) => {
                    self.body(std::slice::from_ref(a), owner);
                    if let Some(b) = b {
                        self.body(std::slice::from_ref(b), owner);
                    }
                }
                StmtKind::Move(_) | StmtKind::Invalid => {}
            }
        }
    }
}

/// Analiza semantică; adaugă erorile în `errors` și întoarce avertismentele.
pub(crate) fn check(ast: &Ast, errors: &mut Vec<CompileError>) -> Vec<Warning> {
    let n = ast.procs.len();
    let main = n; // indexul „programului principal” în graful de apeluri
    let mut c = Checker {
        procs: ast
            .procs
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.as_str(), i))
            .collect(),
        proc_texts: ast.procs.iter().map(|p| p.text.as_str()).collect(),
        errors,
        warnings: Vec::new(),
        calls: vec![HashSet::new(); n + 1],
    };

    if ast.main.is_empty() {
        let (line, span) = ast.main_at.clone().unwrap_or((1, 0..0));
        c.warn(line, &span, WarningKind::EmptyProgram);
    }
    c.body(&ast.main, main);
    for (i, p) in ast.procs.iter().enumerate() {
        if p.body.is_empty() {
            c.warn(p.line, &p.span, WarningKind::EmptyBlock);
        }
        c.body(&p.body, i);
    }

    // Procedurile la care nu se ajunge din programul principal.
    let mut used = HashSet::new();
    let mut stack = vec![main];
    while let Some(i) = stack.pop() {
        for &j in &c.calls[i] {
            if used.insert(j) {
                stack.push(j);
            }
        }
    }
    for (i, p) in ast.procs.iter().enumerate() {
        if !used.contains(&i) {
            c.warn(
                p.line,
                &p.span,
                WarningKind::UnusedProcedure(p.text.clone()),
            );
        }
    }

    // Recursivitate fără oprire: un ciclu din apeluri făcute fără condiție.
    let sure: Vec<Vec<usize>> = ast
        .procs
        .iter()
        .map(|p| {
            let mut names = Vec::new();
            unconditional_calls(&p.body, &mut names);
            names
                .iter()
                .filter_map(|n| c.procs.get(n).copied())
                .collect()
        })
        .collect();
    for (i, p) in ast.procs.iter().enumerate() {
        let mut seen = HashSet::new();
        let mut stack = sure[i].clone();
        let mut cycle = false;
        while let Some(j) = stack.pop() {
            if j == i {
                cycle = true;
                break;
            }
            if seen.insert(j) {
                stack.extend(&sure[j]);
            }
        }
        if cycle {
            c.warn(
                p.line,
                &p.span,
                WarningKind::EndlessRecursion(p.text.clone()),
            );
        }
    }

    let mut warnings = c.warnings;
    warnings.sort_by_key(|w| (w.span.start, w.line));
    warnings
}

#[cfg(test)]
mod tests {
    use crate::lang::{BlockKind, ErrorKind, analyze};

    use super::*;

    fn kinds(src: &str) -> Vec<(usize, ErrorKind)> {
        analyze(src)
            .errors
            .into_iter()
            .map(|e| (e.line, e.kind))
            .collect()
    }

    #[test]
    fn reports_all_errors_not_just_first() {
        let src = "PROCEDURA PinaJos\nCÎT NU_E_MARGINE EXECUTĂ\n    JOS\nSFÎRȘIT CÎT\nSFÎRȘIT PROCEDURA\n\
                   ÎNCEPUT\nSUS\nSTINAG\nREPETĂ de 3 ORI\n    DREAPTA\nSFÎRȘIT REPETĂ\nCÎT NU E_MARGNE\n    JOS\nSFÎRȘIT CÎT\nPinaJso\nSFÎRȘIT\n";
        assert_eq!(
            kinds(src),
            vec![
                (8, ErrorKind::UnknownWord("STINAG".into())),
                (9, ErrorKind::ExpectedNumber),
                (12, ErrorKind::UnknownCondition("E_MARGNE".into())),
                (15, ErrorKind::UnknownWord("PinaJso".into())),
            ]
        );
        let a = analyze(src);
        assert_eq!(a.errors[0].hint.as_deref(), Some("STINGA"));
        assert_eq!(a.errors[2].hint.as_deref(), Some("E_MARGINE"));
        assert_eq!(a.errors[3].hint.as_deref(), Some("PinaJos"));
        assert_eq!(&src[a.errors[0].span.clone()], "STINAG");
        assert!(a.program.is_none());
    }

    #[test]
    fn hints_keep_case() {
        assert_eq!(
            suggest("DREPTA", "drepta", COMMAND_WORDS).as_deref(),
            Some("dreapta")
        );
        assert_eq!(
            suggest("REPTA", "Repta", COMMAND_WORDS).as_deref(),
            Some("Repeta")
        );
        assert_eq!(suggest("ZZZZZZ", "zzzzzz", COMMAND_WORDS), None);
    }

    #[test]
    fn misspelled_end_kind() {
        let a = analyze("ÎNCEPUT\nCÎT NU_E_MARGINE\nJOS\nSFÎRȘIT CTI\nSFÎRȘIT\n");
        let e: Vec<_> = a.errors.iter().map(|e| (e.line, e.kind.clone())).collect();
        assert_eq!(e, vec![(4, ErrorKind::UnknownWord("CTI".into()))]);
        assert_eq!(a.errors[0].hint.as_deref(), Some("CIT"));
    }

    #[test]
    fn block_structure_errors() {
        // Lipsește sfârșitul repetării: eroarea e pe rândul cu REPETĂ.
        assert_eq!(
            kinds("ÎNCEPUT\nREPETĂ 2 ORI\nSUS\nSFÎRȘIT"),
            vec![(2, ErrorKind::MissingEnd(BlockKind::Repeat))]
        );
        assert_eq!(
            kinds("ÎNCEPUT\nSUS\nSFÎRȘIT CÎT\nSFÎRȘIT"),
            vec![(3, ErrorKind::StrayEnd)]
        );
        assert_eq!(
            kinds("ÎNCEPUT\nDACĂ E_MARGINE ATUNCI SUS JOS ALTFEL JOS\nSFÎRȘIT"),
            vec![(2, ErrorKind::StrayElse)]
        );
        assert_eq!(
            kinds("ÎNCEPUT\nSUS\n"),
            vec![(1, ErrorKind::MissingMainEnd)]
        );
        assert_eq!(
            kinds("ÎNCEPUT\nSUS\nSFÎRȘIT\nJOS"),
            vec![(4, ErrorKind::TextAfterProgram)]
        );
        assert_eq!(
            kinds("ÎNCEPUT\nSUS\nPROCEDURA A\nJOS\nSFÎRȘIT PROCEDURA\nSFÎRȘIT"),
            vec![(3, ErrorKind::ProcedureInMain)]
        );
        assert_eq!(
            kinds("PROCEDURA A\nJOS\nÎNCEPUT\nA\nSFÎRȘIT"),
            vec![(1, ErrorKind::MissingEnd(BlockKind::Procedure))]
        );
        // Procedura închisă cu SFÎRȘIT simplu.
        assert_eq!(
            kinds("PROCEDURA A\nJOS\nSFÎRȘIT\nÎNCEPUT\nA\nSFÎRȘIT"),
            vec![(
                3,
                ErrorKind::WrongEnd {
                    expected: BlockKind::Procedure
                }
            )]
        );
        assert_eq!(
            kinds("ÎNCEPUT\nDACĂ E_MARGINE ATUNCI\nSFÎRȘIT"),
            vec![(3, ErrorKind::ExpectedCommand)]
        );
        assert_eq!(kinds("SUS\nSFÎRȘIT"), vec![(2, ErrorKind::MissingBegin)]);
        assert_eq!(
            kinds("SUS\nÎNCEPUT\nJOS\nSFÎRȘIT"),
            vec![(1, ErrorKind::CommandOutsideMain)]
        );
        assert_eq!(
            kinds("ÎNCEPUT\nSUS\nÎNCEPUT\nJOS\nSFÎRȘIT"),
            vec![(3, ErrorKind::DuplicateBegin)]
        );
    }

    #[test]
    fn warnings() {
        let a = analyze(
            "PROCEDURA Nefolosita\nSUS\nSFÎRȘIT PROCEDURA\n\
             PROCEDURA R\nJOS\nR\nSFÎRȘIT PROCEDURA\n\
             ÎNCEPUT\nREPETĂ 0 ORI\nSUS\nSFÎRȘIT REPETĂ\nCÎT NU_E_MARGINE EXECUTĂ\nSFÎRȘIT CÎT\nR\nSFÎRȘIT",
        );
        assert!(a.errors.is_empty(), "{:?}", a.errors);
        assert!(a.program.is_some(), "avertismentele nu opresc rularea");
        let w: Vec<_> = a.warnings.into_iter().map(|w| (w.line, w.kind)).collect();
        assert_eq!(
            w,
            vec![
                (1, WarningKind::UnusedProcedure("Nefolosita".into())),
                (4, WarningKind::EndlessRecursion("R".into())),
                (9, WarningKind::RepeatZero),
                (12, WarningKind::LoopWithoutMovement),
            ]
        );
    }

    #[test]
    fn original_examples_are_clean() {
        for f in ["conferinta.ant", "svi99.ant"] {
            let text =
                std::fs::read_to_string(format!("{}/examples/{f}", env!("CARGO_MANIFEST_DIR")))
                    .unwrap();
            let file = crate::antfile::parse(&text);
            let a = analyze(&file.code);
            assert!(
                a.errors.is_empty() && a.warnings.is_empty(),
                "{f}: {:?} {:?}",
                a.errors,
                a.warnings
            );
        }
    }
}
