use std::{
    fmt::{self, write},
    iter::Cloned,
    ops::ControlFlow,
    slice::SliceIndex,
};

use bit_vec::{BitVec, Blocks};
use rand::prelude::SliceRandom;
static BITS_PS: usize = 3;
pub struct Board {
    internal_state: BitVec,
}
#[derive(Clone, Copy)]
pub enum Piece {
    Filled(Color, bool),
    Empty,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Black,
    Red,
}
#[derive(Clone, Copy)]

pub enum Action {
    Move(usize, usize, usize, usize),
    Capture(usize, usize, usize, usize, usize, usize),
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::Filled(color, king) => {
                write!(
                    f,
                    "{}",
                    match &color {
                        Color::Black => {
                            match king {
                                true => "B",
                                false => "b",
                            }
                        }
                        Color::Red => {
                            match king {
                                true => "R",
                                false => "r",
                            }
                        }
                    }
                )
            }
            Piece::Empty => write!(f, "{}", "_"),
        }
    }
}

impl Piece {
    pub fn get_dirs(&self) -> Vec<(i32, i32)> {
        match self {
            Piece::Filled(color, king) => match king {
                true => vec![(1, 1), (-1, 1), (1, -1), (-1, -1)],
                false => match color {
                    Color::Red => vec![(1, -1), (-1, -1)],
                    Color::Black => vec![(1, 1), (-1, 1)],
                },
            },
            Piece::Empty => {
                vec![]
            }
        }
    }

    pub fn king_y_con(&self) -> usize {
        match self {
            &Piece::Filled(color, _) => match color {
                Color::Black => 7,
                Color::Red => 0,
            },
            &Piece::Empty => 4,
        }
    }
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            &Color::Black => Color::Red,
            &Color::Red => Color::Black,
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            internal_state: BitVec::from_elem(64 * BITS_PS, false),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            internal_state: self.internal_state.clone(),
        }
    }

    fn get_actions(&self, x: usize, y: usize) -> Vec<Action> {
        let piece = self.get_piece(x, y).unwrap();
        let dirs = piece.get_dirs();
        let mut moves = vec![];
        let mut caps = vec![];
        for (xd, yd) in dirs {
            let cx = (x as i32 + xd) as usize;
            let cy = (y as i32 + yd) as usize;
            let mx = (x as i32 + xd * 2) as usize;
            let my = (y as i32 + yd * 2) as usize;
            let capture = self.get_piece(cx, cy);
            let move_to = self.get_piece(mx, my);

            match piece {
                Piece::Filled(self_color, _) => {
                    match capture {
                        Some(cap_piece) => {
                            match cap_piece {
                                Piece::Empty => {
                                    //can move into close spot
                                    moves.push(Action::Move(x, y, cx, cy))
                                }
                                Piece::Filled(cap_color, _) => {
                                    if cap_color != self_color {
                                        match move_to {
                                            Some(move_to) => match move_to {
                                                Piece::Filled(color, _) => {}
                                                Piece::Empty => {
                                                    caps.push(Action::Capture(x, y, mx, my, cx, cy))
                                                }
                                            },
                                            None => {}
                                        }
                                    }
                                }
                            }
                        }
                        None => { /*out of range, so do nothing*/ }
                    }
                }
                Piece::Empty => { /*empty do nothing*/ }
            }
        }
        if caps.len() > 0 {
            return caps;
        }
        return moves;
    }

    pub fn execute_action(&mut self, action: Action) {
        match action {
            Action::Move(x, y, nx, ny) => {
                let mut piece = self.get_piece(x, y).unwrap().clone();
                println!("moved a {} at {}, {} to {}, {}", piece, x, y, nx, ny);
                self.set_piece(x, y, Piece::Empty);
                self.set_piece(nx, ny, piece);
                if ny == piece.king_y_con() {
                    self.king_piece(nx, ny)
                }
            }
            Action::Capture(x, y, nx, ny, cx, cy) => {
                let mut piece = self.get_piece(x, y).unwrap().clone();
                println!("captured with a {}", piece);
                self.set_piece(x, y, Piece::Empty);
                self.set_piece(cx, cy, Piece::Empty);
                self.set_piece(nx, ny, piece);
                if ny == piece.king_y_con() {
                    self.king_piece(nx, ny)
                }
            }
        }
    }

    fn king_piece(&mut self, x: usize, y: usize) {
        let ptr = (y * 8 + x) * BITS_PS;
        self.internal_state.set(ptr, true);
    }

    pub fn get_all_actions(&self, color: Color) -> Vec<Action> {
        let mut moves = vec![];
        let mut capts = vec![];
        for y in 0..8 {
            for x in 0..8 {
                let piece = self.get_piece(x, y).unwrap();
                match piece {
                    Piece::Filled(c, _) => {
                        if c == color {
                            let acts = self.get_actions(x, y);
                            for act in acts {
                                match act {
                                    Action::Capture(_, _, _, _, _, _) => capts.push(act),
                                    Action::Move(_, _, _, _) => {
                                        if capts.len() == 0 {
                                            moves.push(act)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Piece::Empty => {}
                }
            }
        }

        if capts.len() > 0 {
            return capts;
        }
        return moves;
    }

    pub fn make_random_move(&mut self, color: Color) {
        let acts = self.get_all_actions(color);
        let act = acts.choose(&mut rand::thread_rng()).unwrap().to_owned();
        self.execute_action(act);
    }

    pub fn reset(&mut self) {
        for y in 0..3 {
            for x in 0..8 {
                if (x + y) % 2 == 0 {
                    self.set_piece(x, y, Piece::Filled(Color::Black, false));
                }
            }
        }

        for y in 5..8 {
            for x in 0..8 {
                if (x + y) % 2 == 0 {
                    self.set_piece(x, y, Piece::Filled(Color::Red, false));
                }
            }
        }
    }

    pub fn get_piece(&self, x: usize, y: usize) -> Option<Piece> {
        if x >= 8 || y >= 8 {
            return None;
        }

        let ptr = (y * 8 + x) * BITS_PS;
        let king = self.internal_state.get(ptr).unwrap();
        let red = self.internal_state.get(ptr + 2).unwrap();
        let black = self.internal_state.get(ptr + 1).unwrap();
        match red {
            true => return Some(Piece::Filled(Color::Red, king)),
            false => match black {
                true => return Some(Piece::Filled(Color::Black, king)),
                false => return Some(Piece::Empty),
            },
        }
    }

    pub fn set_piece(&mut self, x: usize, y: usize, piece: Piece) {
        let ptr = (y * 8 + x) * BITS_PS;
        match piece {
            Piece::Filled(color, king) => {
                self.internal_state.set(ptr, king);
                self.internal_state.set(ptr + 1, color == Color::Black);
                self.internal_state.set(ptr + 2, color == Color::Red);
            }
            Piece::Empty => {
                self.internal_state.set(ptr, false);
                self.internal_state.set(ptr + 1, false);
                self.internal_state.set(ptr + 2, false)
            }
        }
    }

    pub fn print_board(&self) {
        print!("{}", "-----------------\n");
        for y in 0..8 {
            for x in 0..8 {
                print!("|{}", self.get_piece(x, y).unwrap())
            }
            print!("{}", "|\n-----------------\n")
        }
    }
}
