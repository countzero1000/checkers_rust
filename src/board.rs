use std::{
    fmt::{self, write},
    io::Empty,
    iter::Cloned,
    ops::ControlFlow,
    slice::SliceIndex,
};

use bit_vec::{BitVec, Blocks};
use rand::{prelude::SliceRandom, seq::index, Rng};
static BITS_PS: usize = 3;
#[derive(Clone)]
pub struct Board {
    internal_state: [Piece; 64],
    current_turn: Color,
    last_turn: Option<Color>,
}
#[derive(Clone, Copy)]
pub enum Piece {
    Filled(Color, bool),
    Empty,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Color {
    Black,
    Red,
}
#[derive(Clone, Copy, Debug)]

pub enum Action {
    Move(usize, usize, usize, usize),
    Capture(usize, usize, usize, usize, usize, usize),
}

const StaticSize: usize = 25;
pub struct StaticList<T> {
    mem: [Option<T>; StaticSize],
    len: usize,
}

impl<T> StaticList<T>
where
    Option<T>: Copy,
{
    pub fn new() -> Self {
        Self {
            mem: [None; StaticSize],
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.len >= StaticSize {
            println!("Too many items {}", self.len + 1);
            panic!()
        }
        self.mem[self.len] = Some(item);
        self.len += 1
    }

    pub fn len(&self) -> usize {
        return self.len;
    }

    pub fn get(&self, index: usize) -> T {
        return self.mem[index].unwrap();
    }
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
    pub fn get_current_color(&self) -> Color {
        return self.current_turn;
    }

    pub fn get_last_turn(&self) -> Option<Color> {
        return self.last_turn;
    }

    pub fn new(starting_color: Color) -> Self {
        Self {
            internal_state: [Piece::Empty; 64],
            current_turn: starting_color,
            last_turn: None,
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            internal_state: self.internal_state.clone(),
            current_turn: self.current_turn.clone(),
            last_turn: self.last_turn.clone(),
        }
    }

    fn get_actions(&self, x: usize, y: usize) -> StaticList<Action> {
        let piece = self.get_piece(x, y).unwrap();
        let dirs = piece.get_dirs();
        let mut moves = StaticList::new();
        let mut caps = StaticList::new();

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
                                                Piece::Filled(_, _) => {}
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
                // println!("moved a {} at {}, {} to {}, {}", piece, x, y, nx, ny);
                self.set_piece(x, y, Piece::Empty);
                self.set_piece(nx, ny, piece);
                if ny == piece.king_y_con() {
                    self.king_piece(nx, ny)
                }
                self.last_turn = Some(self.current_turn);
                self.current_turn = self.current_turn.opposite();
            }
            Action::Capture(x, y, nx, ny, cx, cy) => {
                let mut piece = self.get_piece(x, y).unwrap().clone();
                // println!("captured with a {}", piece);
                self.set_piece(x, y, Piece::Empty);
                self.set_piece(cx, cy, Piece::Empty);
                self.set_piece(nx, ny, piece);
                if ny == piece.king_y_con() {
                    self.king_piece(nx, ny)
                }
                self.last_turn = Some(self.current_turn);
                let moves = self.get_all_actions();
                if moves.len() > 0 {
                    match moves.get(0) {
                        Action::Move(_, _, _, _) => {
                            self.current_turn = self.current_turn.opposite()
                        }
                        Action::Capture(_, _, _, _, _, _) => {}
                    }
                }
            }
        }
    }

    fn king_piece(&mut self, x: usize, y: usize) {
        let ptr = y * 8 + x;
        match self.internal_state[ptr] {
            Piece::Filled(color, _) => {
                self.internal_state[ptr] = Piece::Filled(color, true);
            }
            Piece::Empty => panic!(),
        }
    }

    pub fn get_all_actions(&self) -> StaticList<Action> {
        let mut moves = StaticList::new();
        let mut capts = StaticList::new();
        for y in 0..8 {
            for x in 0..8 {
                let piece = self.get_piece(x, y).unwrap();
                match piece {
                    Piece::Filled(c, _) => {
                        if c == self.current_turn {
                            let acts = self.get_actions(x, y);
                            for i in 0..acts.len() {
                                let act = acts.get(i);
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

    pub fn make_random_move(&mut self) -> Option<Color> {
        let acts = self.get_all_actions();
        if acts.len() == 0 {
            return Some(self.current_turn.opposite());
        }
        let index = rand::thread_rng().gen_range(0..acts.len());
        let act = if acts.len() > 0 {
            Some(acts.get(index))
        } else {
            None
        };
        match act {
            Some(act) => {
                self.execute_action(act.to_owned());
                None
            }
            None => Some(self.current_turn.opposite()),
        }
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
        let ptr = y * 8 + x;
        return Some(self.internal_state[ptr]);
    }

    pub fn set_piece(&mut self, x: usize, y: usize, piece: Piece) {
        let ptr = y * 8 + x;
        self.internal_state[ptr] = piece;
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
