mod error;
mod engines;
mod server;
mod client;

use crate::error::KvsError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::fs::{create_dir_all, remove_file, rename, File, OpenOptions};
use std::path::PathBuf;
pub use engines::KvsEngine;
pub use server::KvsServer;
pub use client::KvsClient;

pub type Result<T> = std::result::Result<T, KvsError>;

const COMPACT_NUM_THRESHOLD: usize = 512;

#[derive(Serialize, Deserialize, Debug)]
struct LogPosition {
    start: usize,
    len: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KvStore {
    store_map: HashMap<String, LogPosition>,
    store_path: PathBuf,
    cur_file_end: usize,
    since_last_compact_log_num: usize,
}

#[derive(Serialize, Deserialize, Debug)]
enum LogEntry {
    Set { key: String, value: String },
    Rm { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    is_ok: bool,
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

impl Response {
    pub fn new(is_ok: bool, data: String) -> Response {
        Response { is_ok, data }
    }
}

impl KvStore {
    pub fn new(path: impl Into<PathBuf>, cur_file_end: usize) -> KvStore {
        let since_last_compact_log_num = 0;
        KvStore {
            store_map: HashMap::new(),
            store_path: path.into(),
            cur_file_end,
            since_last_compact_log_num,
        }
    }

    pub fn is_key_exist(&self, key: &str) -> bool {
        return self.store_map.contains_key(key);
    }

    fn save_log_entry(&mut self, log_entry: &LogEntry) -> Result<LogPosition> {
        let mut store_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.store_path.join("kvs_log_entry"))?;
        let serialized_log = serde_json::to_vec(&log_entry)?;
        store_file.write(&serialized_log)?;

        let log_pos = LogPosition {
            start: self.cur_file_end,
            len: serialized_log.len(),
        };
        self.cur_file_end = self.cur_file_end + serialized_log.len();
        self.since_last_compact_log_num = self.since_last_compact_log_num + 1;

        Ok(log_pos)
    }

    fn save_index(&self) -> Result<()> {
        let serialized_kvs = serde_json::to_string(&self)?;
        let mut file = File::create(self.store_path.join("kvs_index"))?;
        file.write(serialized_kvs.as_bytes())?;

        Ok(())
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let log_entry = LogEntry::Set {
            key: key.clone(),
            value,
        };
        let log_pos = self.save_log_entry(&log_entry)?;
        self.store_map.insert(key, log_pos);
        self.save_index()?;

        if self.since_last_compact_log_num > COMPACT_NUM_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        if self.store_map.contains_key(&key) {
            let log_pos = self.store_map.get(&key).unwrap();
            let mut file = File::open(&self.store_path.join("kvs_log_entry"))?;
            file.seek(SeekFrom::Start(log_pos.start as u64))?;
            let mut buf = Vec::with_capacity(log_pos.len);
            file.take(log_pos.len as u64).read_to_end(&mut buf)?;
            let log_entry: LogEntry = serde_json::from_slice(&buf)?;
            return match log_entry {
                LogEntry::Set { key: _, value } => Ok(Some(value)),
                _ => Err(KvsError::Unknown),
            };
        } else {
            return Ok(None);
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.store_map.contains_key(&key) == false {
            return Err(KvsError::KeyNotFound(key));
        }
        self.store_map.remove(&key);
        let log_entry = LogEntry::Rm { key };
        self.save_log_entry(&log_entry)?;
        self.save_index()?;

        if self.since_last_compact_log_num > COMPACT_NUM_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        let kvs_index_path = path.join("kvs_index");
        if kvs_index_path.exists() == false {
            if path.exists() == false {
                create_dir_all(&path)?;
            }
            File::create(&kvs_index_path)?;
            return Ok(KvStore::new(path, 0));
        } else {
            let mut file = File::open(kvs_index_path)?;
            if file.metadata()?.len() == 0 {
                return Ok(KvStore::new(path, 0));
            }
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let kvs: KvStore = serde_json::from_str(&contents)?;

            let store_file = File::open(path.join("kvs_log_entry"))?;
            let file_size = store_file.metadata()?.len();
            if file_size as usize != kvs.cur_file_end {
                return Err(KvsError::RecordError());
            }
            return Ok(kvs);
        }
    }

    fn write_new_log_entry_file(&mut self, content: &Vec<u8>) -> Result<()> {
        let mut store_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.store_path.join("kvs_log_entry.new"))?;
        store_file.write(content)?;

        rename(
            &self.store_path.join("kvs_log_entry"),
            &self.store_path.join("kvs_log_entry.bak"),
        )?;
        rename(
            &self.store_path.join("kvs_log_entry.new"),
            &self.store_path.join("kvs_log_entry"),
        )?;
        remove_file(&self.store_path.join("kvs_log_entry.bak"))?;

        Ok(())
    }

    pub fn compact(&mut self) -> Result<()> {
        self.cur_file_end = 0;
        self.since_last_compact_log_num = 0;
        let mut new_store_map: HashMap<String, LogPosition> = HashMap::new();

        let mut all_serialized_log: Vec<u8> = vec![];
        for key_pos in self.store_map.iter() {
            let key = key_pos.0;
            let value = self.get(key.clone())?.unwrap_or_else(|| "".to_string());
            let log_entry = LogEntry::Set {
                key: key.clone(),
                value: value.clone(),
            };

            let serialized_log = serde_json::to_vec(&log_entry)?;
            all_serialized_log.extend(serialized_log.iter().cloned());

            let log_pos = LogPosition {
                start: self.cur_file_end,
                len: serialized_log.len(),
            };
            self.cur_file_end = self.cur_file_end + serialized_log.len();

            new_store_map.insert(key.clone(), log_pos);
        }

        self.write_new_log_entry_file(&all_serialized_log)?;
        self.store_map = new_store_map;
        self.save_index()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
