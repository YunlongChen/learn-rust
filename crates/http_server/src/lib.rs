use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::thread;

pub struct Worker {
    id: usize,
    // Option<T> 是一个枚举，它表示一个值要么是 Some(T)，要么是 None。
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    ///
    /// 创建一个 Worker 实例。
    /// id 线程序号
    ///
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        dbg!("线程池{}开始监听", id);
        let thread_handle = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("线程池的Worker {} 收到任务，开始执行", id);
                    // 执行线程任务
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
            println!("线程池的Worker {} 任务执行完毕", id);
        });
        Worker {
            id,
            thread: Some(thread_handle),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

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
            println!("创建一个线程：线程Id：{}", id);
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
        println!("{}", self.workers.len());
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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
