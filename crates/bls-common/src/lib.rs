/// Declare common structs, serialization, deserialization and functions here;
/// the `bls-runtime-wasm` and `rust-sdk` will use this crate
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Products {
    pub page: u8,
    pub per_page: u8,
    pub total: u8,
    pub total_pages: u8,
    pub data: Vec<Product>,
    pub support: Support,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: u8,
    pub name: String,
    pub year: u16,
    pub color: String,
    pub pantone_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Support {
    pub url: String,
    pub text: String,
}

pub fn add(left: usize, right: usize) -> usize {
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
