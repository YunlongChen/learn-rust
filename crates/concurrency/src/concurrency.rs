///
/// # 这里是一个add方法
///
/// # Example
///
///  ```
///  use concurrency::add;
///  let result = add(2, 2);
/// ```
///
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
