extern crate reqwest;
extern crate serde_json;
extern crate serde;
extern crate array_tool;
//use std::collections::HashMap;

use std::str::FromStr;
use std::fmt::Display;
use serde::{Deserialize, Deserializer};
use array_tool::vec::Intersect;

const ALL :[u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];

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
    x: usize,
    y: usize,
    value: Option<u8>,
    num_iteration: u32,
}

// For now expect that all boards have size 9
struct Board {
    //squares: [[Option<u8>;  9]; 9]
    squares: [Square; 81],
    is_finished: bool,
}

impl std::fmt::Display for Board {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "").unwrap();
        for y in 0..9 {
            self.squares.into_iter()
                .filter (|square| square.y == y)
                .for_each(|val| {
                    let to_write = match val.value {
                        None => String::from(" "),
                        Some(value) => value.to_string()
                    };
                    write!(fmt, "|{}", to_write).unwrap()
                });
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
        return self.x == other.x && self.y == other.y && self.value == other.value;
    }
}
impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        let self_as_vec: Vec<_> = self.squares.iter().collect();
        let other_as_vec: Vec<_> = other.squares.iter().collect();
        return self_as_vec == other_as_vec;
    }
}



fn main() {
    println!("Sudoku solver");

    let board_response = make_request()
        .expect("Failed to get sudoku");
    println!("debug = {:#?}", board_response);
    let mut board = convert(board_response);
    println!("board = {}", board);
    solve(&mut board.squares, 0);
    println!("solved board = {}", board);
}

fn convert(response: BoardResponse) -> Board {
    let empty_square = Square {
                x: 0,
                y: 0,
                value: None,
                num_iteration: 0,
    };

    let mut arr: [Square; 81] = [empty_square; 81];

    for x in 0..9 {
        for y in 0..9 {
            arr[9*x + y] = Square {
                x: x,
                y: y,
                value: None,
                num_iteration: 0,
            };
        }
    }

    for square in response.squares {
        arr[9*square.x + square.y]= Square {
                x: square.x,
                y: square.y,
                value: Some(square.value),
                num_iteration: 0,
            };
    }

    return Board {
        squares: arr,
        is_finished: false,
    }
}

fn solve(squares: &mut [Square; 81], num_iteration: u32) -> bool {

    let mut all_possible_values: Vec<(usize, usize, Vec<&u8>)> = Vec::new();

    for square in squares.iter() {
        if square.value.is_none() {
            let possible_values = find_value(*squares, square.x, square.y);
            all_possible_values.push((square.x, square.y, possible_values));
        }
    }

    if all_possible_values.len() == 0 {
        return true;
    }

    let less_possible_values = all_possible_values.iter()
                .min_by_key(|value| value.2.len())
                .cloned()
                .unwrap();

    if less_possible_values.2.len() == 1 {
        let x = less_possible_values.0;
        let y = less_possible_values.1;
        let value = less_possible_values.2[0];
        let square : &mut Square = squares.iter_mut().find(|square| square.x == x && square.y == y).unwrap();

        square.value = Some(*value);
        square.num_iteration = num_iteration;


        // for square in squares.iter_mut() {
        //     if square.x == x && square.y == y {
        //         square.value = Some(*value);
        //         square.num_iteration = num_iteration;
        //         break;
        //     }
        // }
        return solve(squares, num_iteration);
    }

    let x = less_possible_values.0;
    let y = less_possible_values.1;
    let size = less_possible_values.2.len();
    for (index, possible_value) in less_possible_values.2.iter().enumerate() {
        let is_last = index == size - 1;
        let next_iteration = if is_last {num_iteration} else {num_iteration + 1};

        let square : &mut Square = squares.iter_mut().find(|square| square.x == x && square.y == y).unwrap();

        square.value = Some(**possible_value);
        square.num_iteration = next_iteration;
        // for square in squares.iter_mut() {
        //     if square.x == x && square.y == y {
        //         square.value = Some(**possible_value);
        //         square.num_iteration = next_iteration;
        //         break;
        //     }
        // }
        let is_finished = solve(squares, next_iteration);

        if is_finished {
            return true;
        }

        for square in squares.iter_mut() {
            if square.num_iteration == next_iteration {
                square.value = None;
                square.num_iteration = 0;
            }
        }
    }

    return false;
}

fn find_value(squares: [Square; 81], x: usize, y: usize) -> Vec<&'static u8> {
    let miss_x = missing_x(&squares, x);
    let miss_y = missing_y(&squares, y);
    let miss_square = miss_square(&squares, x, y);
    return miss_x.intersect(miss_y).intersect(miss_square);
}


