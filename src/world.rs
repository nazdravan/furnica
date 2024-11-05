//! Tabela: o grilă de celule cu litere, pe care se mișcă furnica.
//!
//! Regulile sunt cele din programul original „Furnica” (Ant v2.00): furnica
//! se mută cu o celulă și împinge litera din fața ei, dar nu poate împinge
//! două litere deodată și nici o literă peste marginea tabelei.

use std::collections::HashSet;

use crate::lang::Cond;

/// Direcția furnicii. Ordinea și codurile (1..4) sunt cele din fișierele `.ant`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Dir {
    Down,
    Left,
    Up,
    Right,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Down, Dir::Left, Dir::Up, Dir::Right];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Up => (0, -1),
            Dir::Right => (1, 0),
        }
    }

    /// Codul din fișierul `.ant`: 1 = jos, 2 = stînga, 3 = sus, 4 = dreapta.
    pub fn code(self) -> u8 {
        match self {
            Dir::Down => 1,
            Dir::Left => 2,
            Dir::Up => 3,
            Dir::Right => 4,
        }
    }

    pub fn from_code(c: u8) -> Option<Dir> {
        Dir::ALL.into_iter().find(|d| d.code() == c)
    }

    /// Rotire în sensul acelor ceasornicului (butonul „Rotire” al tabelei).
    pub fn clockwise(self) -> Dir {
        match self {
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Left,
            Dir::Left => Dir::Up,
            Dir::Up => Dir::Right,
        }
    }
}

/// Starea inițială a tabelei: dimensiunea, literele și furnica.
/// Coordonatele sunt de la 0; în fișiere și pe ecran se arată de la 1.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Table {
    pub width: i32,
    pub height: i32,
    /// Celulele, rând cu rând.
    cells: Vec<Option<char>>,
    pub ant: (i32, i32),
    pub dir: Dir,
}

pub const DEFAULT_SIZE: i32 = 10;
pub const MIN_SIZE: i32 = 3;
pub const MAX_SIZE: i32 = 20;

impl Default for Table {
    fn default() -> Self {
        Table::new(DEFAULT_SIZE, DEFAULT_SIZE)
    }
}

impl Table {
    /// Tabelă goală, cu furnica în colțul din stînga-sus privind în jos (ca în original).
    pub fn new(width: i32, height: i32) -> Table {
        let (width, height) = (
            width.clamp(MIN_SIZE, MAX_SIZE),
            height.clamp(MIN_SIZE, MAX_SIZE),
        );
        Table {
            width,
            height,
            cells: vec![None; (width * height) as usize],
            ant: (0, 0),
            dir: Dir::Down,
        }
    }

    /// Tabela citită din setări are dimensiunea, celulele și furnica la locul lor.
    pub fn is_consistent(&self) -> bool {
        (MIN_SIZE..=MAX_SIZE).contains(&self.width)
            && (MIN_SIZE..=MAX_SIZE).contains(&self.height)
            && self.cells.len() == (self.width * self.height) as usize
            && self.inside(self.ant)
            && self.get(self.ant).is_none()
    }

    pub fn inside(&self, (x, y): (i32, i32)) -> bool {
        (0..self.width).contains(&x) && (0..self.height).contains(&y)
    }

    fn index(&self, p: (i32, i32)) -> Option<usize> {
        self.inside(p).then(|| (p.1 * self.width + p.0) as usize)
    }

    pub fn get(&self, p: (i32, i32)) -> Option<char> {
        self.index(p).and_then(|i| self.cells[i])
    }

    /// Pune sau șterge o literă. Sub furnică nu se poate pune nimic.
    pub fn set(&mut self, p: (i32, i32), c: Option<char>) -> bool {
        match self.index(p) {
            Some(i) if p != self.ant || c.is_none() => {
                let changed = self.cells[i] != c;
                self.cells[i] = c;
                changed
            }
            _ => false,
        }
    }

    /// Mută furnica (de exemplu cu mausul) pe o celulă fără literă.
    pub fn place_ant(&mut self, p: (i32, i32)) -> bool {
        if !self.inside(p) || self.get(p).is_some() || p == self.ant {
            return false;
        }
        self.ant = p;
        true
    }

    /// Literele, cu pozițiile lor, rând cu rând.
    pub fn letters(&self) -> impl Iterator<Item = ((i32, i32), char)> + '_ {
        self.cells.iter().enumerate().filter_map(|(i, c)| {
            let i = i as i32;
            c.map(|c| ((i % self.width, i / self.width), c))
        })
    }

    pub fn clear_letters(&mut self) {
        self.cells.fill(None);
    }

    /// Altă dimensiune; literele care nu mai încap se pierd, furnica rămâne înăuntru.
    pub fn resized(&self, width: i32, height: i32) -> Table {
        let mut t = Table::new(width, height);
        for (p, c) in self.letters() {
            t.set(p, Some(c));
        }
        let ant = (self.ant.0.min(t.width - 1), self.ant.1.min(t.height - 1));
        t.set(ant, None);
        t.ant = ant;
        t.dir = self.dir;
        t
    }

    /// Rândul `y` ca text (celulele goale sunt spații); folosit la teste.
    #[cfg(test)]
    pub fn row(&self, y: i32) -> String {
        (0..self.width)
            .map(|x| self.get((x, y)).unwrap_or(' '))
            .collect()
    }
}

