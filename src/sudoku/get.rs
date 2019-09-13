use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;
use super::{SUDOKU_SIZE, Sudoku, Cells, Cell};

#[derive(Deserialize, Debug)]
struct CellResponse {
    x: usize,
    y: usize,
    value: u8,
}

#[derive(Deserialize, Debug)]
struct SudokuResponse {
    response: bool,
    #[serde(deserialize_with = "from_str")]
    size: u8,
    squares: Vec<CellResponse>,
}

pub enum Difficulty {
    Easy =1,
    Medium = 2,
    Hard = 3,
}

pub fn create_sudoku(level: Difficulty) -> Result<Sudoku, String>  {
    return match make_request(level) {
        Ok(response) =>  Ok(convert(response)),
        Err(e) => Err(e.to_string())
    };
}


fn convert(response: SudokuResponse) -> Sudoku {
    let mut cells: Cells = [[None; SUDOKU_SIZE]; SUDOKU_SIZE];

    for cell in response.squares {
        cells[cell.x][cell.y] = Some(Cell {
            value: cell.value,
            num_iteration: 0,
        })
    }

    return Sudoku { cells: cells };
}

fn make_request(level: Difficulty) -> Result<SudokuResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let level_string = (level as u8).to_string();
    let query_params = [("size", SUDOKU_SIZE.to_string()), ("level", level_string)];
    let json = client
        .get("http://www.cs.utep.edu/cheon/ws/sudoku/new")
        .query(&query_params)
        .send()?
        .json()?;
    return Ok(json);
}

// I don't know what this does. Taken from https://github.com/serde-rs/json/issues/317
fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}
