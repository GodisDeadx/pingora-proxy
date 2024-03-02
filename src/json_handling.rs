use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn get_path(filename: &str) -> PathBuf {
    let mut path = home_dir().unwrap();
    path.push(".nebula");
    path.push(filename);
    path
}

pub fn read_file() -> Result<String, ()> {
    let mut path = get_path("config.json");

    return Ok("die".to_string());
}
