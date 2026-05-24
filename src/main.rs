mod app;
mod input;
mod life;
mod render;
mod screen;

use life::Board;
use render::render_frame;

fn main() {
    let mut board = Board::new(8, 8);

    //glider
    board.set(1, 0, true);
    board.set(2, 1, true);
    board.set(0, 2, true);
    board.set(1, 2, true);
    board.set(2, 2, true);

    println!("{}", render_frame(&board));

    board.step();

    println!("After step:\n");
    println!("{}", render_frame(&board));
}
