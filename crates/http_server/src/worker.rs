use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug)]
pub struct Worker {
    pub id: usize,
    // Option<T> 是一个枚举，它表示一个值要么是 Some(T)，要么是 None。
    pub thread: Option<JoinHandle<()>>,
}
pub type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    
    ///
    /// 创建一个 Worker 实例。
    /// id 线程序号
    ///
    pub(crate) fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        println!("创建一个线程，线程Id：{}", id);
        let thread_handle = thread::spawn(move || {
            loop {
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
            }
        });
        Worker {
            id,
            thread: Some(thread_handle),
        }
    }
}
