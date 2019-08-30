extern crate reqwest;
extern crate serde_json;
extern crate serde;

//use std::collections::HashMap;

use std::str::FromStr;
use std::fmt::Display;
use serde::{Deserialize, Deserializer};

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

    let board_response = make_request()
        .expect("Failed to get sudoku");
    //println!("body = {:#?}", board_response);
    let mut board = convert(board_response);
    println!("board = {}", board);
    solve3(&mut board);
    println!("solved board = {}", board);
}

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

fn solve3(board: &mut Board) {
    loop {
        let mut squares = board.squares;
        for square in squares.iter_mut() {
            if square.value.is_none() {
                square.value = find_value(board, square.x, square.y);
                break;
            }
        }
        board.squares = squares
    }
}

fn find_value(board: &Board, x: usize, y: usize) -> Option<u8> {
    let miss_x = missing_x(board, x);
    let miss_y = missing_y(board, y);
    let miss_square = miss_square(board, x, y);
    println!("missing x={}, y={}: {:?}", x, y, miss_square);
    return Some(8)
}


fn missing_x(board: &Board, x: usize) -> Vec<&u8> {

    let column = board.squares.into_iter()
        .filter(|square| square.x == x && square.value.is_some())
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.into_iter()
                .filter(|value| column.iter().find(|c| c == value).is_none())
                .collect::<Vec<_>>();
}

fn missing_y(board: &Board, y: usize) -> Vec<&u8> {

    let column = board.squares.into_iter()
        .filter(|square| square.y == y && square.value.is_some())
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.into_iter()
                .filter(|value| column.iter().find(|c| c == value).is_none())
                .collect::<Vec<_>>();
}

fn miss_square(board: &Board, x: usize, y: usize) -> Vec<&u8> {
    let offset = get_offset(x, y);
    let possible_x = offset.0.into_iter()
        .map(|x_offset| (*x_offset + x as i8) as usize)
        .collect::<Vec<_>>();
    let possible_y = offset.1.into_iter()
        .map(|y_offset| (*y_offset + y as i8) as usize)
        .collect::<Vec<_>>();

    // println!("offset {:?}", offset);
    // println!("possible_x {:?}", possible_x);
    // println!("possible_y {:?}", possible_y);

    let subset = board.squares.into_iter()
        .filter(|square| {
            possible_x.contains(&square.x) && possible_y.contains(&square.y) && square.value.is_some()
        })
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.into_iter()
                .filter(|value| subset.iter().find(|c| c == value).is_none())
                .collect::<Vec<_>>();
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
