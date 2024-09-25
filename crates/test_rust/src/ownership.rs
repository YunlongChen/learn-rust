pub fn test_mod_ownership() {
    let s1 = String::from("测试用户姓名");

    let _s2 = s1.clone();

    println!("{},s2:{}", s1, _s2);
}
