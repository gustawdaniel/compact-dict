#![feature(portable_simd)]

pub mod dict;

#[cfg(test)]
mod tests {
    use crate::dict::Dict;
    use super::*;

    #[test]
    fn it_works() {
        let mut dict: Dict<u32> = dict::Dict::new(2);
        dict.put("a", 3);
        assert_eq!(dict.get_or("a", 0), 3);
        assert_eq!(dict.get_or("b", 0), 0);
        assert_eq!(dict.len(), 1);
        assert_eq!(dict.contains("a"), true);
        assert_eq!(dict.contains("b"), false);
    }
}
