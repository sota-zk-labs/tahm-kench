use crate::ecc::point::Point;
use crate::utils::math::MontgomerySpace;

pub struct PublicKey {
    key: Vec<u8>,
    px: u128,
    py: u128,
}

impl PublicKey {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let m = bytes.len() >> 1;
        let px = u128::from_be_bytes(bytes[0..m].try_into().unwrap());
        let py = u128::from_be_bytes(bytes[m..m * 2].try_into().unwrap());
        Self {
            key: bytes,
            px,
            py,
        }
    }

    pub fn from_point(px: u128, py: u128) -> Self {
        let mut key = px.to_be_bytes().to_vec();
        key.extend(&py.to_be_bytes());
        Self { key, px, py }
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.key
    }

    pub fn to_point(&self, space: &MontgomerySpace) -> Point {
        Point::new(space.new_mont(&self.px), space.new_mont(&self.py))
    }
}
