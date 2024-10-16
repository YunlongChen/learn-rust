extern crate concurrency;

use concurrency::test_thread_join;
use test_thread::test_thread;

mod test_thread;

fn main() {
    println!("这里是并发测试库");
    test_thread();
    test_thread_join();
    test_thread();
}
