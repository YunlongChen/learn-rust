extern crate concurrency;

use crate::ownership::test_mod_ownership;
use crate::test_enum::test_enum;
use crate::test_slice::test_slice;
use std::fs;

// 下面两个方法他们的定义方式和定义层级都是一样的，但是支持两种不同的引入方式
use concurrency::test_thread::{test_thread};
use concurrency::{test_thread2};

mod ownership;
mod test_enum;
mod test_slice;
mod test_struct;


// 这里是一个main函数
fn main() {
    println!("Hello, world!");

    let username = concurrency::add(1, 2);
    println!("这里是测试用户名：{}", username);

    test_thread2();

    test_thread();

    let mut a: i64 = 5 + 10;
    a += 1;
    a += 1;
    print!("{}", a);

    let bool_value: bool = true;

    println!("{}", bool_value);

    // 这里是一个备注信息
    let username: String = "陈云龙的值".to_string();

    println!("{}", username);

    let user = String::from("1515");

    let slice = &user[..];

    println!("经过切片的名称是：{}", slice);

    print_username(username);

    let value = another_function();

    println!("{}", value);

    let _ok = test_block();

    test_condition();

    test_loop();

    test_ownership();

    println!("测试线程并发编程");
    test_thread();

    println!("测试多模块文件");
    test_mod_ownership();

    test_reference();

    test_borrow();

    test_borrow_and_change();

    test_slice();

    println!("测试结构体");
    test_struct::test_struct();

    println!("测试枚举类");
    test_enum();

    // 测试文件读取
    let string = fs::read_to_string("hello.html").unwrap();
    println!("文件内容：{}", string);
}

fn print_username(username: String) {
    println!("{}", username);
}

fn another_function() -> i64 {
    println!("Hello, runoob!");
    123
}

fn test_block() -> i64 {
    let x = 5;

    let y = {
        let x = 3;
        x + 1
    };

    println!("x 的值为 : {}", x);
    println!("y 的值为 : {}", y);
    y
}

fn test_condition() -> i64 {
    let x = 5;

    let y;
    if x > 5 && x != 1 {
        y = 152;
        println!("计算y的值！{}", y);
    } else {
        y = {
            let x = 3;
            x + 1
        };
    }

    println!("x 的值为 : {}", x);
    println!("y 的值为 : {}", y);
    y
}

/// while循环
/// 测试注释
/// 备注信息
///
///
///
fn test_loop() {
    // while循环
    let mut number = 1;
    while number != 4 {
        println!("while循环：第{}轮", number);
        number += 1;
    }
    println!("结束while循环");
    // 不支持do-while

    // 测试单行注释
    let a = [10, 20, 30, 40, 50];

    for x in a {
        println!("第{}个值", x);
    }

    let b = [10, 20, 30, 40, 50];

    for x in b.iter() {
        println!("{}", x);
    }

    println!("通过下标访问");
    let c = [10, 20, 30, 40, 50];

    for x in 0..5 {
        println!("{}", c[x]);
    }

    // loop循环，rust原生的无限循环
    let s = ['R', 'U', 'N', 'O', 'O', 'B'];
    let mut index = 0;
    let location = loop {
        let ch = s[index];
        if ch == 'O' {
            break index;
        }
        index += 1;
    };

    println!("\'O\'的下标是：{}", location);
}

/// 测试所有权
/// 当变量超出范围时，Rust 自动调用释放资源函数并清理该变量的堆内存。但是 s1 和 s2 都被释放的话堆区中的 "hello" 被释放两次，这是不被系统允许的。
/// 为了确保安全，在给 s2 赋值时 s1 已经无效了。没错，在把 s1 的值赋给 s2 以后 s1 将不可以再被使用。所以下面这段程序是错的：
fn test_ownership() {
    let s1 = String::from("测试用户姓名");

    let s2 = s1.clone();

    println!("{}", s1);
    println!("{}", s2);
}

///
/// 引用本身也是一个类型并具有一个值，这个值记录的是别的值所在的位置，但引用不具有所指值的所有权：
fn test_reference() {
    let s1 = String::from("hello");
    let s2 = &s1;
    // 当一个变量的值被引用时，变量本身不会被认定无效。因为"引用"并没有在栈中复制变量的值：所有上面两个变量都是“有效值”
    println!("s1 is {}, s2 is {}", s1, s2);
}

///  既然引用不具有所有权，即使它租借了所有权，它也只享有使用权（这跟租房子是一个道理）。
///
/// 如果尝试利用租借来的权利来修改数据会被阻止：
/// 当前也存在一中关系，如果五月规定房主可以修改房子结构，房租在租借是也在合同中申明赋予你这样的权力，你是可以重新装修房子的
/// 就是在编程的时候你需要签一个合同
fn test_borrow() {
    let s1 = String::from("hello world!");

    println!("s1 is {s1}, s2 is {}", s1);
    let s2 = &s1;

    // 下面这段程序不正确：因为 s2 租借的 s1 已经将所有权移动到 s3，所以 s2 将无法继续租借使用 s1 的所有权。如果需要使用 s2 使用该值，必须重新租借：
    println!("{}", s2);

    println!("s1 is {}, s2 is {}", s1, s2);
}

/// 租借并且修改
fn test_borrow_and_change() {
    let mut s1 = String::from("hello world!");

    println!("s1 is {s1}, s2 is {}", s1);
    let s2 = &mut s1;

    s2.push_str("添加了一个");

    // 下面这段程序不正确：因为 s2 租借的 s1 已经将所有权移动到 s3，所以 s2 将无法继续租借使用 s1 的所有权。如果需要使用 s2 使用该值，必须重新租借：
    println!("{}", s2);

    println!("s2 is {}", s2);

    println!("原始的s1：{}", s1);

    let reference_to_nothing = dangle();

    println!("{}", reference_to_nothing);
}

fn dangle() -> String {
    // s的生命周期被移除函数，转移到外层的reference_to_nothing上面去了！
    let _s = String::from("hello");
    _s
}
