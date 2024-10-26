use rand::Rng;
use tiny_keccak::{Hasher, Keccak};
use crate::symmetric_encryption::scheme::SymmetricEncryptionScheme;

pub struct SymmetricKey(Vec<u8>);
pub struct SimpleSE {
    key: SymmetricKey
}

impl SymmetricEncryptionScheme for SimpleSE {
    fn new(key: Vec<u8>) -> Self {
        Self {
            key: SymmetricKey(key)
        }
    }

    fn encrypt(&self, rng: &mut impl Rng, plaintext: &[u8]) -> Vec<u8> {
        let mut nonce = [0; 16];
        rng.fill_bytes(&mut nonce);
        let mut r = self.calc_r(&nonce, plaintext.len());
        let mut res = nonce.to_vec();
        for i in 0..plaintext.len() {
            r[i] = (((r[i] as u16) + (plaintext[i] as u16)) % 256) as u8;
        }
        res.extend(&r);
        res
        // plaintext.to_vec()
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Vec<u8> {
        let nonce = &ciphertext[..16];
        let encrypted_text = &ciphertext[16..];
        let r = self.calc_r(nonce, encrypted_text.len());
        let res = encrypted_text.iter().zip(r.iter()).map(|(a, b)| {
            (((*a as i16) - (*b as i16) + 256) % 256) as u8
        }).collect();
        res
        // ciphertext.to_vec()
    }
}

impl SimpleSE {
    fn calc_r(&self, nonce: &[u8], mut n: usize) -> Vec<u8> {
        let mut hasher = Keccak::v256();
        hasher.update(nonce);
        hasher.update(&self.key.0);
        let mut a = [0; 32];
        hasher.finalize(&mut a);
        let mut res = vec![0; n];
        loop {
            let mut hasher = Keccak::v256();
            hasher.update(&a);
            hasher.update(&n.to_be_bytes());
            if n < 32 {
                hasher.finalize(&mut res[..n]);
                break;
            } else {
                hasher.finalize(&mut res[n - 32..n]);
            }
            n -= 32;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::symmetric_encryption::scheme::SymmetricEncryptionScheme;
    use crate::symmetric_encryption::simple::SimpleSE;

    #[test]
    fn test_simple_se() {
        let g = SimpleSE::new(vec![1, 2, 3, 4, 5]);
        let t = g.encrypt(&mut rand::thread_rng(), b"hello world");
        let p = g.decrypt(&t);
        assert_eq!(p, b"hello world");
    }
}