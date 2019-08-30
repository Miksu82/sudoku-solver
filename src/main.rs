extern crate reqwest;
extern crate serde_json;
extern crate serde;

//use std::collections::HashMap;

use std::str::FromStr;
use std::fmt::Display;
use serde::{Deserialize, Deserializer};

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
}

// For now expect that all boards have size 9
struct Board {
    //squares: [[Option<u8>;  9]; 9]
    squares: [Square; 81],
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


fn main() {
    println!("Hello, world!");
    let f :u8 = 10;
    let s = f.to_string();

    let board_response = make_request()
        .expect("Failed to get sudoku");
    println!("body = {:#?}", board_response);
    let board = convert(board_response);
    println!("board = {}", board);
    //solve(&mut board);
    //println!("solved board = {}", board);
}

// fn convert(response: BoardResponse) -> Board {
//     let mut arr: [[Option<u8>; 9]; 9] = [[None; 9]; 9];

//     for square in response.squares.iter() {
//         arr[square.x][square.y] = Some(square.value)
//     }
//     return Board {
//         squares: arr
//     };
// }

fn convert(response: BoardResponse) -> Board {
    let empty_square = Square {
                x: 0,
                y: 0,
                value: None
    };

    let mut arr: [Square; 81] = [empty_square; 81];

    for x in 0..9 {
        for y in 0..9 {
            arr[9*x + y] = Square {
                x: x,
                y: y,
                value: None
            };
        }
    }

    for square in response.squares {
        arr[9*square.x + square.y]= Square {
                x: square.x,
                y: square.y,
                value: Some(square.value)
            };
    }

    return Board {
        squares: arr
    }
}

// fn clear_table(table: &mut [char; 9 ]) {
//     for field in table {
//         *field = ' ';
//     }
// }

// fn solve3(board: &mut Board) {
//     loop {
//         for square in &board.squares {

//         }
//     }
// }

// fn solve2(board: &mut Board) {
//     loop {
//         for row in &board.squares {
//             for maybe_value in row {
//                 if maybe_value.is_none() {
//                     *maybe_value = find_value(board, 1, 2)
//                 }
//             }
//         }
//     }
// }

// fn solve(board: &mut Board) {
//     loop {
//         for (x, row) in board.squares.iter().enumerate() {
//             for (y, maybe_value) in row.iter().enumerate() {
//                 if maybe_value.is_none() {
//                     *maybe_value = find_value(board, x, y)
//                 }
//             }
//         }
//     }
// }

// fn find_value(board: &Board, x: usize, y: usize) -> Option<u8> {
//     return Some(8)
// }

fn make_request() -> Result<BoardResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let json = client.get("http://www.cs.utep.edu/cheon/ws/sudoku/new")
                        .query(&[("size", "9")]) // What is &[()]
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
