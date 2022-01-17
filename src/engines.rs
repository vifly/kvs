use crate::Result;

pub trait KvsEngine {
    // Set the value of a string key to a string.
    fn set(&mut self, key: String, value: String) -> Result<()>;
    // Get the string value of a string key.
    fn get(&self, key: String) -> Result<Option<String>>;
    // Remove a given string key.
    fn remove(&mut self, key: String) -> Result<()>;
} 
