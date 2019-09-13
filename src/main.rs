extern crate reqwest;
extern crate serde_json;
extern crate serde;
extern crate array_tool;

#[macro_use]
extern crate lazy_static;

use std::str::FromStr;
use std::fmt::Display;
use serde::{Deserialize, Deserializer};

const BOARD_SIZE : usize = 9;
const DIVIDER_SIZE : usize = 3;
lazy_static! {

    // Why this doesn't work
    //static ref DIVIDER_SIZE : usize = (BOARD_SIZE as f64).sqrt() as usize;

    static ref OFFSET : [[([usize; DIVIDER_SIZE], [usize; DIVIDER_SIZE]); BOARD_SIZE]; BOARD_SIZE] = {

        let mut arr = [[([0; DIVIDER_SIZE], [0; DIVIDER_SIZE]); BOARD_SIZE]; BOARD_SIZE];
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let indeces_x = get_offsets(x);
                let indeces_y = get_offsets(y);
                arr[x][y].0 = indeces_x;
                arr[x][y].1 = indeces_y;
            }
        }
        arr
    };
}

const ALL :[Option<u8>; BOARD_SIZE] = [Some(1), Some(2), Some(3), Some(4), Some(5), Some(6), Some(7), Some(8), Some(9)];

#[derive(Deserialize, Debug)]
struct SquareResponse {
    x: usize,
    y: usize,
    value: u8,
}

#[derive(Deserialize, Debug)]
struct BoardResponse {
    response: bool,
    #[serde(deserialize_with = "from_str")]
    size: u8,
    squares: Vec<SquareResponse>,
}

#[derive(Debug, Copy, Clone)]
struct Square {
    value: u8,
    num_iteration: u32,
}

type Squares = [[Option<Square>;  BOARD_SIZE]; BOARD_SIZE];

#[derive(Copy, Clone)]
struct Board {
    squares: Squares,
}

impl std::fmt::Display for Board {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "").unwrap();
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let to_write = match self.squares[x][y] {
                    None => String::from(" "),
                    Some(square) => square.value.to_string()
                };
                write!(fmt, "|{}", to_write).unwrap()
            }
            writeln!(fmt, "|").unwrap();
        }
        return Ok(());
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.squares[..].fmt(formatter)
    }
}

impl PartialEq for Square {
    fn eq(&self, other: &Self) -> bool {
        return self.value == other.value;
    }
}
impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        return self.squares == other.squares;
    }
}


fn main() {
    println!("Sudoku solver");

    let board_response = make_request()
        .expect("Failed to get sudoku");
    let mut board = convert(board_response);
    println!("board = {}", board);
    solve(&mut board.squares, 0);
    println!("solved board = {}", board);
}

fn convert(response: BoardResponse) -> Board {
    let mut squares : Squares = [[None; BOARD_SIZE]; BOARD_SIZE];

    for square in response.squares {
        squares[square.x][square.y] = Some(Square {
            value: square.value,
            num_iteration: 0,
        })
    }

    return Board {
        squares: squares,
    }
}

fn solve(squares: &mut Squares, num_iteration: u32) -> bool {
    let mut all_possible_values: Vec<(usize, usize, Vec<u8>)> = Vec::new();

    let mut should_finish = false;
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            if squares[x][y].is_none() {
                let possible_values = find_value(squares, x, y);
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

    let less_possible_values = if should_finish  {
        all_possible_values.last().unwrap()
    } else {
        all_possible_values.iter()
                .min_by_key(|value| value.2.len())
                .unwrap()
    };

    if less_possible_values.2.len() == 1 {
        let x = less_possible_values.0;
        let y = less_possible_values.1;
        let value = less_possible_values.2[0];
        squares[x][y] = Some(Square {
            value: value,
            num_iteration: num_iteration,
        });

        return solve(squares, num_iteration);
    }

    let x = less_possible_values.0;
    let y = less_possible_values.1;
    let size = less_possible_values.2.len();
    for (index, possible_value) in less_possible_values.2.iter().enumerate() {
        let is_last = index == size - 1;
        let next_iteration = if is_last {num_iteration} else {num_iteration + 1};

         squares[x][y] = Some(Square {
            value: *possible_value,
            num_iteration: next_iteration,
        });

        let is_finished = solve(squares, next_iteration);

        if is_finished {
            return true;
        }

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if squares[x][y].is_some() &&  squares[x][y].unwrap().num_iteration == next_iteration {
                    squares[x][y] = None;
                }
            }
        }
    }

    return false;
}

