//! Editarea tabelei de pornire: furnica și literele se mută prin tragere,
//! iar literele se scriu de la tastatură, ca într-un editor de text.

use crate::world::Table;

/// Ce ține mausul în timpul tragerii.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grab {
    Ant,
    /// Litera și celula de unde a fost luată.
    Letter(char, (i32, i32)),
}

impl Grab {
    /// Ce se poate apuca din celula `p`.
    pub fn at(table: &Table, p: (i32, i32)) -> Option<Grab> {
        if p == table.ant {
            Some(Grab::Ant)
        } else {
            table.get(p).map(|c| Grab::Letter(c, p))
        }
    }

    /// Ce s-ar întâmpla la eliberarea deasupra celulei `target` (`None` = în afara tabelei).
    pub fn outcome(self, table: &Table, target: Option<(i32, i32)>) -> Drop {
        match (self, target) {
            (Grab::Ant, Some(t)) if t == table.ant => Drop::Nothing,
            (Grab::Ant, Some(t)) if table.get(t).is_none() => Drop::Move,
            (Grab::Ant, _) => Drop::Refused,
            (Grab::Letter(..), None) => Drop::Delete,
            (Grab::Letter(_, from), Some(t)) if t == from => Drop::Nothing,
            (Grab::Letter(..), Some(t)) if t == table.ant => Drop::Refused,
            (Grab::Letter(..), Some(t)) if table.get(t).is_some() => Drop::Swap,
            (Grab::Letter(..), Some(_)) => Drop::Move,
        }
    }

    /// Eliberează ce era tras; întoarce `true` dacă tabela s-a schimbat.
    pub fn drop(self, table: &mut Table, target: Option<(i32, i32)>) -> bool {
        let outcome = self.outcome(table, target);
        match (self, outcome, target) {
            (Grab::Ant, Drop::Move, Some(t)) => table.place_ant(t),
            (Grab::Letter(_, from), Drop::Delete, _) => table.set(from, None),
            (Grab::Letter(c, from), Drop::Move | Drop::Swap, Some(t)) => {
                let other = table.get(t);
                table.set(t, Some(c));
                table.set(from, other);
                true
            }
            _ => false,
        }
    }
}

/// Rezultatul eliberării, ca tabela să-l poată arăta dinainte (verde / roșu).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drop {
    Nothing,
    Move,
    /// Litera trece în locul celeilalte, iar aceea în locul ei.
    Swap,
    /// Litera trasă în afara tabelei se șterge.
    Delete,
    /// Furnica pe o literă sau în afara tabelei, ori o literă peste furnică.
    Refused,
}

/// Celula selectată, în care se scrie următoarea literă.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub cell: (i32, i32),
    /// Coloana la care se întoarce Enter.
    pub line_start: i32,
}

impl Cursor {
    pub fn at(cell: (i32, i32)) -> Cursor {
        Cursor {
            cell,
            line_start: cell.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeKey {
    Char(char),
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Enter,
}

fn moved(table: &Table, (x, y): (i32, i32), (dx, dy): (i32, i32)) -> (i32, i32) {
    (
        (x + dx).clamp(0, table.width - 1),
        (y + dy).clamp(0, table.height - 1),
    )
}

/// Următoarea celulă la scriere: spre dreapta, apoi pe rândul următor.
fn advance(table: &Table, cur: &mut Cursor) {
    let (x, y) = cur.cell;
    cur.cell = if x + 1 < table.width {
        (x + 1, y)
    } else if y + 1 < table.height {
        (cur.line_start.min(table.width - 1), y + 1)
    } else {
        (x, y)
    };
}

/// O tastă apăsată cât e selectată o celulă; întoarce `true` dacă tabela s-a schimbat.
pub fn type_key(table: &mut Table, cur: &mut Cursor, key: TypeKey) -> bool {
    match key {
        TypeKey::Char(c) => {
            // Peste furnică nu se scrie: litera ajunge în celula de după ea.
            if cur.cell == table.ant {
                advance(table, cur);
            }
            let letter = (!c.is_whitespace()).then_some(c);
            let changed = cur.cell != table.ant && table.set(cur.cell, letter);
            advance(table, cur);
            changed
        }
        TypeKey::Backspace => {
            cur.cell = moved(table, cur.cell, (-1, 0));
            table.set(cur.cell, None)
        }
        TypeKey::Delete => table.set(cur.cell, None),
        TypeKey::Left => {
            cur.cell = moved(table, cur.cell, (-1, 0));
            false
        }
        TypeKey::Right => {
            cur.cell = moved(table, cur.cell, (1, 0));
            false
        }
        TypeKey::Up => {
            cur.cell = moved(table, cur.cell, (0, -1));
            false
        }
        TypeKey::Down => {
            cur.cell = moved(table, cur.cell, (0, 1));
            false
        }
        TypeKey::Enter => {
            cur.cell = moved(table, (cur.line_start, cur.cell.1), (0, 1));
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(table: &mut Table, start: (i32, i32), keys: &str) -> Cursor {
        let mut cur = Cursor::at(start);
        for c in keys.chars() {
            let key = match c {
                '<' => TypeKey::Backspace,
                '\n' => TypeKey::Enter,
                c => TypeKey::Char(c),
            };
            type_key(table, &mut cur, key);
        }
        cur
    }

    #[test]
    fn typing_writes_words() {
        let mut t = Table::new(5, 3);
        let cur = typed(&mut t, (1, 0), "abX<c\nde");
        assert_eq!(t.row(0), " abc ");
        assert_eq!(t.row(1), " de  ");
        assert_eq!(cur.cell, (3, 1));
    }

    #[test]
    fn typing_wraps_and_skips_the_ant() {
        let mut t = Table::new(4, 3);
        t.ant = (2, 1);
        typed(&mut t, (2, 0), "abcd");
        assert_eq!(t.row(0), "  ab");
        assert_eq!(t.row(1), "   c", "furnica rămâne pe loc");
        assert_eq!(t.row(2), "  d ");
        // Spațiul șterge litera.
        typed(&mut t, (2, 0), " ");
        assert_eq!(t.row(0), "   b");
    }

    #[test]
    fn dragging() {
        let mut t = Table::new(5, 5);
        t.set((1, 1), Some('A'));
        t.set((2, 2), Some('B'));
        let a = Grab::at(&t, (1, 1)).unwrap();
        assert_eq!(a.outcome(&t, Some((0, 0))), Drop::Refused, "peste furnică");
        assert!(a.drop(&mut t, Some((2, 2))));
        assert_eq!((t.get((1, 1)), t.get((2, 2))), (Some('B'), Some('A')));
        assert!(Grab::at(&t, (2, 2)).unwrap().drop(&mut t, None));
        assert_eq!(t.get((2, 2)), None);

        let ant = Grab::at(&t, (0, 0)).unwrap();
        assert_eq!(ant, Grab::Ant);
        assert!(!ant.drop(&mut t, Some((1, 1))), "nu pe o literă");
        assert!(!ant.drop(&mut t, None));
        assert!(ant.drop(&mut t, Some((4, 3))));
        assert_eq!(t.ant, (4, 3));
        assert_eq!(Grab::at(&t, (3, 3)), None);
    }
}
