use rand::Rng;

pub trait SymmetricEncryptionScheme {
    fn new(key: Vec<u8>) -> Self;
    fn encrypt(&self, rng: &mut impl Rng, plaintext: &[u8]) -> Vec<u8>;
    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8>;
}