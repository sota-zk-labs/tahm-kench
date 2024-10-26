use crate::utils::math::{Montgomery, MontgomerySpace};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Point {
    pub x: Montgomery,
    pub y: Montgomery,
    pub is_none: bool,
}

impl Point {
    pub fn new(x: Montgomery, y: Montgomery) -> Self {
        Self {
            x,
            y,
            is_none: false,
        }
    }

    pub fn none_point(space: &MontgomerySpace) -> Self {
        Self {
            x: space.zero(),
            y: space.zero(),
            is_none: true,
        }
    }
    
    pub fn to_norm(&self, space: &MontgomerySpace) -> (u128, u128) {
        (space.to_norm(&self.x), space.to_norm(&self.y))
    }
    
    pub fn to_bytes(&self, space: &MontgomerySpace) -> Vec<u8> {
        let (x, y) = self.to_norm(space);
        let mut bytes = x.to_be_bytes().to_vec();
        bytes.extend(y.to_be_bytes());
        bytes
    }
}