fn find_value(squares: &mut Squares, x: usize, y: usize) -> Vec<u8> {

    let mut possible_values : [Option<u8>; BOARD_SIZE] = [None; BOARD_SIZE];
    possible_values.copy_from_slice(&ALL);

    let mut set_value_if_necessary = |square: &Option<Square>| {
        if square.is_some() {
            possible_values[(square.unwrap().value - 1) as usize] = None;
        }
    };

    for rolling_y in 0..BOARD_SIZE {
        let square = squares[x][rolling_y];
        set_value_if_necessary(&square);
    }

    for rolling_x in 0..BOARD_SIZE {
        let square = squares[rolling_x][y];
        set_value_if_necessary(&square);
    }

    let offset = OFFSET[x][y];
    for rolling_x in offset.0.iter() {
        for rolling_y in offset.1.iter() {
            let square = squares[*rolling_x][*rolling_y];
            set_value_if_necessary(&square);
        }
    }

    return possible_values.iter()
            .filter_map(|x| *x)
            .collect();
}

fn get_offsets(index: usize) -> [usize; DIVIDER_SIZE] {
    let offset = (index % DIVIDER_SIZE) as i8;
    let rolling = 0..(DIVIDER_SIZE as i8);

    let offsets = rolling.map(|value| value - offset);
    let mut indeces : [usize; DIVIDER_SIZE] = [0; DIVIDER_SIZE];

    for (i, value) in offsets.enumerate() {
        indeces[i] = (value + index as i8) as usize;
    }

    return indeces;
}

fn make_request() -> Result<BoardResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let json = client.get("http://www.cs.utep.edu/cheon/ws/sudoku/new")
                        .query(&[("size", BOARD_SIZE.to_string()), ("level", 3.to_string())]) // What is &[()]
                        .send()?
                        .json()?;
    return Ok(json)
}

// I don't know what this does. Taken from https://github.com/serde-rs/json/issues/317
fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

#[cfg(test)]
#[macro_use]
extern crate time_test;

mod tests {

    use super::*;

    #[test]
    fn test_solve() {
        let board_response = BoardResponse {
            response: true,
            size: 9,
            squares: vec![
                SquareResponse {
                    x: 2,
                    y: 0,
                    value: 1,
                },
                SquareResponse {
                    x: 6,
                    y: 0,
                    value: 6,
                },
                SquareResponse {
                    x: 1,
                    y: 1,
                    value: 4,
                },
                SquareResponse {
                    x: 3,
                    y: 1,
                    value: 7,
                },
                SquareResponse {
                    x: 5,
                    y: 1,
                    value: 3,
                },
                SquareResponse {
                    x: 7,
                    y: 1,
                    value: 8,
                },
                SquareResponse {
                    x: 0,
                    y: 2,
                    value: 3,
                },
                SquareResponse {
                    x: 8,
                    y: 2,
                    value: 9,
                },
                SquareResponse {
                    x: 1,
                    y: 3,
                    value: 7,
                },
                SquareResponse {
                    x: 3,
                    y: 3,
                    value: 4,
                },
                SquareResponse {
                    x: 5,
                    y: 3,
                    value: 9,
                },
                SquareResponse {
                    x: 7,
                    y: 3,
                    value: 3,
                },
                SquareResponse {
                    x: 4,
                    y: 4,
                    value: 8,
                },
                SquareResponse {
                    x: 1,
                    y: 5,
                    value: 8,
                },
                SquareResponse {
                    x: 3,
                    y: 5,
                    value: 5,
                },
                SquareResponse {
                    x: 5,
                    y: 5,
                    value: 1,
                },
                SquareResponse {
                    x: 7,
                    y: 5,
                    value: 6,
                },
                SquareResponse {
                    x: 0,
                    y: 6,
                    value: 6,
                },
                SquareResponse {
                    x: 8,
                    y: 6,
                    value: 4,
                },
                SquareResponse {
                    x: 1,
                    y: 7,
                    value: 3,
                },
                SquareResponse {
                    x: 3,
                    y: 7,
                    value: 1,
                },
                SquareResponse {
                    x: 5,
                    y: 7,
                    value: 7,
                },
                SquareResponse {
                    x: 7,
                    y: 7,
                    value: 5,
                },
                SquareResponse {
                    x: 2,
                    y: 8,
                    value: 9,
                },
                SquareResponse {
                    x: 6,
                    y: 8,
                    value: 2,
                },
            ],
        };

        let expected_board = Board {
            squares: [
    [
        Some(
            Square {
                value: 7,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 2,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 9,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 1,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 1,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 0,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 8,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 3,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 4,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 3,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 5,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 3,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 6,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 0,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 2,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 1,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 3,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 3,
            },
        ),
    ],
]

        };

        let mut board = convert(board_response);

        time_test!();

        solve(&mut board.squares, 0);
        assert_eq!(expected_board, board);
    }

