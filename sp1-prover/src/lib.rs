use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};
use ecies::Ecies;
use ecies::private_key::PrivateKey;
use ecies::symmetric_encryption::simple::SimpleSE;

#[derive(Deserialize, Serialize, Debug)]
pub struct AuctionData {
    pub bidders: Vec<Bidder>,
    pub id: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Bidder {
    pub encrypted_amount: Vec<u8>,
    pub address: Vec<u8>,
}

#[sp1_derive::cycle_tracker]
pub fn decrypt_bidder_data(scheme: &Ecies, bidder: &Bidder) -> u128 {
    u128::from_be_bytes(
        scheme.decrypt(&bidder.encrypted_amount).try_into().unwrap()
    )
}

#[sp1_derive::cycle_tracker]
pub fn calc_auction_hash(auction_data: &AuctionData) -> [u8; 32] {
    let mut input = vec![];
    let mut hasher = Keccak::v256();

    input.extend(&auction_data.id);
    for bidder in &auction_data.bidders {
        input.extend(&bidder.address);
        input.extend(&bidder.encrypted_amount);
        println!("{:?}", bidder.encrypted_amount);

    }

    let mut output = [0u8; 32];
    hasher.update(&input);
    hasher.finalize(&mut output);
    output
}

pub fn find_winner(auction_data: &AuctionData, pvk: PrivateKey) -> (Vec<u8>, u128) {
    let scheme = Ecies::<SimpleSE>::from_pvk(pvk);

    let mut winner_addr = &vec![];
    let mut winner_amount = 0;
    for bidder in &auction_data.bidders {
        println!("cycle-tracker-start: decrypt-bidder-data");
        let bidder_amount = decrypt_bidder_data(&scheme, bidder);
        println!("cycle-tracker-end: decrypt-bidder-data");
        if winner_amount < bidder_amount {
            winner_amount = bidder_amount;
            winner_addr = &bidder.address;
        }
    }

    (winner_addr.clone(), winner_amount)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ecies::{Ecies};
    use ecies::private_key::PrivateKey;
    use ecies::public_key::PublicKey;
    use rand::rngs::OsRng;

    use crate::{calc_auction_hash, AuctionData, Bidder};

    const PVK_HEX: &str = include_str!("../private_encryption_key");

    #[test]
    fn test_decrypt_data() {
        let (pvk, pbk) = get_key();
        let scheme = Ecies::from_pvk(pvk);
        // let mut rng = rand::thread_rng();
        let bidder = Bidder {
            encrypted_amount: encrypt_bidder_amount(&(1e23 as u128), &pbk),
            address: vec![0; 32],
        };
        let amount = super::decrypt_bidder_data(&scheme, &bidder);
        assert_eq!(amount, 1e23 as u128);
    }

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
        let (_, pbk) = get_key();
        let data = auction_data(&pbk);
        println!("{:?}",data);
        // println!("{:?}", hex::encode(&data.bidders[0].address));
        // println!("{:?}", hex::encode(&data.bidders[1].encrypted_amount));
        println!("{:?}", hex::encode(calc_auction_hash(&data)));
    }

    fn encrypt_bidder_amount(amount: &u128, pbk: &PublicKey) -> Vec<u8> {
        let scheme: Ecies = Ecies::from_pvk(PrivateKey::from_rng(&mut OsRng));
        scheme.encrypt(&mut OsRng, pbk, &amount.to_be_bytes())
    }

    pub fn get_key() -> (PrivateKey, PublicKey) {
        let pvk = PrivateKey::from_hex(PVK_HEX);
        let pbk = pvk.to_public_key();
        (pvk, pbk)
    }

    fn auction_data(pbk: &PublicKey) -> AuctionData {
        AuctionData {
            bidders: vec![
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&3, pbk),
                    address: hex::decode("eDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap(),
                },
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&2, pbk),
                    address: hex::decode("eDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap(),
                },
            ],
            id: vec![0; 32],
        }
    }
}
