extern crate reqwest;
extern crate serde_json;
extern crate serde;
extern crate array_tool;
#[macro_use]
extern crate lazy_static;
//use std::collections::HashMap;

use std::str::FromStr;
use std::fmt::Display;
use serde::{Deserialize, Deserializer};
use array_tool::vec::Intersect;

lazy_static! {
    static ref OFFSET : [[([usize; 3], [usize; 3]); 9]; 9] = {

        let mut arr = [[([0; 3], [0; 3]); 9]; 9];
        for x in 0..9 {
            for y in 0..9 {
                let offset = get_offset(x, y);
                let possible_x = offset.0.into_iter()
                    .map(|x_offset| (*x_offset + x as i8) as usize)
                    .collect::<Vec<_>>();
                let possible_y = offset.1.into_iter()
                    .map(|y_offset| (*y_offset + y as i8) as usize)
                    .collect::<Vec<_>>();

                for (index, val) in possible_x.iter().enumerate() {
                    arr[x][y].0[index] = *val;
                }

                for (index, val) in possible_y.iter().enumerate() {
                    arr[x][y].1[index] = *val;
                }
            }
        }
        arr
    };
}

const ALL :[Option<u8>; 9] = [Some(1), Some(2), Some(3), Some(4), Some(5), Some(6), Some(7), Some(8), Some(9)];



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

#[derive(Debug, Copy, Clone)]
struct SimpleSquare {
    value: u8,
    num_iteration: u32,
}

// For now expect that all boards have size 9
struct Board {
    //squares: [[Option<u8>;  9]; 9]
    squares: [Square; 81],
    is_finished: bool,
}

#[derive(Debug, Copy, Clone)]
struct BoardAsArray {
    squares: [[Option<SimpleSquare>;  9]; 9]
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

impl std::fmt::Display for BoardAsArray {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "").unwrap();
        for x in 0..9 {
            for y in 0..9 {
                let to_write = match self.squares[x][y] {
                    None => String::from(" "),
                    Some(value) => value.value.to_string()
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
    //let mut board = convert(board_response);
    let mut board = convertToBoardAsArray(board_response);
    println!("board = {}", board);
    solve2(&mut board.squares, 0);
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

fn convertToBoardAsArray(response: BoardResponse) -> BoardAsArray {
    let mut arr : [[Option<SimpleSquare>; 9]; 9] = [[None; 9]; 9];

    for square in response.squares {
        arr[square.x][square.y] = Some(SimpleSquare {
            value: square.value,
            num_iteration: 0,
        })
    }

    return BoardAsArray {
        squares: arr,
    }
}

fn solve2(squares: &mut [[Option<SimpleSquare>; 9]; 9], num_iteration: u32) -> bool {
    let mut all_possible_values: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    for x in 0..9 {
        for y in 0..9 {
            if squares[x][y].is_none() {
                let possible_values = find_value2(squares, x, y);
                all_possible_values.push((x, y, possible_values));
            }
        }
    }

    if all_possible_values.len() == 0 {
        return true;
    }

    let less_possible_values = all_possible_values.iter()
                .min_by_key(|value| value.2.len())
                .unwrap();

    if less_possible_values.2.len() == 1 {
        let x = less_possible_values.0;
        let y = less_possible_values.1;
        let value = less_possible_values.2[0];
        squares[x][y] = Some(SimpleSquare {
            value: value,
            num_iteration: num_iteration,
        });

        return solve2(squares, num_iteration);
    }

    let x = less_possible_values.0;
    let y = less_possible_values.1;
    let size = less_possible_values.2.len();
    for (index, possible_value) in less_possible_values.2.iter().enumerate() {
        let is_last = index == size - 1;
        let next_iteration = if is_last {num_iteration} else {num_iteration + 1};

         squares[x][y] = Some(SimpleSquare {
            value: *possible_value,
            num_iteration: next_iteration,
        });

        let is_finished = solve2(squares, next_iteration);

        if is_finished {
            return true;
        }

        for x in 0..9 {
            for y in 0..9 {
                if squares[x][y].is_some() &&  squares[x][y].unwrap().num_iteration == next_iteration {
                    squares[x][y] = None;

                }
            }
        }
    }

    return false;
}

fn remove_value(vector: &mut Vec<u8>, value: u8) {
    let index = vector.iter().position(|x| *x == value);
    if index.is_some() {
        vector.remove(index.unwrap());
    }
}

fn find_value2(squares: &mut [[Option<SimpleSquare>; 9]; 9], x: usize, y: usize) -> Vec<u8> {

    let mut miss_x : [Option<u8>; 9] = [None; 9];
    miss_x.copy_from_slice(&ALL);

    for y2 in 0..9 {
        let square = squares[x][y2];
        if square.is_some() {
            miss_x[(square.unwrap().value - 1) as usize] = None;
        }
    }

    //let mut miss_y = ALL.to_vec();
    for x2 in 0..9 {
        let square = squares[x2][y];
        if square.is_some() {
            //let value = square.unwrap().value;
            miss_x[(square.unwrap().value - 1) as usize] = None;
        }
    }



    let offset = OFFSET[x][y];
    //let mut miss_square = ALL.to_vec();
    //let mut subset : Vec<u8> = Vec::new();
    for x2 in offset.0.iter() {
        for y2 in offset.1.iter() {
            let square = squares[*x2][*y2];
            if square.is_some() {
                miss_x[(square.unwrap().value - 1) as usize] = None;
                //if !ALL.contains(value)
                //subset.push(square.unwrap().value)
            }
        }
    }

    //return miss_x.intersect(miss_y).intersect(miss_square);
    return miss_x.iter()
            .filter_map(|x| *x)
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
        let mut board = convertToBoardAsArray(board_response);

        time_test!();
        println!("board = {}", board);

        solve2(&mut board.squares, 0);
        println!("solved board = {}", board);
        //assert_eq!(result, board);
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

        let mut board = convertToBoardAsArray(board_response);
        time_test!();
        println!("board = {}", board);

        solve2(&mut board.squares, 0);
        println!("solved board = {}", board);
        //assert_eq!(result, board);
    }
}
