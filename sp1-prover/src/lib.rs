use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};
use ecies::Ecies;

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

pub fn decrypt_bidder_data(scheme: &Ecies, bidder: &Bidder) -> u128 {
    u128::from_be_bytes(
        scheme.decrypt(bidder.encrypted_amount.as_slice()).try_into().unwrap()
    )
}

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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::process::Command;

    use ecies::{Ecies};
    use ecies::private_key::PrivateKey;
    use ecies::public_key::PublicKey;
    use rand::rngs::OsRng;
    // use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding};
    // use rsa::rand_core::OsRng;
    // use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

    use crate::{calc_auction_hash, AuctionData, Bidder};
    // use ecies_ed25519::generate_keypair;
    // use rand::{CryptoRng, RngCore};

    const PVK_HEX: &str = include_str!("../private_encryption_key");

    #[test]
    fn test_sp1_prover() {
        // compile main
        let output = Command::new("cargo")
            .args(["prove", "build"])
            .output()
            .unwrap();
        println!("{:?}", String::from_utf8_lossy(output.stdout.as_slice()));
        let elf = {
            let mut buffer = Vec::new();
            File::open("./elf/riscv32im-succinct-zkvm-elf")
                .unwrap()
                .read_to_end(&mut buffer)
                .unwrap();
            buffer
        };

        // let mut rng = rand::thread_rng();
        let (_, pbk) = get_key();

        let mut stdin = SP1Stdin::new();
        stdin.write(&auction_data(&pbk));

        let client = ProverClient::new();
        let (pk, vk) = client.setup(elf.as_slice());

        let Ok(mut proof) = client.prove(&pk, stdin).run() else {
            println!("Something went wrong!");
            return;
        };

        println!("Proof generated successfully. Verifying proof...");
        client.verify(&proof, &vk).expect("verification failed");
        println!("Proof verified successfully.");

        // println!("{:?}", proof.public_values);
        let hash_data = proof.public_values.read::<[u8; 32]>();
        println!("{:?}", hash_data);
        let winner_addr = proof.public_values.read::<Vec<u8>>();
        println!("{:?}", winner_addr);
        // Todo: validate with data
    }

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
