use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions, remove_file, rename};
use std::io::{Read, Seek, SeekFrom, Write};
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::{KvsEngine, KvsError, Result};

const COMPACT_NUM_THRESHOLD: usize = 512;

#[derive(Serialize, Deserialize, Debug)]
pub struct LogPosition {
    start: usize,
    len: usize,
}

#[derive(Serialize, Deserialize, Debug)]
enum LogEntry {
    Set { key: String, value: String },
    Rm { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaData {
    store_path: PathBuf,
    cur_file_end: usize,
    since_last_compact_log_num: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct MutableKvsData {
    metadata: MetaData,
    store_map: HashMap<String, LogPosition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KvStore {
    data: Arc<Mutex<MutableKvsData>>,
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

fn rebuild_map(log_entry_path: impl Into<PathBuf>) -> Result<HashMap<String, LogPosition>> {
    let mut reader = BufReaderWithPos::new(File::open(log_entry_path.into())?)?;
    let mut pos = reader.seek(SeekFrom::Start(0))? as usize;
    let mut stream = Deserializer::from_reader(reader).into_iter::<LogEntry>();
    let mut result: HashMap<String, LogPosition> = HashMap::new();
    while let Some(log_entry) = stream.next() {
        let new_pos = stream.byte_offset() as usize;
        match log_entry? {
            LogEntry::Set { key, .. } => {
                let log_position = LogPosition { start: pos, len: new_pos - pos };
                result.insert(key, log_position);
            }
            LogEntry::Rm { key } => {
                result.remove(&key);
            }
        };
        pos = new_pos;
    }
    Ok(result)
}

impl KvStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let store_map = HashMap::new();
        let path_buf = path.into();

        let metadata = MetaData {
            store_path: path_buf,
            cur_file_end: 0,
            since_last_compact_log_num: 0,
        };
        let data = Arc::new(Mutex::new(MutableKvsData {
            metadata,
            store_map,
        }));
        KvStore {
            data
        }
    }

    pub fn new_with_data(metadata: MetaData, store_map: HashMap<String, LogPosition>) -> KvStore {
        let data = Arc::new(Mutex::new(MutableKvsData {
            metadata,
            store_map,
        }));
        KvStore {
            data
        }
    }

    fn save_memory_map(&self) -> Result<()> {
        self.data.lock().unwrap().save_memory_map()
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        let kvs_metadata_path = path.join("kvs_metadata");
        let log_entry_path = path.join("kvs_log_entry");
        let memory_map_path = path.join("kvs_memory_map");
        if !kvs_metadata_path.exists() {
            if !path.exists() {
                create_dir_all(&path)?;
            }
            File::create(&kvs_metadata_path)?;
            Ok(KvStore::new(path))
        } else {
            let mut file = File::open(kvs_metadata_path)?;
            if file.metadata()?.len() == 0 {
                return Ok(KvStore::new(path));
            }
            let mut metadata_contents = String::new();
            file.read_to_string(&mut metadata_contents)?;
            let metadata: MetaData = serde_json::from_str(&metadata_contents)?;

            let mut store_map = HashMap::new();
            if memory_map_path.exists() {
                let mut store_map_file = File::open(memory_map_path)?;
                if store_map_file.metadata().unwrap().len() != 0 {
                    let mut store_map_contents = String::new();
                    store_map_file.read_to_string(&mut store_map_contents)?;
                    store_map = serde_json::from_str(&store_map_contents)?;
                }
            } else if log_entry_path.exists() {
                store_map = rebuild_map(log_entry_path)?;
            }

            let kvs: KvStore = KvStore::new_with_data(metadata, store_map);

            let log_entry_file = File::open(path.join("kvs_log_entry"))?;
            let file_size = log_entry_file.metadata()?.len();
            if file_size as usize != kvs.data.lock().unwrap().metadata.cur_file_end {
                return Err(KvsError::RecordError());
            }
            Ok(kvs)
        }
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        self.save_memory_map().unwrap();
    }
}

impl KvsEngine for KvStore {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.data.lock().unwrap().set(key, value)?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        self.data.lock().unwrap().get(key)
    }

    fn remove(&self, key: String) -> Result<()> {
        self.data.lock().unwrap().remove(key)?;
        Ok(())
    }
}

impl MutableKvsData {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let log_entry = LogEntry::Set {
            key: key.clone(),
            value,
        };
        let log_pos = self.save_log_entry(&log_entry)?;
        self.store_map.insert(key, log_pos);
        self.save_metadata()?;

        if self.metadata.since_last_compact_log_num > COMPACT_NUM_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        if self.store_map.contains_key(&key) {
            let log_pos = self.store_map.get(&key).unwrap();
            let mut file = File::open(&self.metadata.store_path.join("kvs_log_entry"))?;
            file.seek(SeekFrom::Start(log_pos.start as u64))?;
            let mut buf = Vec::with_capacity(log_pos.len);
            file.take(log_pos.len as u64).read_to_end(&mut buf)?;
            let log_entry: LogEntry = serde_json::from_slice(&buf)?;
            match log_entry {
                LogEntry::Set { key: _, value } => Ok(Some(value)),
                _ => Err(KvsError::Unknown),
            }
        } else {
            Ok(None)
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if !self.store_map.contains_key(&key) {
            return Err(KvsError::KeyNotFound(key));
        }
        self.store_map.remove(&key);
        let log_entry = LogEntry::Rm { key };
        self.save_log_entry(&log_entry)?;
        self.save_metadata()?;

        if self.metadata.since_last_compact_log_num > COMPACT_NUM_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    fn save_log_entry(&mut self, log_entry: &LogEntry) -> Result<LogPosition> {
        let mut store_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.metadata.store_path.join("kvs_log_entry"))?;
        let serialized_log = serde_json::to_vec(&log_entry)?;
        store_file.write_all(&serialized_log)?;

        let log_pos = LogPosition {
            start: self.metadata.cur_file_end,
            len: serialized_log.len(),
        };
        self.metadata.cur_file_end += serialized_log.len();
        self.metadata.since_last_compact_log_num += 1;

        Ok(log_pos)
    }

    fn save_metadata(&self) -> Result<()> {
        let serialized_kvs = serde_json::to_string(&self.metadata)?;
        let mut file = File::create(self.metadata.store_path.join("kvs_metadata"))?;
        file.write_all(serialized_kvs.as_bytes())?;

        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        self.metadata.cur_file_end = 0;
        self.metadata.since_last_compact_log_num = 0;
        let mut new_store_map: HashMap<String, LogPosition> = HashMap::new();

        let mut all_serialized_log: Vec<u8> = vec![];
        for key_pos in self.store_map.iter() {
            let key = key_pos.0;
            let value = self.get(key.clone())?.unwrap_or_else(|| "".to_string());
            let log_entry = LogEntry::Set {
                key: key.clone(),
                value,
            };

            let serialized_log = serde_json::to_vec(&log_entry)?;
            all_serialized_log.extend(serialized_log.iter().cloned());

            let log_pos = LogPosition {
                start: self.metadata.cur_file_end,
                len: serialized_log.len(),
            };
            self.metadata.cur_file_end += serialized_log.len();

            new_store_map.insert(key.clone(), log_pos);
        }

        self.write_new_log_entry_file(&all_serialized_log)?;
        self.store_map = new_store_map;
        self.save_metadata()?;

        Ok(())
    }

    fn write_new_log_entry_file(&self, content: &Vec<u8>) -> Result<()> {
        let mut store_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.metadata.store_path.join("kvs_log_entry.new"))?;
        store_file.write_all(content)?;

        rename(
            &self.metadata.store_path.join("kvs_log_entry"),
            &self.metadata.store_path.join("kvs_log_entry.bak"),
        )?;
        rename(
            &self.metadata.store_path.join("kvs_log_entry.new"),
            &self.metadata.store_path.join("kvs_log_entry"),
        )?;
        remove_file(&self.metadata.store_path.join("kvs_log_entry.bak"))?;

        Ok(())
    }

    fn save_memory_map(&self) -> Result<()> {
        let serialized_kvs = serde_json::to_string(&self.store_map)?;
        let mut file = File::create(self.metadata.store_path.join("kvs_memory_map"))?;
        file.write(serialized_kvs.as_bytes())?;

        Ok(())
    }
}
