struct ResourceConfig<T> {
    list: Box<dyn Fn() -> Vec<T>>,
    search: Box<dyn Fn(String) -> Vec<T>>,
    new: Box<dyn Fn() -> T>,

    show: Box<dyn Fn(String) -> T>,
    edit: Box<dyn Fn(String) -> T>,
    delete: Box<dyn Fn(String) -> T>,
}

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
