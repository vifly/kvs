use std::path::PathBuf;

use sled::IVec;

use crate::{KvsEngine, Result};

#[derive(Clone)]
pub struct SledKvsEngine {
    db: sled::Db,
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<SledKvsEngine> {
        let db = sled::open(&path.into())?;
        Ok(SledKvsEngine { db })
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.db.insert(IVec::from(key.as_str()), IVec::from(value.as_str()))?;
        self.db.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let res = self.db.get(IVec::from(key.as_str()))?;
        match res {
            Some(val) => Ok(Some(String::from_utf8_lossy(val.to_vec().as_slice()).to_string())),
            None => Ok(None)
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        self.db.remove(IVec::from(key.as_str()))?;
        self.db.flush()?;
        Ok(())
    }
}