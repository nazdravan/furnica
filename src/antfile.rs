//! Fișierele `.ant`: tabela și programul, în formatul programului original.
//!
//! ```text
//! Ant executor file
//! [table description]
//! 1 10          poziția furnicii (coloana, rândul; de la 1)
//! 3             direcția: 1 jos, 2 stînga, 3 sus, 4 dreapta
//! 2 9 i         literele: coloana, rândul, litera
//! 1 10 *        celula furnicii este marcată cu *
//! 666 666 *     sfârșitul tabelei
//! inceput       programul, până la sfârșitul fișierului
//! ...
//! ```
//!
//! Tabelele de altă dimensiune decât 10 × 10 (care nu existau în original)
//! au în plus rândul `[size <coloane> <rânduri>]` înainte de `[table description]`.

use crate::world::{DEFAULT_SIZE, Dir, Table};

const MAGIC: &str = "Ant executor file";
const TABLE: &str = "[table description]";
const END: &str = "666 666 *";
/// Marcajul celulei furnicii.
const ANT: char = '*';

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AntFile {
    pub table: Table,
    pub code: String,
}

/// Citește un fișier `.ant`. Un text fără antet este doar program, cu tabela goală.
pub fn parse(text: &str) -> AntFile {
    let text = text.replace("\r\n", "\n");
    let mut lines = text.split('\n');
    if !lines
        .next()
        .is_some_and(|l| l.trim().eq_ignore_ascii_case(MAGIC))
    {
        return AntFile {
            table: Table::default(),
            code: text,
        };
    }

    let numbers =
        |l: &str| -> Vec<i64> { l.split_whitespace().map_while(|w| w.parse().ok()).collect() };
    let mut size = (DEFAULT_SIZE, DEFAULT_SIZE);
    let mut line = lines.next().unwrap_or("").trim();
    if let Some(rest) = line.strip_prefix("[size").and_then(|r| r.strip_suffix(']'))
        && let [w, h] = numbers(rest)[..]
    {
        size = (w as i32, h as i32);
        line = lines.next().unwrap_or("").trim();
    }
    let mut table = Table::new(size.0, size.1);
    // Rândul „[table description]” lipsă: îl considerăm deja citit.
    let mut pending = (!line.eq_ignore_ascii_case(TABLE)).then_some(line);
    let mut next = || pending.take().or_else(|| lines.next().map(str::trim));

    let ant = match numbers(next().unwrap_or(""))[..] {
        [x, y, ..] => (x as i32 - 1, y as i32 - 1),
        _ => (0, 0),
    };
    if table.inside(ant) {
        table.ant = ant;
    }
    if let Some(&d) = numbers(next().unwrap_or("")).first() {
        table.dir = u8::try_from(d)
            .ok()
            .and_then(Dir::from_code)
            .unwrap_or(Dir::Down);
    }
    let mut letters = Vec::new();
    while let Some(l) = next() {
        if l == END {
            break;
        }
        let mut parts = l.splitn(3, ' ').map(str::trim);
        let (Some(x), Some(y), Some(c)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let (Ok(x), Ok(y), Some(c)) = (x.parse::<i32>(), y.parse::<i32>(), c.chars().next()) else {
            continue;
        };
        letters.push(((x - 1, y - 1), c));
    }
    for (p, c) in letters {
        if !(p == table.ant && c == ANT) {
            table.set(p, Some(c));
        }
    }
    let code: Vec<&str> = lines.collect();
    AntFile {
        table,
        code: code.join("\n"),
    }
}

/// Scrie fișierul în formatul original (cu CRLF, ca să-l poată citi și versiunea veche).
pub fn write(table: &Table, code: &str) -> String {
    let mut out = vec![MAGIC.to_string()];
    if (table.width, table.height) != (DEFAULT_SIZE, DEFAULT_SIZE) {
        out.push(format!("[size {} {}]", table.width, table.height));
    }
    out.push(TABLE.into());
    out.push(format!("{} {}", table.ant.0 + 1, table.ant.1 + 1));
    out.push(table.dir.code().to_string());
    let ant = std::iter::once((table.ant, ANT));
    for ((x, y), c) in ant.chain(table.letters()) {
        out.push(format!("{} {} {c}", x + 1, y + 1));
    }
    out.push(END.into());
    let mut text = out.join("\r\n");
    text.push_str("\r\n");
    text.push_str(&code.replace('\n', "\r\n"));
    text
}

/// Fișierele vechi sunt în Windows-1250 (română) sau Windows-1251 (rusă).
pub fn decode(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cyr, _, _) = encoding_rs::WINDOWS_1251.decode(bytes);
            // În rusă cuvintele sunt scrise numai cu chirilice; un text românesc
            // decodat greșit ar amesteca litere latine cu chirilice în același cuvânt.
            let mixed = cyr.split(|c: char| !c.is_alphanumeric()).any(|w| {
                w.chars().any(|c| c.is_ascii_alphabetic())
                    && w.chars().any(|c| ('\u{400}'..='\u{4FF}').contains(&c))
            });
            if mixed {
                encoding_rs::WINDOWS_1250.decode(bytes).0.into_owned()
            } else {
                cyr.into_owned()
            }
        }
    };
    text.replace("\r\n", "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(name: &str) -> String {
        std::fs::read_to_string(format!("{}/examples/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    #[test]
    fn reads_original_file() {
        let f = parse(&example("conferinta.ant"));
        assert_eq!(f.table.ant, (0, 9));
        assert_eq!(f.table.dir, Dir::Up);
        assert_eq!(f.table.get((1, 6)), Some('n'));
        assert_eq!(f.table.row(8), " icotnrfea");
        assert_eq!(f.table.get((0, 9)), None, "* marchează doar furnica");
        assert_eq!(f.table.letters().count(), 10);
        assert!(f.code.starts_with("procedura pina_jos\n"));
        assert!(f.code.trim_end().ends_with("sfirsit"));
    }

    #[test]
    fn round_trip() {
        for name in ["conferinta.ant", "svi99.ant"] {
            let f = parse(&example(name));
            let again = parse(&write(&f.table, &f.code));
            assert_eq!(again, f, "{name}");
        }
        let mut t = Table::new(15, 7);
        t.set((14, 6), Some('Ж'));
        t.ant = (3, 2);
        t.dir = Dir::Left;
        let text = write(&t, "ÎNCEPUT\nSUS\nSFÎRȘIT");
        assert!(text.contains("[size 15 7]\r\n"));
        let f = parse(&text);
        assert_eq!(f.table, t);
        assert_eq!(f.code, "ÎNCEPUT\nSUS\nSFÎRȘIT");
    }

    #[test]
    fn written_file_looks_like_the_original() {
        let mut t = Table::default();
        t.set((1, 0), Some('a'));
        assert_eq!(
            write(&t, "inceput\nsfirsit"),
            "Ant executor file\r\n[table description]\r\n1 1\r\n1\r\n1 1 *\r\n2 1 a\r\n666 666 *\r\ninceput\r\nsfirsit"
        );
    }

    #[test]
    fn plain_text_is_a_program() {
        let f = parse("inceput\nsus\nsfirsit");
        assert_eq!(f.code, "inceput\nsus\nsfirsit");
        assert_eq!(f.table, Table::default());
    }

    #[test]
    fn decodes_legacy_encodings() {
        let rus = encoding_rs::WINDOWS_1251
            .encode("НАЧАЛО\r\nВВЕРХ\r\nКОНЕЦ")
            .0;
        assert_eq!(decode(&rus), "НАЧАЛО\nВВЕРХ\nКОНЕЦ");
        let rom = encoding_rs::WINDOWS_1250
            .encode("ÎNCEPUT\nSTÎNGA\nSFÎRŞIT")
            .0;
        assert_eq!(decode(&rom), "ÎNCEPUT\nSTÎNGA\nSFÎRŞIT");
        assert!(crate::lang::compile(&decode(&rom)).is_ok());
    }
}
