use super::{Result, ThreadPool};

pub struct RayonThreadPool {
    rayon_wrapper: rayon::ThreadPool,
}

impl ThreadPool for RayonThreadPool {
    fn new(threads: usize) -> Result<Self> {
        assert!(threads > 0);
        let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
        Ok(RayonThreadPool { rayon_wrapper: pool })
    }

    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        self.rayon_wrapper.spawn(job);
    }
}
