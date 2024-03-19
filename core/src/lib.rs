uniffi::include_scaffolding!("vvcore");

pub fn my_rusty_add(left: u64, right: u64) -> u64 {
    left + right
}

pub enum MyEnum {
    A,
    B,
    C,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = my_rusty_add(2, 2);
        assert_eq!(result, 4);
    }
}
