use std::f32::consts::PI;

use board::{Color, Piece};

mod board;
mod minimax;
mod montecarlo;

fn main() {
    let mut board = board::Board::new();
    board.reset();
    board.print_board();

    // board.set_piece(2, 2, Piece::Filled(Color::Red, true));

    // board.print_board();

    let mut cur_color = Color::Red;

    while true {
        board.make_random_move(cur_color);
        board.print_board();
        cur_color = cur_color.opposite();
    }
}
