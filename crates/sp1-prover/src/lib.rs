use ecies::Ecies;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

#[derive(Deserialize, Serialize, Debug)]
pub struct AuctionData {
    pub bidders: Vec<Bidder>,
    pub id: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Bidder {
    pub encrypted_amount: Vec<u8>,
    pub address: Vec<u8>,
}

/// Decrypt the encrypted bidder data using owner's private key
///
/// # Arguments
///
/// * `scheme`: ECIES scheme
/// * `bidder`: bidder's data
///
/// returns: u128 Bidder's amount
pub fn decrypt_bidder_data(scheme: &Ecies, bidder: &Bidder) -> u128 {
    u128::from_be_bytes(scheme.decrypt(&bidder.encrypted_amount).try_into().unwrap())
}

/// Calculate the hash of the auction data to ensure the integrity of the data
///
/// # Arguments
///
/// * `auction_data`: Data of the auction
///
/// returns: [u8; 32] hash(data) in bytes
pub fn calc_auction_hash(auction_data: &AuctionData) -> [u8; 32] {
    let mut input = vec![];
    let mut hasher = Keccak::v256();

    input.extend(&auction_data.id);
    for bidder in &auction_data.bidders {
        input.extend(&bidder.address);
        input.extend(&bidder.encrypted_amount);
    }

    let mut output = [0u8; 32];
    hasher.update(&input);
    hasher.finalize(&mut output);
    output
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ecies::private_key::PrivateKey;
    use rand::rngs::OsRng;

    use crate::{calc_auction_hash, AuctionData, Bidder};

    #[test]
    fn test_gen_key() {
        let mut rng = OsRng;
        let pvk = PrivateKey::from_rng(&mut rng);
        let pbk = pvk.to_public_key();
        let pvk = hex::encode(pvk.to_bytes());
        let pbk = hex::encode(pbk.to_bytes());
        println!("Private key: {}", &pvk);
        println!("Public key: {}", &pbk);
        fs::write("private_encryption_key", pvk).expect("failed to write private key to file");
        fs::write("encryption_key", pbk).expect("failed to write public key to file");
    }

    #[test]
    fn test_hash_auction() {
        let data = AuctionData {
            bidders: vec![
                Bidder {
                    encrypted_amount: vec![
                        4, 36, 76, 235, 221, 48, 100, 119, 65, 250, 235, 5, 239, 222, 110, 182,
                        196, 66, 147, 29, 250, 89, 160, 63, 120, 239, 240, 253, 94, 78, 33, 188,
                        195, 75, 141, 42, 28, 254, 170, 66, 121, 72, 236, 121, 34, 99, 139, 9, 67,
                        182, 123, 254, 145, 255, 126, 180, 165, 90, 99, 195, 102, 162, 233, 234,
                        105, 23, 212, 34, 133, 236, 42, 142, 159, 79, 230, 27, 94, 198, 99, 212,
                        62, 163, 201, 140, 172, 197, 150, 68, 146, 121, 231, 197, 42, 210, 146, 21,
                        198, 175, 0, 60, 119, 39, 22, 163, 219, 169, 137, 88, 176, 34, 11, 241, 59,
                    ],
                    address: vec![
                        237, 228, 194, 180, 189, 190, 88, 7, 80, 169, 159, 1, 107, 10, 21, 129,
                        195, 128, 143, 163,
                    ],
                },
                Bidder {
                    encrypted_amount: vec![
                        4, 135, 60, 218, 68, 89, 15, 3, 52, 54, 251, 189, 250, 82, 153, 236, 7, 21,
                        206, 164, 136, 176, 7, 18, 49, 124, 131, 25, 117, 42, 7, 93, 254, 222, 55,
                        197, 231, 144, 218, 110, 106, 201, 198, 195, 219, 167, 63, 26, 5, 212, 144,
                        112, 71, 94, 37, 130, 104, 155, 229, 253, 202, 30, 31, 179, 100, 166, 77,
                        55, 229, 125, 201, 182, 218, 104, 73, 29, 177, 108, 130, 149, 155, 130,
                        175, 14, 107, 74, 229, 220, 17, 67, 121, 146, 196, 153, 254, 38, 23, 50,
                        92, 100, 214, 119, 215, 201, 89, 185, 26, 67, 235, 156, 61, 65, 250,
                    ],
                    address: vec![
                        237, 228, 194, 180, 189, 190, 88, 7, 80, 169, 159, 1, 107, 10, 21, 129,
                        195, 128, 143, 163,
                    ],
                },
            ],
            id: vec![0; 32],
        };
        assert_eq!(
            hex::encode(calc_auction_hash(&data)),
            "eec15d5f07c54d6bc722494c90d0404960ec255907c62e8928e2969a186a2350"
        );
    }
}
