use std::thread;
use std::time::Duration;


///
/// 测试rust多线程编程
///
/// main函数中，主线程和子线程交替执行，主线程先执行，然后子线程执行，最后主线程执行
///
pub fn test_thread() {
    thread::spawn(|| {
        for sid in 1..5 {
            println!("hi number {sid} from the spawned thread!");
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..5 {
        println!("hi number {i} from the main thread!");
        thread::sleep(Duration::from_millis(1));
    }
}


/// 测试rust多线程编程
///
/// main函数中，主线程和子线程交替执行，主线程先执行，然后子线程执行，最后主线程执行
///
pub fn test_thread2() {
    thread::spawn(|| {
        for i in 1..10 {
            println!("hi number {} from the spawned thread!", i);
            thread::sleep(Duration::from_secs(1));
        }
    });

    for i in 1..5 {
        println!("hi number {} from the main thread!", i);
        thread::sleep(Duration::from_millis(1));
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = test_thread();
        println!("{:?}", result);
    }

    #[test]
    fn test_work2() {
        let result = test_thread2();
        println!("{:?}", result);
    }
}