/// O mișcare executată (pentru animație).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub dir: Dir,
    /// Direcția dinainte (pentru rotirea lină a furnicii).
    pub dir_before: Dir,
    pub from: (i32, i32),
    pub to: (i32, i32),
    /// Litera împinsă și celula în care a ajuns.
    pub pushed: Option<(char, (i32, i32))>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    /// Furnica ar ieși din tabelă.
    OutOfField,
    /// Litera ar fi împinsă peste marginea tabelei.
    PushOut,
    /// În față sunt două litere una după alta.
    TooHeavy,
}

/// Tabela în timpul rulării: literele se mută, iar furnica lasă urme.
#[derive(Clone, Debug)]
pub struct World {
    pub table: Table,
    /// Celulele prin care a trecut furnica, în ordine (drumul ei).
    pub path: Vec<(i32, i32)>,
    visited: HashSet<(i32, i32)>,
    pub step_count: usize,
    pub pushes: usize,
}

impl World {
    pub fn new(table: Table) -> World {
        World {
            path: vec![table.ant],
            table,
            visited: HashSet::new(),
            step_count: 0,
            pushes: 0,
        }
    }

    pub fn pos(&self) -> (i32, i32) {
        self.table.ant
    }

    pub fn dir(&self) -> Dir {
        self.table.dir
    }

    /// Celula a fost părăsită de furnică cel puțin o dată (urma din original).
    pub fn visited(&self, p: (i32, i32)) -> bool {
        self.visited.contains(&p)
    }

    /// E_MARGINE: furnica privește (în direcția ultimei mișcări) spre marginea tabelei.
    /// Originalul greșea pentru stînga/dreapta (verifica marginea opusă); aici e corectat.
    pub fn check(&self, c: Cond) -> bool {
        let (dx, dy) = self.dir().delta();
        let (x, y) = self.pos();
        !self.table.inside((x + dx, y + dy)) != c.negated
    }

    pub fn apply(&mut self, dir: Dir) -> Result<Move, WorldError> {
        // Ca în original, furnica se întoarce spre direcția mișcării chiar dacă nu poate merge.
        let dir_before = self.table.dir;
        self.table.dir = dir;
        let (dx, dy) = dir.delta();
        let from = self.pos();
        let to = (from.0 + dx, from.1 + dy);
        if !self.table.inside(to) {
            return Err(WorldError::OutOfField);
        }
        let mut pushed = None;
        if let Some(c) = self.table.get(to) {
            let dest = (to.0 + dx, to.1 + dy);
            if !self.table.inside(dest) {
                return Err(WorldError::PushOut);
            }
            if self.table.get(dest).is_some() {
                return Err(WorldError::TooHeavy);
            }
            self.table.set(to, None);
            self.table.set(dest, Some(c));
            pushed = Some((c, dest));
            self.pushes += 1;
        }
        self.table.ant = to;
        self.visited.insert(from);
        self.path.push(to);
        self.step_count += 1;
        Ok(Move {
            dir,
            dir_before,
            from,
            to,
            pushed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(letters: &[((i32, i32), char)]) -> World {
        let mut t = Table::new(5, 5);
        for &(p, c) in letters {
            t.set(p, Some(c));
        }
        World::new(t)
    }

    #[test]
    fn pushes_one_letter() {
        let mut w = world(&[((1, 0), 'A')]);
        let m = w.apply(Dir::Right).unwrap();
        assert_eq!(m.pushed, Some(('A', (2, 0))));
        assert_eq!(w.pos(), (1, 0));
        assert_eq!(w.table.row(0), "  A  ");
        assert!(w.visited((0, 0)));
    }

    #[test]
    fn cannot_push_two_letters_or_over_the_edge() {
        let mut w = world(&[((1, 0), 'A'), ((2, 0), 'B')]);
        assert_eq!(w.apply(Dir::Right), Err(WorldError::TooHeavy));
        assert_eq!(w.pos(), (0, 0));
        assert_eq!(w.dir(), Dir::Right);

        let mut w = world(&[((0, 1), 'A')]);
        w.apply(Dir::Down).unwrap();
        w.apply(Dir::Down).unwrap();
        w.apply(Dir::Down).unwrap();
        assert_eq!(w.apply(Dir::Down), Err(WorldError::PushOut));
        assert_eq!(w.table.get((0, 4)), Some('A'));

        assert_eq!(w.apply(Dir::Left), Err(WorldError::OutOfField));
    }

    #[test]
    fn border_condition_uses_last_direction() {
        let border = Cond { negated: false };
        let mut w = world(&[]);
        assert!(!w.check(border), "privește în jos, spre interior");
        w.apply(Dir::Right).unwrap();
        w.apply(Dir::Left).unwrap();
        assert!(w.check(border));
        for _ in 0..4 {
            w.apply(Dir::Right).unwrap();
        }
        assert!(
            w.check(border),
            "la marginea din dreapta, privind spre dreapta"
        );
    }

    #[test]
    fn resize_keeps_what_fits() {
        let mut t = Table::new(10, 10);
        t.set((2, 2), Some('X'));
        t.set((8, 8), Some('Y'));
        t.ant = (9, 9);
        let r = t.resized(5, 5);
        assert_eq!(r.letters().collect::<Vec<_>>(), vec![((2, 2), 'X')]);
        assert_eq!(r.ant, (4, 4));
    }
}
