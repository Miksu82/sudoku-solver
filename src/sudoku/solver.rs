use super::constants::{BOX_SIZE, SUDOKU_SIZE};
use super::{Cell, Cells, Sudoku};

lazy_static! {

    // Why this doesn't work
    //static ref BOX_SIZE : usize = (SUDOKU_SIZE as f64).sqrt() as usize;

    /// A table with indeces for a box. For example for x=3, y=1 the containing box has
    /// indeces (x: [3, 4, 5], y: [0, 1, 2])
    static ref BOX_INDECES : [[([usize; BOX_SIZE], [usize; BOX_SIZE]); SUDOKU_SIZE]; SUDOKU_SIZE] = {

        fn get_box_indeces(index: usize) -> [usize; BOX_SIZE] {
            let offset = (index % BOX_SIZE) as i8;
            let indeces = (0..(BOX_SIZE as i8))
                            .map(|value| value - offset) // offset inside the box
                            .map(|value| value + (index as i8)) // convert to index
                            .map(|value| value as usize); // convert to correct type

            return array_init::from_iter(indeces).unwrap();
        }

        let mut arr = [[([0; BOX_SIZE], [0; BOX_SIZE]); SUDOKU_SIZE]; SUDOKU_SIZE];
        for x in 0..SUDOKU_SIZE {
            for y in 0..SUDOKU_SIZE {
                arr[x][y].0 = get_box_indeces(x);
                arr[x][y].1 = get_box_indeces(y);
            }
        }
        arr
    };
}

pub fn solve(sudoku: &mut Sudoku) -> bool {
    return solve_internal(&mut sudoku.cells, 0);
}

fn solve_internal(cells: &mut Cells, num_iteration: u32) -> bool {
    let mut all_possible_values: Vec<(usize, usize, Vec<u8>)> = Vec::new();

    let mut should_finish = false;
    for x in 0..SUDOKU_SIZE {
        for y in 0..SUDOKU_SIZE {
            if cells[x][y].is_none() {
                let possible_values = find_value(cells, x, y);
                should_finish = possible_values.len() == 1;

                all_possible_values.push((x, y, possible_values));

                if should_finish {
                    break;
                }
            }
        }

        if should_finish {
            break;
        }
    }

    if all_possible_values.len() == 0 {
        return true;
    }

    let less_possible_values = if should_finish {
        all_possible_values.last().unwrap()
    } else {
        all_possible_values
            .iter()
            .min_by_key(|value| value.2.len())
            .unwrap()
    };

    if less_possible_values.2.len() == 1 {
        let x = less_possible_values.0;
        let y = less_possible_values.1;
        let value = less_possible_values.2[0];
        cells[x][y] = Some(Cell {
            value: value,
            num_iteration: num_iteration,
        });

        return solve_internal(cells, num_iteration);
    }

    let x = less_possible_values.0;
    let y = less_possible_values.1;
    let size = less_possible_values.2.len();
    for (index, possible_value) in less_possible_values.2.iter().enumerate() {
        let is_last = index == size - 1;
        let next_iteration = if is_last {
            num_iteration
        } else {
            num_iteration + 1
        };

        cells[x][y] = Some(Cell {
            value: *possible_value,
            num_iteration: next_iteration,
        });

        let is_finished = solve_internal(cells, next_iteration);

        if is_finished {
            return true;
        }

        for x in 0..SUDOKU_SIZE {
            for y in 0..SUDOKU_SIZE {
                if cells[x][y].is_some() && cells[x][y].unwrap().num_iteration == next_iteration {
                    cells[x][y] = None;
                }
            }
        }
    }

    return false;
}

