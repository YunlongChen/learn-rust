pub fn test_slice() {
    test_string_slice();
}

fn test_string_slice() {
    let mut s = String::from("broadcast");

    let part1 = &s[0..5];
    let part2 = &s[5..9];

    println!("{}={}+{}", s, part1, part2);

    s.push_str("test_value");

    let mut _value = "testValue";

    println!("{}", _value.to_owned() + "123123");

    // 快速的将String转换成str
    let mut s1 = String::from("hello");

    s1.push_str("12312");

    let s2 = &s1[..];

    println!("查看s1的容量：{}", s1.capacity());
    println!("快速的将String转换成str：{}", s2);
}
