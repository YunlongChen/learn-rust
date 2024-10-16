use std::thread;
use std::time::Duration;

pub fn test_thread_join() {
    let mut username = 1;

    let thread = thread::spawn(move || {
        for sid in 1..5 {
            println!("hi number {sid} from the spawned thread!");
            thread::sleep(Duration::from_millis(1));
            println!("当前的username的值：{username}");
            username += 1;
        }
        println!("当前的username的值：{username}");
    });

    // 如果在主线程之前join，会在执行完子线程的任务指标，才执行主线程的任务
    // thread.join().unwrap();

    for i in 1..10 {
        println!("hi number {i} from the main thread!");
        thread::sleep(Duration::from_millis(1));
    }

    // 主线程会等到子线程的任务执行完毕之后再结束
    thread.join().unwrap();
    println!("测试任务执行完毕，username的最终结果是{}!", username);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = test_thread_join();
        println!("{:?}", result);
    }

    #[test]
    fn it_works2() {
        let result = test_thread_join();
        println!("{:?}", result);
    }
}