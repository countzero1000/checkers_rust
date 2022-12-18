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

const STATIC_SIZE: usize = 25;

const KING_MOVES: [(i32, i32); 4] = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
const BLACK_MOVES: [(i32, i32); 2] = [(1, 1), (-1, 1)];
const RED_MOVES: [(i32, i32); 2] = [(1, -1), (-1, -1)];

pub struct StaticList<T> {
    mem: [Option<T>; STATIC_SIZE],
    len: usize,
}

pub struct MoveMemHandler {
    captures: StaticList<Action>,
    moves: StaticList<Action>,
}

// Custom allocator to handle the complexity of determing valid moves
// and avoid constantly reallocating memory when retrieving the moves for a board
impl MoveMemHandler {
    pub fn new() -> Self {
        Self {
            captures: StaticList::new(),
            moves: StaticList::new(),
        }
    }

    pub fn add_capture(&mut self, action: Action) {
        self.captures.push(action)
    }

    pub fn add_move(&mut self, action: Action) {
        if !self.contains_capture() {
            self.moves.push(action)
        }
    }

    pub fn add_action(&mut self, action: Action) {
        match action {
            Action::Capture(_, _, _, _, _, _) => self.add_capture(action),
            Action::Move(_, _, _, _) => self.add_move(action),
        }
    }

    pub fn get(&self, index: usize) -> Action {
        if self.contains_capture() {
            return self.captures.get(index);
        }
        return self.moves.get(index);
    }

    pub fn clear(&mut self) {
        self.moves.clear();
        self.captures.clear();
    }

    pub fn contains_capture(&self) -> bool {
        return self.captures.len() > 0;
    }

    pub fn has_actions(&self) -> bool {
        return self.captures.len() > 0 || self.moves.len() > 0;
    }

    pub fn len(&self) -> usize {
        if self.contains_capture() {
            return self.captures.len();
        }
        return self.moves.len();
    }

    pub fn get_random_move(&self) -> Action {
        if self.contains_capture() {
            let index = rand::thread_rng().gen_range(0..self.captures.len());
            return self.captures.get(index);
        } else if self.moves.len() > 0 {
            let index = rand::thread_rng().gen_range(0..self.moves.len());
            return self.moves.get(index);
        }
        panic!("Called get random moves with no moves available")
    }
}

