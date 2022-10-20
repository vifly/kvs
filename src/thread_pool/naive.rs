use std::thread;

use super::{Result, ThreadPool};

/// This isn't a real thread pool. DO NOT USE IN PRODUCTION.
pub struct NaiveThreadPool {}

impl ThreadPool for NaiveThreadPool {
    fn new(threads: usize) -> Result<Self> {
        assert!(threads > 0);
        Ok(NaiveThreadPool {})
    }

    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        let _handle = thread::spawn(move || { job(); });
    }
}
