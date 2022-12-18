use board::{Board, Color};
use montecarlo::Tree;
use std::time::Instant;

mod board;
mod minimax;
mod montecarlo;

fn main() {
    let mut board = Board::new(Color::Black);
    board.reset();
    let mut tree = Tree::new(board);

    let now = Instant::now();
    println!("best move {:?}", tree.get_monte_carlo_move());
    let elapsed = now.elapsed();
    println!("took: {:.2?}", elapsed)

    // use indextree::Arena;
    // let mut arena = Arena::new();
    // let n1 = arena.new_node("1");
    // let n1_1 = arena.new_node("1_1");
    // n1.append(n1_1, &mut arena);
    // let n1_1_1 = arena.new_node("1_1_1");
    // n1_1.append(n1_1_1, &mut arena);
    // let n1_2 = arena.new_node("1_2");
    // n1.append(n1_2, &mut arena);
    // let n1_3 = arena.new_node("1_3");
    // n1.append(n1_3, &mut arena);

    // let n1_3_1 = arena.new_node("1_3_1");
    // n1_3.append(n1_3_1, &mut arena);

    // let mut iter = n1.children(&arena);
    // assert_eq!(iter.next(), Some(n1_1));
    // assert_eq!(iter.next(), Some(n1_2));
    // assert_eq!(iter.next(), Some(n1_3));
    // assert_eq!(iter.next(), None);
}