fn missing_x(squares: &[Square; 81], x: usize) -> Vec<&'static u8> {

    let column = squares.iter()
        .filter(|square| square.x == x && square.value.is_some())
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.iter()
                .filter(|value| column.iter().find(|c| c == value).is_none())
                .collect();
}

fn missing_y(squares: &[Square; 81], y: usize) -> Vec<&'static u8> {

    let column = squares.iter()
        .filter(|square| square.y == y && square.value.is_some())
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.into_iter()
                .filter(|value| column.iter().find(|c| c == value).is_none())
                .collect();
}

fn miss_square(squares: &[Square; 81], x: usize, y: usize) -> Vec<&'static u8> {
    let offset = get_offset(x, y);
    let possible_x = offset.0.into_iter()
        .map(|x_offset| (*x_offset + x as i8) as usize)
        .collect::<Vec<_>>();
    let possible_y = offset.1.into_iter()
        .map(|y_offset| (*y_offset + y as i8) as usize)
        .collect::<Vec<_>>();

    let subset = squares.iter()
        .filter(|square| {
            square.value.is_some() && possible_x.contains(&square.x) && possible_y.contains(&square.y)
        })
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.into_iter()
                .filter(|value| subset.iter().find(|c| c == value).is_none())
                .collect();
}

fn get_offset(x: usize, y: usize) -> ([i8; 3], [i8; 3]) {
    let x_offset = match x % 3 {
        0 => [0, 1, 2],
        1 => [-1, 0, 1],
        2 => [-1, 0, -2],
        _ => panic!("Should not happen")
    };
    let y_offset = match y % 3 {
        0 => [0, 1, 2],
        1 => [-1, 0, 1],
        2 => [-1, 0, -2],
        _ => panic!("Should not happen")
    };
    return (x_offset, y_offset)
}

fn make_request() -> Result<BoardResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let json = client.get("http://www.cs.utep.edu/cheon/ws/sudoku/new")
                        .query(&[("size", "9"), ("level", "3")]) // What is &[()]
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

#[macro_use]
extern crate time_test;

