#![feature(duration_constructors)]
extern crate core;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs};

use aligned_sp1_prover::AuctionData;
use anyhow::Result;
use ecies::{PublicKey, SecretKey};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::types::{Address, Bytes};
use sp1_sdk::network::proto::network::ProofMode;
use sp1_sdk::{HashableKey, NetworkProverV1, Prover, SP1Stdin};

/// Return winner and proof for the function `revealWinner` in the contract
///
/// # Arguments
///
/// * `wallet`: wallet of the owner
/// * `auction_data`: data of the auction
/// * `rpc_url`: rpc url of the network
/// * `network`: network supported by Aligned
/// * `batcher_url`: Aligned batcher URL
///
/// returns: Result<(H160, u128, Vec<u8, Global>), Error> (winner address, winner amount, verified proof)
pub async fn find_winner(
    auction_data: &AuctionData,
    wallet_private_key: &SigningKey,
) -> Result<(Address, u128, Bytes, Bytes)> {
    println!("Creating proof...");

    // setup sp1 prover environment variables
    env::set_var(
        "SP1_PRIVATE_KEY",
        hex::encode(wallet_private_key.to_bytes()),
    );

    let mut stdin = SP1Stdin::new();
    stdin.write(auction_data);
    stdin.write(&get_private_encryption_key()?.serialize().to_vec());

    let client = NetworkProverV1::new();

    let elf = &get_elf()?;
    let (_pk, vk) = client.setup(elf);
    let mut proof = client
        .prove(elf, stdin, ProofMode::Plonk, Some(Duration::from_hours(1)))
        .await?;

    println!("Proof created successfully");

    client.verify(&proof, &vk)?;

    let _hash_data = proof.public_values.read::<[u8; 32]>().to_vec(); // hash(auctionData)
    let winner_addr = Address::from_slice(proof.public_values.read::<Vec<u8>>().as_slice()); // winner address
    let winner_amount = proof.public_values.read::<u128>(); // winner amount

    fs::write("public_values", proof.public_values.as_slice())?;
    fs::write("verifying_key", vk.bytes32())?;
    fs::write("proof", proof.bytes())?;
    Ok((
        winner_addr,
        winner_amount,
        Bytes::from(proof.public_values.to_vec()),
        Bytes::from(proof.bytes()),
    ))
}

/// Encrypts the amount of a bidder using the public key of the owner
///
/// # Arguments
///
/// * `amount`: bid amount
/// * `pbk`: public key of the owner
///
/// returns: Vec<u8, Global> encrypted amount
pub fn encrypt_bidder_amount(amount: &u128, pbk: &PublicKey) -> Vec<u8> {
    ecies::encrypt(&pbk.serialize(), &amount.to_be_bytes()).expect("failed to encrypt bidder data")
}

/// Get the public encryption key of the owner
pub fn get_encryption_key() -> Result<PublicKey> {
    Ok(PublicKey::parse(
        &hex::decode(fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sp1-prover/encryption_key"),
        )?)?
        .try_into()
        .unwrap(),
    )
    .expect("parsing public encryption key failed"))
}

/// Get the private encryption key of the owner
pub fn get_private_encryption_key() -> Result<SecretKey> {
    Ok(SecretKey::parse_slice(&hex::decode(fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sp1-prover/private_encryption_key"),
    )?)?)
    .expect("parsing private encryption key failed"))
}

/// Get the ELF file that was compiled with the SP1 prover
pub fn get_elf() -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    File::open(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../sp1-prover//elf/riscv32im-succinct-zkvm-elf"),
    )?
    .read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Flatten a 2D array into a 1D array
///
/// # Arguments
///
/// * `vec`: 2D array
///
/// returns: Vec<u8, Global> Flatten array
pub fn flatten(vec: &[[u8; 32]]) -> Vec<u8> {
    let mut res = vec![];
    for v in vec.iter() {
        res.extend_from_slice(v);
    }
    res
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::str::FromStr;

    use aligned_sp1_prover::{AuctionData, Bidder};
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::types::{Bytes, H160};
    use sp1_sdk::{ProverClient, SP1Stdin};

    use crate::{encrypt_bidder_amount, get_encryption_key, get_private_encryption_key};

    #[tokio::test]
    async fn test_submit_proof() {
        let pvk =
            SigningKey::from_bytes(hex::decode("PRIVATE_KEY").unwrap().as_slice().into()).unwrap();
        let (_, winner_amount, _, _) = super::find_winner(&auction_data(), &pvk).await.unwrap();
        dbg!(winner_amount);
    }

    #[test]
    fn test_sp1_prover() {
        // find_winner(&auction_data(), PrivateKey::from_bytes(hex::decode(ENCRYPTION_PRIVATE_KEY).unwrap()));
        let elf = {
            let mut buffer = Vec::new();
            File::open("../sp1-prover/elf/riscv32im-succinct-zkvm-elf")
                .unwrap()
                .read_to_end(&mut buffer)
                .unwrap();
            buffer
        };

        let mut stdin = SP1Stdin::new();
        stdin.write(&auction_data());
        stdin.write(&get_private_encryption_key().unwrap().serialize().to_vec());

        let client = ProverClient::new();
        let (pk, vk) = client.setup(elf.as_slice());

        println!("Generating proof...");
        let Ok(mut proof) = client.prove(&pk, stdin).compressed().run() else {
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
    fn test_type() {
        let x = H160::from_str("0xeDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap();
        assert_eq!(
            hex::encode(x.as_fixed_bytes()),
            "ede4c2b4bdbe580750a99f016b0a1581c3808fa3".to_string()
        );

        let y = Bytes::from(vec![1, 2, 3]);
        assert_eq!(y.to_vec(), vec![1, 2, 3]);

        println!("{:?}", hex::encode(include_bytes!("../public_values")));
    }

    fn auction_data() -> AuctionData {
        let pbk = get_encryption_key().unwrap();

        AuctionData {
            bidders: vec![
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&3, &pbk),
                    address: hex::decode("eDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap(),
                },
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&2, &pbk),
                    address: hex::decode("eDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap(),
                },
            ],
            id: vec![0; 32],
        }
    }
}