impl<T> StaticList<T>
where
    Option<T>: Copy,
{
    pub fn new() -> Self {
        Self {
            mem: [None; STATIC_SIZE],
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.len >= STATIC_SIZE {
            println!("Too many items {}", self.len + 1);
            panic!()
        }
        self.mem[self.len] = Some(item);
        self.len += 1
    }

    pub fn len(&self) -> usize {
        return self.len;
    }

    pub fn clear(&mut self) {
        self.len = 0;
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

enum MoveType {
    King([(i32, i32); 4]),
    Normal([(i32, i32); 2]),
    Empty,
}

impl Piece {
    fn get_dirs(&self) -> MoveType {
        match self {
            Piece::Filled(color, king) => match king {
                true => MoveType::King(KING_MOVES),
                false => match color {
                    Color::Red => MoveType::Normal(RED_MOVES),
                    Color::Black => MoveType::Normal(BLACK_MOVES),
                },
            },
            Piece::Empty => MoveType::Empty,
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

    fn get_action(&self, x: usize, y: usize, xd: i32, yd: i32, piece: Piece) -> Option<Action> {
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
                                return Some(Action::Move(x, y, cx, cy));
                            }
                            Piece::Filled(cap_color, _) => {
                                return if cap_color != self_color {
                                    match move_to {
                                        Some(move_to) => match move_to {
                                            Piece::Filled(_, _) => None,
                                            Piece::Empty => {
                                                Some(Action::Capture(x, y, mx, my, cx, cy))
                                            }
                                        },
                                        None => None,
                                    }
                                } else {
                                    None
                                }
                            }
                        }
                    }
                    None => None,
                }
            }
            Piece::Empty => None,
        }
    }

    fn extract_actions(
        &self,
        x: usize,
        y: usize,
        move_handler: &mut MoveMemHandler,
        xd: i32,
        yd: i32,
        piece: Piece,
    ) {
        let action_maybe = self.get_action(x, y, xd, yd, piece);
        if let Some(action) = action_maybe {
            move_handler.add_action(action)
        }
    }

    fn get_actions(&self, x: usize, y: usize, move_handler: &mut MoveMemHandler) {
        let piece = self.get_piece(x, y).unwrap();
        let dirs = piece.get_dirs();

        match dirs {
            MoveType::King(king) => {
                for (xd, yd) in king {
                    self.extract_actions(x, y, move_handler, xd, yd, piece)
                }
            }
            MoveType::Normal(norm) => {
                for (xd, yd) in norm {
                    self.extract_actions(x, y, move_handler, xd, yd, piece)
                }
            }
            MoveType::Empty => {
                for (xd, yd) in [] {
                    self.extract_actions(x, y, move_handler, xd, yd, piece)
                }
            }
        };
    }

    pub fn execute_action(&mut self, action: Action) {
        match action {
            Action::Move(x, y, nx, ny) => {
                let piece = self.get_piece(x, y).unwrap().clone();
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
                let piece = self.get_piece(x, y).unwrap().clone();
                // println!("captured with a {}", piece);
                self.set_piece(x, y, Piece::Empty);
                self.set_piece(cx, cy, Piece::Empty);
                self.set_piece(nx, ny, piece);
                if ny == piece.king_y_con() {
                    self.king_piece(nx, ny)
                }

                if !self.piece_has_capture(nx, ny) {
                    self.last_turn = Some(self.current_turn);
                }
            }
        }
    }

    fn piece_has_capture(&self, x: usize, y: usize) -> bool {
        let piece = self.get_piece(x, y);
        if let Some(p) = piece {
            let dirs = p.get_dirs();
            match dirs {
                MoveType::King(king) => {
                    for (xd, yd) in king {
                        if let Some(action) = self.get_action(x, y, xd, yd, p) {
                            if let Action::Capture(..) = action {
                                return true;
                            }
                        }
                    }
                }
                MoveType::Normal(norm) => {
                    for (xd, yd) in norm {
                        if let Some(action) = self.get_action(x, y, xd, yd, p) {
                            if let Action::Capture(..) = action {
                                return true;
                            }
                        }
                    }
                }
                MoveType::Empty => return false,
            }
        }
        return false;
    }

    fn king_piece(&mut self, x: usize, y: usize) {
        let ptr = y * 8 + x;
        match self.internal_state[ptr] {
            Piece::Filled(color, _) => {
                self.internal_state[ptr] = Piece::Filled(color, true);
            }

            Piece::Empty => {
                println!("trying to king piece {} {}", y, x);
                panic!("tried to king empty piece")
            }
        }
    }

    // gets all actions and places them in the move_mem
    // leaves the move mem unclean
    pub fn get_all_actions(&self, move_mem: &mut MoveMemHandler) {
        move_mem.clear();
        for y in 0..8 {
            for x in 0..8 {
                let piece = self.get_piece(x, y).unwrap();
                match piece {
                    Piece::Filled(c, _) => {
                        if c == self.current_turn {
                            self.get_actions(x, y, move_mem);
                        }
                    }
                    Piece::Empty => {}
                }
            }
        }
    }

    // makes a move leaves the move_mem in an empty state
    pub fn make_random_move(&mut self, move_mem: &mut MoveMemHandler) -> Option<Color> {
        move_mem.clear();
        self.get_all_actions(move_mem);
        if !move_mem.has_actions() {
            return Some(self.current_turn.opposite());
        }
        let act = move_mem.get_random_move();
        self.execute_action(act);
        move_mem.clear();
        None
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