#[cfg(test)]
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

        let result = Board {
            is_finished: true,
            squares: [Square {
                x: 0,
                y: 0,
                value: Some(7),
                num_iteration: 4
            }, Square {
                x: 0,
                y: 1,
                value: Some(2),
                num_iteration: 3
            }, Square {
                x: 0,
                y: 2,
                value: Some(3),
                num_iteration: 0
            }, Square {
                x: 0,
                y: 3,
                value: Some(1),
                num_iteration: 3
            }, Square {
                x: 0,
                y: 4,
                value: Some(4),
                num_iteration: 5
            }, Square {
                x: 0,
                y: 5,
                value: Some(9),
                num_iteration: 5
            }, Square {
                x: 0,
                y: 6,
                value: Some(6),
                num_iteration: 0
            }, Square {
                x: 0,
                y: 7,
                value: Some(8),
                num_iteration: 5
            }, Square {
                x: 0,
                y: 8,
                value: Some(5),
                num_iteration: 2
            }, Square {
                x: 1,
                y: 0,
                value: Some(9),
                num_iteration: 3
            }, Square {
                x: 1,
                y: 1,
                value: Some(4),
                num_iteration: 0
            }, Square {
                x: 1,
                y: 2,
                value: Some(5),
                num_iteration: 5
            }, Square {
                x: 1,
                y: 3,
                value: Some(7),
                num_iteration: 0
            }, Square {
                x: 1,
                y: 4,
                value: Some(6),
                num_iteration: 5
            }, Square {
                x: 1,
                y: 5,
                value: Some(8),
                num_iteration: 0
            }, Square {
                x: 1,
                y: 6,
                value: Some(2),
                num_iteration: 2
            }, Square {
                x: 1,
                y: 7,
                value: Some(3),
                num_iteration: 0
            }, Square {
                x: 1,
                y: 8,
                value: Some(1),
                num_iteration: 1
            }, Square {
                x: 2,
                y: 0,
                value: Some(1),
                num_iteration: 0
            }, Square {
                x: 2,
                y: 1,
                value: Some(6),
                num_iteration: 5
            }, Square {
                x: 2,
                y: 2,
                value: Some(8),
                num_iteration: 5
            }, Square {
                x: 2,
                y: 3,
                value: Some(2),
                num_iteration: 5
            }, Square {
                x: 2,
                y: 4,
                value: Some(5),
                num_iteration: 5
            }, Square {
                x: 2,
                y: 5,
                value: Some(3),
                num_iteration: 5
            }, Square {
                x: 2,
                y: 6,
                value: Some(7),
                num_iteration: 5
            }, Square {
                x: 2,
                y: 7,
                value: Some(4),
                num_iteration: 5
            }, Square {
                x: 2,
                y: 8,
                value: Some(9),
                num_iteration: 0
            }, Square {
                x: 3,
                y: 0,
                value: Some(8),
                num_iteration: 6
            }, Square {
                x: 3,
                y: 1,
                value: Some(7),
                num_iteration: 0
            }, Square {
                x: 3,
                y: 2,
                value: Some(2),
                num_iteration: 6
            }, Square {
                x: 3,
                y: 3,
                value: Some(4),
                num_iteration: 0
            }, Square {
                x: 3,
                y: 4,
                value: Some(3),
                num_iteration: 5
            }, Square {
                x: 3,
                y: 5,
                value: Some(5),
                num_iteration: 0
            }, Square {
                x: 3,
                y: 6,
                value: Some(9),
                num_iteration: 6
            }, Square {
                x: 3,
                y: 7,
                value: Some(1),
                num_iteration: 0
            }, Square {
                x: 3,
                y: 8,
                value: Some(6),
                num_iteration: 6
            }, Square {
                x: 4,
                y: 0,
                value: Some(4),
                num_iteration: 6
            }, Square {
                x: 4,
                y: 1,
                value: Some(9),
                num_iteration: 6
            }, Square {
                x: 4,
                y: 2,
                value: Some(1),
                num_iteration: 6
            }, Square {
                x: 4,
                y: 3,
                value: Some(6),
                num_iteration: 5
            }, Square {
                x: 4,
                y: 4,
                value: Some(8),
                num_iteration: 0
            }, Square {
                x: 4,
                y: 5,
                value: Some(7),
                num_iteration: 5
            }, Square {
                x: 4,
                y: 6,
                value: Some(5),
                num_iteration: 6
            }, Square {
                x: 4,
                y: 7,
                value: Some(2),
                num_iteration: 5
            }, Square {
                x: 4,
                y: 8,
                value: Some(3),
                num_iteration: 6
            }, Square {
                x: 5,
                y: 0,
                value: Some(5),
                num_iteration: 6
            }, Square {
                x: 5,
                y: 1,
                value: Some(3),
                num_iteration: 0
            }, Square {
                x: 5,
                y: 2,
                value: Some(6),
                num_iteration: 6
            }, Square {
                x: 5,
                y: 3,
                value: Some(9),
                num_iteration: 0
            }, Square {
                x: 5,
                y: 4,
                value: Some(2),
                num_iteration: 5
            }, Square {
                x: 5,
                y: 5,
                value: Some(1),
                num_iteration: 0
            }, Square {
                x: 5,
                y: 6,
                value: Some(8),
                num_iteration: 6
            }, Square {
                x: 5,
                y: 7,
                value: Some(7),
                num_iteration: 0
            }, Square {
                x: 5,
                y: 8,
                value: Some(4),
                num_iteration: 6
            }, Square {
                x: 6,
                y: 0,
                value: Some(6),
                num_iteration: 0
            }, Square {
                x: 6,
                y: 1,
                value: Some(5),
                num_iteration: 6
            }, Square {
                x: 6,
                y: 2,
                value: Some(7),
                num_iteration: 6
            }, Square {
                x: 6,
                y: 3,
                value: Some(8),
                num_iteration: 6
            }, Square {
                x: 6,
                y: 4,
                value: Some(1),
                num_iteration: 6
            }, Square {
                x: 6,
                y: 5,
                value: Some(4),
                num_iteration: 5
            }, Square {
                x: 6,
                y: 6,
                value: Some(3),
                num_iteration: 6
            }, Square {
                x: 6,
                y: 7,
                value: Some(9),
                num_iteration: 5
            }, Square {
                x: 6,
                y: 8,
                value: Some(2),
                num_iteration: 0
            }, Square {
                x: 7,
                y: 0,
                value: Some(2),
                num_iteration: 6
            }, Square {
                x: 7,
                y: 1,
                value: Some(8),
                num_iteration: 0
            }, Square {
                x: 7,
                y: 2,
                value: Some(4),
                num_iteration: 6
            }, Square {
                x: 7,
                y: 3,
                value: Some(3),
                num_iteration: 0
            }, Square {
                x: 7,
                y: 4,
                value: Some(9),
                num_iteration: 5
            }, Square {
                x: 7,
                y: 5,
                value: Some(6),
                num_iteration: 0
            }, Square {
                x: 7,
                y: 6,
                value: Some(1),
                num_iteration: 5
            }, Square {
                x: 7,
                y: 7,
                value: Some(5),
                num_iteration: 0
            }, Square {
                x: 7,
                y: 8,
                value: Some(7),
                num_iteration: 1
            }, Square {
                x: 8,
                y: 0,
                value: Some(3),
                num_iteration: 6
            }, Square {
                x: 8,
                y: 1,
                value: Some(1),
                num_iteration: 6
            }, Square {
                x: 8,
                y: 2,
                value: Some(9),
                num_iteration: 0
            }, Square {
                x: 8,
                y: 3,
                value: Some(5),
                num_iteration: 6
            }, Square {
                x: 8,
                y: 4,
                value: Some(7),
                num_iteration: 6
            }, Square {
                x: 8,
                y: 5,
                value: Some(2),
                num_iteration: 5
            }, Square {
                x: 8,
                y: 6,
                value: Some(4),
                num_iteration: 0
            }, Square {
                x: 8,
                y: 7,
                value: Some(6),
                num_iteration: 5
            }, Square {
                x: 8,
                y: 8,
                value: Some(8),
                num_iteration: 6
            }]
        };
        let mut board = convert(board_response);

        time_test!();

        solve(&mut board.squares, 0);
        assert_eq!(result, board);
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

        let result = Board {
            is_finished: true,
            squares: [
                Square {
                    x: 0,
                    y: 0,
                    value: Some(
                        4,
                    ),
                    num_iteration: 8,
                },
                Square {
                    x: 0,
                    y: 1,
                    value: Some(
                        6,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 0,
                    y: 2,
                    value: Some(
                        1,
                    ),
                    num_iteration: 8,
                },
                Square {
                    x: 0,
                    y: 3,
                    value: Some(
                        2,
                    ),
                    num_iteration: 10,
                },
                Square {
                    x: 0,
                    y: 4,
                    value: Some(
                        7,
                    ),
                    num_iteration: 11,
                },
                Square {
                    x: 0,
                    y: 5,
                    value: Some(
                        3,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 0,
                    y: 6,
                    value: Some(
                        5,
                    ),
                    num_iteration: 2,
                },
                Square {
                    x: 0,
                    y: 7,
                    value: Some(
                        8,
                    ),
                    num_iteration: 12,
                },
                Square {
                    x: 0,
                    y: 8,
                    value: Some(
                        9,
                    ),
                    num_iteration: 12,
                },
                Square {
                    x: 1,
                    y: 0,
                    value: Some(
                        7,
                    ),
                    num_iteration: 8,
                },
                Square {
                    x: 1,
                    y: 1,
                    value: Some(
                        2,
                    ),
                    num_iteration: 7,
                },
                Square {
                    x: 1,
                    y: 2,
                    value: Some(
                        8,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 1,
                    y: 3,
                    value: Some(
                        9,
                    ),
                    num_iteration: 9,
                },
                Square {
                    x: 1,
                    y: 4,
                    value: Some(
                        5,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 1,
                    y: 5,
                    value: Some(
                        4,
                    ),
                    num_iteration: 9,
                },
                Square {
                    x: 1,
                    y: 6,
                    value: Some(
                        1,
                    ),
                    num_iteration: 1,
                },
                Square {
                    x: 1,
                    y: 7,
                    value: Some(
                        3,
                    ),
                    num_iteration: 9,
                },
                Square {
                    x: 1,
                    y: 8,
                    value: Some(
                        6,
                    ),
                    num_iteration: 9,
                },
                Square {
                    x: 2,
                    y: 0,
                    value: Some(
                        9,
                    ),
                    num_iteration: 6,
                },
                Square {
                    x: 2,
                    y: 1,
                    value: Some(
                        5,
                    ),
                    num_iteration: 8,
                },
                Square {
                    x: 2,
                    y: 2,
                    value: Some(
                        3,
                    ),
                    num_iteration: 8,
                },
                Square {
                    x: 2,
                    y: 3,
                    value: Some(
                        1,
                    ),
                    num_iteration: 13,
                },
                Square {
                    x: 2,
                    y: 4,
                    value: Some(
                        8,
                    ),
                    num_iteration: 12,
                },
                Square {
                    x: 2,
                    y: 5,
                    value: Some(
                        6,
                    ),
                    num_iteration: 13,
                },
                Square {
                    x: 2,
                    y: 6,
                    value: Some(
                        4,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 2,
                    y: 7,
                    value: Some(
                        7,
                    ),
                    num_iteration: 12,
                },
                Square {
                    x: 2,
                    y: 8,
                    value: Some(
                        2,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 3,
                    y: 0,
                    value: Some(
                        5,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 3,
                    y: 1,
                    value: Some(
                        1,
                    ),
                    num_iteration: 14,
                },
                Square {
                    x: 3,
                    y: 2,
                    value: Some(
                        2,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 3,
                    y: 3,
                    value: Some(
                        6,
                    ),
                    num_iteration: 15,
                },
                Square {
                    x: 3,
                    y: 4,
                    value: Some(
                        3,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 3,
                    y: 5,
                    value: Some(
                        7,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 3,
                    y: 6,
                    value: Some(
                        9,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 3,
                    y: 7,
                    value: Some(
                        4,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 3,
                    y: 8,
                    value: Some(
                        8,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 4,
                    y: 0,
                    value: Some(
                        6,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 4,
                    y: 1,
                    value: Some(
                        9,
                    ),
                    num_iteration: 14,
                },
                Square {
                    x: 4,
                    y: 2,
                    value: Some(
                        7,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 4,
                    y: 3,
                    value: Some(
                        8,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 4,
                    y: 4,
                    value: Some(
                        4,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 4,
                    y: 5,
                    value: Some(
                        1,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 4,
                    y: 6,
                    value: Some(
                        3,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 4,
                    y: 7,
                    value: Some(
                        2,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 4,
                    y: 8,
                    value: Some(
                        5,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 5,
                    y: 0,
                    value: Some(
                        8,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 5,
                    y: 1,
                    value: Some(
                        3,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 5,
                    y: 2,
                    value: Some(
                        4,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 5,
                    y: 3,
                    value: Some(
                        5,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 5,
                    y: 4,
                    value: Some(
                        2,
                    ),
                    num_iteration: 17,
                },
                Square {
                    x: 5,
                    y: 5,
                    value: Some(
                        9,
                    ),
                    num_iteration: 17,
                },
                Square {
                    x: 5,
                    y: 6,
                    value: Some(
                        6,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 5,
                    y: 7,
                    value: Some(
                        1,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 5,
                    y: 8,
                    value: Some(
                        7,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 6,
                    y: 0,
                    value: Some(
                        2,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 6,
                    y: 1,
                    value: Some(
                        7,
                    ),
                    num_iteration: 8,
                },
                Square {
                    x: 6,
                    y: 2,
                    value: Some(
                        6,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 6,
                    y: 3,
                    value: Some(
                        4,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 6,
                    y: 4,
                    value: Some(
                        1,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 6,
                    y: 5,
                    value: Some(
                        5,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 6,
                    y: 6,
                    value: Some(
                        8,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 6,
                    y: 7,
                    value: Some(
                        9,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 6,
                    y: 8,
                    value: Some(
                        3,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 7,
                    y: 0,
                    value: Some(
                        3,
                    ),
                    num_iteration: 6,
                },
                Square {
                    x: 7,
                    y: 1,
                    value: Some(
                        4,
                    ),
                    num_iteration: 5,
                },
                Square {
                    x: 7,
                    y: 2,
                    value: Some(
                        9,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 7,
                    y: 3,
                    value: Some(
                        7,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 7,
                    y: 4,
                    value: Some(
                        6,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 7,
                    y: 5,
                    value: Some(
                        8,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 7,
                    y: 6,
                    value: Some(
                        2,
                    ),
                    num_iteration: 3,
                },
                Square {
                    x: 7,
                    y: 7,
                    value: Some(
                        5,
                    ),
                    num_iteration: 4,
                },
                Square {
                    x: 7,
                    y: 8,
                    value: Some(
                        1,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 8,
                    y: 0,
                    value: Some(
                        1,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 8,
                    y: 1,
                    value: Some(
                        8,
                    ),
                    num_iteration: 8,
                },
                Square {
                    x: 8,
                    y: 2,
                    value: Some(
                        5,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 8,
                    y: 3,
                    value: Some(
                        3,
                    ),
                    num_iteration: 10,
                },
                Square {
                    x: 8,
                    y: 4,
                    value: Some(
                        9,
                    ),
                    num_iteration: 17,
                },
                Square {
                    x: 8,
                    y: 5,
                    value: Some(
                        2,
                    ),
                    num_iteration: 17,
                },
                Square {
                    x: 8,
                    y: 6,
                    value: Some(
                        7,
                    ),
                    num_iteration: 0,
                },
                Square {
                    x: 8,
                    y: 7,
                    value: Some(
                        6,
                    ),
                    num_iteration: 16,
                },
                Square {
                    x: 8,
                    y: 8,
                    value: Some(
                        4,
                    ),
                    num_iteration: 0,
                },
            ]
        };

        let mut board = convert(board_response);
        time_test!();
        solve(&mut board.squares, 0);
        assert_eq!(result, board);
    }
}