fn find_value(cells: &mut Cells, x: usize, y: usize) -> Vec<u8> {
    // [Some(1), Some(2), ...];
    let mut possible_values: [Option<u8>; SUDOKU_SIZE] =
        array_init::array_init(|i| Some((i as u8) + 1));

    let mut remove_value_if_necessary = |cell: &Option<Cell>| {
        if cell.is_some() {
            possible_values[(cell.unwrap().value - 1) as usize] = None;
        }
    };

    for rolling in 0..SUDOKU_SIZE {
        let cell_x = cells[x][rolling];
        remove_value_if_necessary(&cell_x);

        let cell_y = cells[rolling][y];
        remove_value_if_necessary(&cell_y);
    }

    let box_indeces = BOX_INDECES[x][y];
    for box_x in box_indeces.0.iter() {
        for box_y in box_indeces.1.iter() {
            let cell = cells[*box_x][*box_y];
            remove_value_if_necessary(&cell);
        }
    }

    return possible_values.iter().filter_map(|x| *x).collect();
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_solve() {
        let mut sudoku = Sudoku {
            cells: [
                [
                    None,
                    None,
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                ],
                [
                    None,
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 8,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    None,
                ],
                [
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 9,
                        num_iteration: 0,
                    }),
                ],
                [
                    None,
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    None,
                ],
                [
                    None,
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 8,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                ],
                [
                    None,
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 9,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    None,
                ],
                [
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 2,
                        num_iteration: 0,
                    }),
                ],
                [
                    None,
                    Some(Cell {
                        value: 8,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    None,
                ],
                [
                    None,
                    None,
                    Some(Cell {
                        value: 9,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                ],
            ],
        };

        let expected_sudoku = Sudoku {
            cells: [
                [
                    Some(Cell {
                        value: 7,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 2,
                    }),
                ],
                [
                    Some(Cell {
                        value: 9,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 1,
                    }),
                ],
                [
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 0,
                    }),
                ],
                [
                    Some(Cell {
                        value: 8,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 3,
                    }),
                ],
                [
                    Some(Cell {
                        value: 4,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 3,
                    }),
                ],
                [
                    Some(Cell {
                        value: 5,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 3,
                    }),
                ],
                [
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 0,
                    }),
                ],
                [
                    Some(Cell {
                        value: 2,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 1,
                    }),
                ],
                [
                    Some(Cell {
                        value: 3,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 3,
                    }),
                ],
            ],
        };

        //time_test!();

        solve(&mut sudoku);
        assert_eq!(expected_sudoku, sudoku);
    }

    #[test]
    fn test_may_take_long_time_to_solve() {
        let mut sudoku = Sudoku {
            cells: [
                [
                    None,
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                ],
                [
                    None,
                    None,
                    Some(Cell {
                        value: 8,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                ],
                [
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 2,
                        num_iteration: 0,
                    }),
                ],
                [
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
                [
                    None,
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                ],
                [
                    None,
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    None,
                ],
                [
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                ],
                [
                    None,
                    None,
                    Some(Cell {
                        value: 9,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                ],
                [
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    None,
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                ],
            ],
        };

        let expected_sudoku = Sudoku {
            cells: [
                [
                    Some(Cell {
                        value: 4,
                        num_iteration: 8,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 8,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 10,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 11,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 2,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 12,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 12,
                    }),
                ],
                [
                    Some(Cell {
                        value: 7,
                        num_iteration: 8,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 7,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 9,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 9,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 1,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 9,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 9,
                    }),
                ],
                [
                    Some(Cell {
                        value: 9,
                        num_iteration: 6,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 8,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 8,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 13,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 12,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 13,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 12,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 0,
                    }),
                ],
                [
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 14,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 15,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 16,
                    }),
                ],
                [
                    Some(Cell {
                        value: 6,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 14,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 16,
                    }),
                ],
                [
                    Some(Cell {
                        value: 8,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 17,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 17,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 16,
                    }),
                ],
                [
                    Some(Cell {
                        value: 2,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 8,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 0,
                    }),
                ],
                [
                    Some(Cell {
                        value: 3,
                        num_iteration: 6,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 5,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 3,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 4,
                    }),
                    Some(Cell {
                        value: 1,
                        num_iteration: 16,
                    }),
                ],
                [
                    Some(Cell {
                        value: 1,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 8,
                        num_iteration: 8,
                    }),
                    Some(Cell {
                        value: 5,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 3,
                        num_iteration: 10,
                    }),
                    Some(Cell {
                        value: 9,
                        num_iteration: 17,
                    }),
                    Some(Cell {
                        value: 2,
                        num_iteration: 17,
                    }),
                    Some(Cell {
                        value: 7,
                        num_iteration: 0,
                    }),
                    Some(Cell {
                        value: 6,
                        num_iteration: 16,
                    }),
                    Some(Cell {
                        value: 4,
                        num_iteration: 0,
                    }),
                ],
            ],
        };
        //time_test!();

        solve(&mut sudoku);
        assert_eq!(expected_sudoku, sudoku);
    }
}
