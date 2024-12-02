use crate::worker::{Job, Worker};
use log::info;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    /// 创建线程池。
    ///
    /// 线程池中线程的数量。
    ///
    /// # Panics
    ///
    /// `new` 函数在 size 为 0 时会 panic。
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut threads = Vec::with_capacity(size);

        for id in 0..size {
            let work = Worker::new(id, Arc::clone(&receiver));
            threads.push(work);
        }
        ThreadPool {
            workers: threads,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        info!(
            "收到请求，正在进行处理：当前建立连接的数量{}",
            self.workers.len()
        );
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

///
/// 线程池的关闭
///
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
