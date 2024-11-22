pub struct PublicKey {
    pub key: secp256k1::PublicKey
}

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            key: secp256k1::PublicKey::from_slice(bytes).unwrap()
        }
    }

    pub fn to_bytes(&self) -> [u8; 65] {
        self.key.serialize_uncompressed()
    }
}