    #[test]
    fn test_may_take_long_time_to_solve() {
        let board_response = BoardResponse {
    response: true,
    size: 9,
    squares: vec![
        SquareResponse {
            x: 0,
            y: 1,
            value: 6,
        },
        SquareResponse {
            x: 0,
            y: 5,
            value: 3,
        },
        SquareResponse {
            x: 1,
            y: 2,
            value: 8,
        },
        SquareResponse {
            x: 1,
            y: 4,
            value: 5,
        },
        SquareResponse {
            x: 2,
            y: 6,
            value: 4,
        },
        SquareResponse {
            x: 2,
            y: 8,
            value: 2,
        },
        SquareResponse {
            x: 3,
            y: 0,
            value: 5,
        },
        SquareResponse {
            x: 4,
            y: 4,
            value: 4,
        },
        SquareResponse {
            x: 4,
            y: 6,
            value: 3,
        },
        SquareResponse {
            x: 5,
            y: 1,
            value: 3,
        },
        SquareResponse {
            x: 5,
            y: 2,
            value: 4,
        },
        SquareResponse {
            x: 5,
            y: 7,
            value: 1,
        },
        SquareResponse {
            x: 6,
            y: 5,
            value: 5,
        },
        SquareResponse {
            x: 6,
            y: 8,
            value: 3,
        },
        SquareResponse {
            x: 7,
            y: 2,
            value: 9,
        },
        SquareResponse {
            x: 7,
            y: 3,
            value: 7,
        },
        SquareResponse {
            x: 7,
            y: 4,
            value: 6,
        },
        SquareResponse {
            x: 8,
            y: 0,
            value: 1,
        },
        SquareResponse {
            x: 8,
            y: 6,
            value: 7,
        },
        SquareResponse {
            x: 8,
            y: 8,
            value: 4,
        },
    ],
};

let expected_board = Board {
    squares: [
    [
        Some(
            Square {
                value: 4,
                num_iteration: 8,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 8,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 10,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 11,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 2,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 12,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 12,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 7,
                num_iteration: 8,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 7,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 9,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 9,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 1,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 9,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 9,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 9,
                num_iteration: 6,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 8,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 8,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 13,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 12,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 13,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 12,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 0,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 5,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 14,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 15,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 16,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 6,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 14,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 16,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 8,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 17,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 17,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 16,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 2,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 8,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 0,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 3,
                num_iteration: 6,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 5,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 3,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 4,
            },
        ),
        Some(
            Square {
                value: 1,
                num_iteration: 16,
            },
        ),
    ],
    [
        Some(
            Square {
                value: 1,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 8,
                num_iteration: 8,
            },
        ),
        Some(
            Square {
                value: 5,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 3,
                num_iteration: 10,
            },
        ),
        Some(
            Square {
                value: 9,
                num_iteration: 17,
            },
        ),
        Some(
            Square {
                value: 2,
                num_iteration: 17,
            },
        ),
        Some(
            Square {
                value: 7,
                num_iteration: 0,
            },
        ),
        Some(
            Square {
                value: 6,
                num_iteration: 16,
            },
        ),
        Some(
            Square {
                value: 4,
                num_iteration: 0,
            },
        ),
    ],
]
};

        let mut board = convert(board_response);
        time_test!();
        println!("board = {}", board);

        solve(&mut board.squares, 0);
        assert_eq!(expected_board, board);
    }
}
