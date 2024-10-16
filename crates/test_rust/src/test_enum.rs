#[derive(Debug)]
enum Book {
    Papery,
    Electronic,
    Other,
}

/// 测试枚举类
pub fn test_enum() {
    let book = Book::Papery;
    println!("测试枚举类：{:?}", book);

    let book = Book::Electronic;
    println!("测试枚举类：{:?}", book);

    let book = Book::Other;
    println!("测试其他枚举类：{:?}", book);

    let result = match book {
        Book::Papery => {
            println!("这里是第一类Book");
            1
        }
        Book::Electronic => {
            println!("这里是第二类Book");
            2
        }
        _ => {
            println!("以外情况");
            4
        }
    };

    println!("match的结果：{}", result);

    let opt = Some("Hello");

    match opt {
        None => {
            println!("有值")
        }
        Some(_) => {
            println!("这里什么都没有！")
        }
    }
}
