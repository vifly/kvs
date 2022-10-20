use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

use super::{Job, Result, ThreadPool};

enum JobOrShutdown {
    Job(Job),
    Shutdown,
}

pub struct SharedQueueThreadPool {
    thread_num: usize,
    sender: mpsc::Sender<JobOrShutdown>,
}

#[derive(Clone)]
struct MyReceiver {
    receiver: Arc<Mutex<mpsc::Receiver<JobOrShutdown>>>,
}

fn spawn_in_pool(receiver: MyReceiver) {
    let _handle = thread::spawn(move || loop {
        let message = receiver.receiver.lock().expect("can't get lock").recv().unwrap();
        match message {
            JobOrShutdown::Job(job) => job(),
            JobOrShutdown::Shutdown => break
        }
    });
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: usize) -> Result<Self> {
        assert!(threads > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..threads {
            let thread_safety_receiver = MyReceiver { receiver: receiver.clone() };
            spawn_in_pool(thread_safety_receiver);
        }
        let thread_pool = SharedQueueThreadPool { thread_num: threads, sender };
        Ok(thread_pool)
    }

    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(job);
        let message = JobOrShutdown::Job(job);
        self.sender.send(message).unwrap();
    }
}


impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.thread_num {
            self.sender.send(JobOrShutdown::Shutdown).unwrap();
        }
    }
}

impl Drop for MyReceiver {
    fn drop(&mut self) {
        if thread::panicking() {
            spawn_in_pool(self.clone());
        }
    }
}
