pub struct PublicKey {
    pub key: secp256k1::PublicKey
}

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            key: secp256k1::PublicKey::from_slice(bytes).unwrap()
        }
    }

    // pub fn from_point(px: u128, py: u128) -> Self {
    //     let mut key = px.to_be_bytes().to_vec();
    //     key.extend(&py.to_be_bytes());
    //     Self { key, px, py }
    // }

    pub fn to_bytes(&self) -> [u8; 65] {
        self.key.serialize_uncompressed()
    }

    // pub fn to_point(&self, space: &MontgomerySpace) -> Point {
    //     Point::new(space.new_mont(&self.px), space.new_mont(&self.py))
    // }
}
