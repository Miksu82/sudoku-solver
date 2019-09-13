pub mod get;
pub mod solver;

const SUDOKU_SIZE: usize = 9;
// Unfortunately f64.sqrt() is not const fn so BOX_SIZE cannot be computed
// in compile time.
const BOX_SIZE: usize = 3;

#[derive(Debug, Copy, Clone)]
struct Cell {
    value: u8,
    num_iteration: u32,
}

type Cells = [[Option<Cell>; SUDOKU_SIZE]; SUDOKU_SIZE];

#[derive(Copy, Clone)]
pub struct Sudoku {
    cells: Cells,
}

impl std::fmt::Display for Sudoku {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "").unwrap();
        for x in 0..SUDOKU_SIZE {
            for y in 0..SUDOKU_SIZE {
                let to_write = match self.cells[x][y] {
                    None => String::from(" "),
                    Some(cell) => cell.value.to_string(),
                };
                write!(fmt, "|{}", to_write).unwrap()
            }
            writeln!(fmt, "|").unwrap();
        }
        return Ok(());
    }
}

impl std::fmt::Debug for Sudoku {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.cells[..].fmt(formatter)
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        return self.value == other.value;
    }
}
impl PartialEq for Sudoku {
    fn eq(&self, other: &Self) -> bool {
        return self.cells == other.cells;
    }
}
