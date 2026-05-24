// terminal drawing

use crate::life::Board;
// importing board

pub fn render_frame(board: &Board) -> String // read only borrow
{
    let width = board.width();
    let height = board.height();
    let rows = height.div_ceil(2);

    // pre allocate output buffer
    let capacity = width * rows + 2 * rows;
    let mut output = String::with_capacity(capacity);

    // header
    // output += "TIDE : [ACTIVE]\n\n";

    // now to convert ur rows to single string
    // step by 2 since half-block rendering packs 2 rows into 1 char
    for y in (0..height).step_by(2) {
        // check once per row pair

        //out of loop since its fixed in a loop
        let has_bottom = y + 1 < height;

        for x in 0..width {
            let top = board.is_alive(x, y);

            // on odd height boards last bottom row may not exist
            // if bottom row doesnt exist treat as dead
            let bottom = if has_bottom {
                board.is_alive(x, y + 1)
            } else {
                false
            };

            // state based charecter add
            let cell_char = match (top, bottom) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };
            output.push(cell_char);
        }

        // newline after packed row, except for the last line to prevent scrolling
        if y + 2 < height {
            output.push_str("\r\n");
        }
    }

    output
}
