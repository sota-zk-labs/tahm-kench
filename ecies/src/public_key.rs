use curve25519_dalek::edwards::CompressedEdwardsY;
use curve25519_dalek::EdwardsPoint;

pub struct PublicKey {
    key: EdwardsPoint,
    bytes: [u8; 32]
}

impl PublicKey {
    pub const SIZE_IN_BYTES: usize = 32;
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self {
            key: CompressedEdwardsY(bytes).decompress().unwrap(),
            bytes
        }
    }
    
    pub fn from_point(point: EdwardsPoint) -> Self {
        Self {
            key: point,
            bytes: point.compress().0,
        }
    }

    // pub fn from_point(px: u128, py: u128) -> Self {
    //     let mut key = px.to_be_bytes().to_vec();
    //     key.extend(&py.to_be_bytes());
    //     Self { key, px, py }
    // }

    pub fn to_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
        
    pub fn point(&self) -> &EdwardsPoint {
        &self.key
    }
}
