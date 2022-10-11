use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::Result;

pub trait KvsEngine: Clone + Send + 'static {
    // Set the value of a string key to a string.
    fn set(&self, key: String, value: String) -> Result<()>;
    // Get the string value of a string key.
    fn get(&self, key: String) -> Result<Option<String>>;
    // Remove a given string key.
    fn remove(&self, key: String) -> Result<()>;
}

pub fn get_engine_name(path: impl Into<PathBuf>) -> Result<Option<String>> {
    let path = path.into().join("engine");
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        return Ok(Some(contents));
    }
    Ok(None)
}

pub fn write_engine(engine: &str, path: impl Into<PathBuf>) -> Result<()> {
    let path = path.into().join("engine");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(path)?;
    file.write_all(engine.as_bytes())?;

    Ok(())
}