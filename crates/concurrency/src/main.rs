extern crate concurrency;

use test_thread::test_thread;

mod test_thread;
mod test_thread_join;

fn main() {
    println!("这里是并发测试库");
    test_thread();
}
