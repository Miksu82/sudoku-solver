#[macro_use]
extern crate lazy_static; // used in sudoku::solver

#[cfg(test)]
#[macro_use]
extern crate time_test; // used in sudoku::solver::test

pub mod sudoku;

fn main() {
    println!("Sudoku solver");

    let mut sudoku =
        sudoku::get::create_sudoku(sudoku::get::Difficulty::Hard).expect("Failed to get sudoku");
    println!("Sudoku = {}", sudoku);
    sudoku::solver::solve(&mut sudoku);
    println!("solved Sudoku = {}", sudoku);
}
