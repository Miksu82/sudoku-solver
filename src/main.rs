extern crate reqwest;
extern crate serde_json;
extern crate serde;

//use std::collections::HashMap;

use std::str::FromStr;
use std::fmt::Display;
use serde::{Deserialize, Deserializer};
use std::collections::HashSet;

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
    num_iteration: u8,
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


fn main() {
    println!("Sudoku solver");

    let board_response = make_request()
        .expect("Failed to get sudoku");
    let mut board = convert(board_response);
    println!("board = {}", board);
    solve(&mut board, true, 0);
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

fn solve(board: &mut Board, find_sure: bool, num_iteration: u8) {

    let mut squares = board.squares;
    let mut all_possible_values: Vec<(usize, usize, Vec<u8>)> = Vec::new();

    for square in squares.iter() {
        if square.value.is_none() {
            let possible_values = find_value(board, square.x, square.y);
            all_possible_values.push((square.x, square.y, possible_values.into_iter().collect()));
        }
    }

    if all_possible_values.len() == 0 {
        board.is_finished = true;
        return;
    }

    let less_possible_values = all_possible_values.iter()
                .min_by_key(|value| value.2.len())
                .cloned()
                .unwrap();

    if less_possible_values.2.len() == 1 {
        let x = less_possible_values.0;
        let y = less_possible_values.1;
        let value = less_possible_values.2[0];
        for square in squares.iter_mut() {
            if square.x == x && square.y == y {
                square.value = Some(value);
                square.num_iteration = num_iteration;
                break;
            }
        }
        board.squares = squares;
        solve(board, true, num_iteration);
        return;
    }

    let x = less_possible_values.0;
    let y = less_possible_values.1;
    let size = less_possible_values.2.len();
    for (index, possible_value) in less_possible_values.2.iter().enumerate() {
        let is_last = index == size - 1;
        let next_iteration = if is_last {num_iteration} else {num_iteration + 1};

        for square in squares.iter_mut() {
            if square.x == x && square.y == y {
                square.value = Some(*possible_value);
                square.num_iteration = next_iteration;
                break;
            }
        }
        board.squares = squares;
        solve(board, find_sure, next_iteration);

        if board.is_finished {
            break;
        }

        for square in squares.iter_mut() {
            if square.num_iteration == num_iteration {
                square.value = None;
                square.num_iteration = 0;
            }
        }
        board.squares = squares;
    }
}

fn find_value(board: &Board, x: usize, y: usize) -> HashSet<u8> {
    let miss_x = missing_x(board, x);
    let miss_y = missing_y(board, y);
    let miss_square = miss_square(board, x, y);

    let missing1 = miss_x
                    .intersection(&miss_y)
                    // Cloned?: https://stackoverflow.com/questions/32354947/type-issue-with-iterator-collect
                    .cloned()
                     // Is it possible to do multiple set intersections with collect()
                    .collect::<HashSet<_>>();
    let missing = missing1
                    .intersection(&miss_square)
                    .cloned()
                    .collect::<HashSet<_>>();
    return missing;
}


fn missing_x(board: &Board, x: usize) -> HashSet<u8> {

    let column = board.squares.into_iter()
        .filter(|square| square.x == x && square.value.is_some())
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.into_iter()
                .filter(|value| column.iter().find(|c| c == value).is_none())
                .cloned()
                .collect::<HashSet<_>>();
}

fn missing_y(board: &Board, y: usize) -> HashSet<u8> {

    let column = board.squares.into_iter()
        .filter(|square| square.y == y && square.value.is_some())
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.into_iter()
                .filter(|value| column.iter().find(|c| c == value).is_none())
                .cloned()
                .collect::<HashSet<_>>();
}

fn miss_square(board: &Board, x: usize, y: usize) -> HashSet<u8> {
    let offset = get_offset(x, y);
    let possible_x = offset.0.into_iter()
        .map(|x_offset| (*x_offset + x as i8) as usize)
        .collect::<Vec<_>>();
    let possible_y = offset.1.into_iter()
        .map(|y_offset| (*y_offset + y as i8) as usize)
        .collect::<Vec<_>>();

    let subset = board.squares.into_iter()
        .filter(|square| {
            possible_x.contains(&square.x) && possible_y.contains(&square.y) && square.value.is_some()
        })
        .map(|square| square.value.unwrap())
        .collect::<Vec<_>>();

    return ALL.into_iter()
                .filter(|value| subset.iter().find(|c| c == value).is_none())
                .cloned()
                .collect::<HashSet<_>>();
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
