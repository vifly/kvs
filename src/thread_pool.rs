use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::{Result};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub trait ThreadPool {
    fn new(threads: u32) -> Result<Self> where Self: Sized;
    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static;
}

pub struct NaiveThreadPool {
    thread_num: u32,
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker {
    id: u32,
    handle: thread::JoinHandle<()>,
}

impl ThreadPool for NaiveThreadPool {
    fn new(threads: u32) -> Result<Self> {
        assert!(threads > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = vec![];
        for i in 0..threads {
            let worker = Worker::new(i, receiver.clone())?;
            workers.push(worker);
        }
        let thread_pool = NaiveThreadPool { thread_num: threads, workers, sender };
        Ok(thread_pool)
    }

    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(job);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: u32, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Self> {
        let handle = thread::spawn(move || loop {
            let job = receiver.lock().expect("can't get lock").recv().unwrap();
            job();
        });
        Ok(Worker { id, handle })
    }
}