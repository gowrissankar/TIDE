//conways engine

//board API

// new()
// with_rules()
// step_ahead()
// is_alive()
// set()
// count_live_neighbors()
// width()
// height()

//1d array for faster reading
//indx = y *width + x

pub struct Rules {
    pub birth: Vec<u8>,
    pub survive: Vec<u8>,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            // B3/S23 by default
            birth: vec![3],
            survive: vec![2, 3],
        }
    }
}

pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<bool>,
    next: Vec<bool>,
    rules: Rules,
}

impl Board {
    //default initiator
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_rules(width, height, Rules::default())
        //returns self ir board
    }

    //custom rules fn
    pub fn with_rules(width: usize, height: usize, rules: Rules) -> Self {
        let size = width * height;

        Self {
            width,
            height,
            cells: vec![false; size], //curr board state
            next: vec![false; size],  //next board state , need curr asuch to gen new
            rules,
        }
    }

    //for other to read : encaps
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn is_alive(&self, x: usize, y: usize) -> bool //read only immutabel
    {
        self.cells[self.idx(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) //modify mutable
    {
        let idx = self.idx(x, y);
        self.cells[idx] = value;
    }

    pub fn count_live_neighbors(&self, x: usize, y: usize) -> u8 {
        //need to count nos of live neighs
        //the board is toroidal

        //need to handle edges properly and the 0-1 since unsigned ints

        let mut count = 0;

        let width = self.width as isize;
        let height = self.height as isize;

        let x = x as isize;
        let y = y as isize;

        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = ((x + dx + width) % width) as usize;

                let ny = ((y + dy + height) % height) as usize;

                if self.is_alive(nx, ny) {
                    count += 1;
                }
            }
        }

        count //no return statements bruhhh...
    }

    pub fn step(&mut self) {
        //calculates the nest gen
        //u built in next state in the def of board so use that

        for y in 0..self.height {
            for x in 0..self.width {
                let alive_neighbours = self.count_live_neighbors(x, y);
                let is_alive = self.is_alive(x, y);

                // let new_state: bool;

                // if !is_alive && self.rules.birth.contains(&alive_neighbours) {
                //     new_state = true;
                // } else if is_alive && self.rules.survive.contains(&alive_neighbours) {
                //     new_state = true;
                // } else {
                //     new_state = false;
                // }

                let new_state = (!is_alive && self.rules.birth.contains(&alive_neighbours))
                    || (is_alive && self.rules.survive.contains(&alive_neighbours));

                //update the next board state
                let idx = self.idx(x, y);
                self.next[idx] = new_state;
            }
        }

        //rewrite the board , no return
        std::mem::swap(&mut self.cells, &mut self.next);
    }
}
