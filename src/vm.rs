//! Mașina virtuală care execută un `Program` câte o mișcare pe rând,
//! pentru a putea anima furnica și a evidenția linia curentă.

use crate::lang::{Instr, Program};
use crate::world::{Move, World, WorldError};

/// Adâncimea maximă a apelurilor de proceduri.
pub const MAX_CALL_DEPTH: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    World(WorldError),
    StackOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tick {
    /// S-a executat o mișcare pe linia dată.
    Moved(Move, usize),
    /// Bugetul de instrucțiuni s-a terminat fără nicio mișcare (ex. ciclu gol).
    Busy,
    Finished,
    Failed(RuntimeError, usize),
}

pub struct Vm {
    program: Program,
    pc: usize,
    calls: Vec<usize>,
    counts: Vec<u32>,
    done: bool,
}

impl Vm {
    pub fn new(program: Program) -> Vm {
        let pc = program.entry;
        Vm {
            program,
            pc,
            calls: Vec::new(),
            counts: Vec::new(),
            done: false,
        }
    }

    /// Execută instrucțiuni până la prima mișcare (sau până se termină bugetul).
    pub fn tick(&mut self, world: &mut World, budget: usize) -> Tick {
        if self.done {
            return Tick::Finished;
        }
        for _ in 0..budget {
            let line = self.program.lines[self.pc];
            match self.program.code[self.pc].clone() {
                Instr::Act(d) => match world.apply(d) {
                    Ok(m) => {
                        self.pc += 1;
                        return Tick::Moved(m, line);
                    }
                    Err(e) => return self.fail(RuntimeError::World(e), line),
                },
                Instr::PushCount(n) => {
                    self.counts.push(n);
                    self.pc += 1;
                }
                Instr::CountDown(end) => {
                    let top = self.counts.last_mut().expect("contor lipsă");
                    if *top == 0 {
                        self.counts.pop();
                        self.pc = end;
                    } else {
                        *top -= 1;
                        self.pc += 1;
                    }
                }
                Instr::Jump(t) => self.pc = t,
                Instr::JumpUnless(c, t) => {
                    self.pc = if world.check(c) { self.pc + 1 } else { t };
                }
                Instr::Call(t) => {
                    if self.calls.len() >= MAX_CALL_DEPTH {
                        return self.fail(RuntimeError::StackOverflow, line);
                    }
                    self.calls.push(self.pc + 1);
                    self.pc = t;
                }
                Instr::Return => self.pc = self.calls.pop().expect("return fără apel"),
                Instr::Halt => {
                    self.done = true;
                    return Tick::Finished;
                }
            }
        }
        Tick::Busy
    }

    fn fail(&mut self, e: RuntimeError, line: usize) -> Tick {
        self.done = true;
        Tick::Failed(e, line)
    }
}

/// Rulează un program până la capăt (folosit la teste).
#[cfg(test)]
pub fn run_to_end(vm: &mut Vm, world: &mut World, max_moves: usize) -> Tick {
    let mut moves = 0;
    loop {
        match vm.tick(world, 100_000) {
            Tick::Moved(..) => {
                moves += 1;
                if moves >= max_moves {
                    return Tick::Busy;
                }
            }
            t => return t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::compile;
    use crate::world::Table;

    fn run(src: &str, table: Table) -> (World, Tick) {
        let mut vm = Vm::new(compile(src).unwrap());
        let mut world = World::new(table);
        let t = run_to_end(&mut vm, &mut world, 1_000_000);
        (world, t)
    }

    fn example(name: &str) -> crate::antfile::AntFile {
        let text =
            std::fs::read_to_string(format!("{}/examples/{name}", env!("CARGO_MANIFEST_DIR")))
                .unwrap();
        crate::antfile::parse(&text)
    }

    #[test]
    fn goes_down_to_the_border() {
        let src = "inceput\ncit nu_e_margine executa\njos\nsfirsit cit\nsfirsit";
        let (w, t) = run(src, Table::new(10, 10));
        assert_eq!(t, Tick::Finished);
        assert_eq!(w.pos(), (0, 9));
        assert_eq!(w.step_count, 9);
    }

    #[test]
    fn right_border_is_detected() {
        // În original, E_MARGINE spre dreapta verifica marginea din stînga și furnica ieșea din tabelă.
        let src = "inceput\ndreapta\ncit nu_e_margine executa\ndreapta\nsfirsit cit\nsfirsit";
        let (w, t) = run(src, Table::new(10, 10));
        assert_eq!(t, Tick::Finished);
        assert_eq!(w.pos(), (9, 0));
    }

    #[test]
    fn errors_stop_on_their_line() {
        let (_, t) = run("inceput\nsus\nsfirsit", Table::new(5, 5));
        assert_eq!(
            t,
            Tick::Failed(RuntimeError::World(WorldError::OutOfField), 2)
        );
    }

    #[test]
    fn recursion_overflows() {
        let (_, t) = run(
            "procedura a\na\nsfirsit procedura\ninceput\na\nsfirsit",
            Table::new(5, 5),
        );
        assert!(matches!(t, Tick::Failed(RuntimeError::StackOverflow, _)));
    }

    #[test]
    fn conferinta_spells_the_word() {
        let f = example("conferinta.ant");
        let (w, t) = run(&f.code, f.table);
        assert_eq!(t, Tick::Finished);
        assert_eq!(w.table.row(0).trim(), "conferinta");
    }

    #[test]
    fn svi99_runs() {
        let f = example("svi99.ant");
        let (w, t) = run(&f.code, f.table);
        assert_eq!(t, Tick::Finished);
        assert_eq!(w.table.row(0).trim(), "svi99");
    }
}
