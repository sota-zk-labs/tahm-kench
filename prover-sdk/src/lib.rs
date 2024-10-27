use std::fs;

use aligned_sdk::core::types::{Network, PriceEstimate, ProvingSystemId, VerificationData};
use aligned_sdk::sdk::{estimate_fee, get_next_nonce, submit_and_wait_verification};
use aligned_sp1_prover::AuctionData;
use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use ecies::public_key::PublicKey;
use ecies::Ecies;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::core::rand::rngs::OsRng;
use ethers::prelude::Signer;
use ethers::signers::Wallet;
use ethers::types::{Address, U256};
use sp1_sdk::{ProverClient, SP1Stdin};

pub const ELF: &[u8] = include_bytes!("../../sp1-prover/elf/riscv32im-succinct-zkvm-elf");
pub const ENCRYPTION_PUBLIC_KEY: &str = include_str!("../../sp1-prover/encryption_key");

/// Return winner and proof for the function `revealWinner` in the contract
pub async fn get_winner_and_submit_proof(
    wallet: Wallet<SigningKey>,
    auction_data: &AuctionData,
    rpc_url: &str,
    network: Network,
    batcher_url: &str,
) -> Result<(Address, u128, Vec<u8>)> {
    let mut stdin = SP1Stdin::new();
    stdin.write(auction_data);

    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);

    println!("Creating proof...");
    let mut proof = client.prove(&pk, stdin).compressed().run()?;
    println!("Proof created successfully");

    client.verify(&proof, &vk)?;

    let pub_input = proof.public_values.to_vec();
    let _hash_data = proof.public_values.read::<[u8; 32]>().to_vec(); // hash(auctionData)
    let winner_addr = proof.public_values.read::<Vec<u8>>(); // winner address
    let winner_amount = proof.public_values.read::<u128>(); // winner amount

    let proof = bincode::serialize(&proof).expect("Failed to serialize proof");

    // let proof = include_bytes!("../proof").to_vec();
    // let pub_input = include_bytes!("../pub_input").to_vec();
    // let winner_addr = vec![];
    // let winner_amount = 0; // winner amount
    dbg!(proof.len());

    fs::write("proof", &proof).expect("Failed to write proof to file");
    fs::write("pub_input", &pub_input).expect("Failed to write pub_input to file");
    let verification_data = VerificationData {
        proving_system: ProvingSystemId::SP1,
        proof,
        proof_generator_addr: wallet.address(),
        vm_program_code: Some(ELF.to_vec()),
        verification_key: None,
        pub_input: Some(pub_input.clone()),
    };
    let max_fee = estimate_fee(rpc_url, PriceEstimate::Instant)
        .await
        .expect("failed to fetch gas price from the blockchain");

    #[cfg(not(test))]
    let max_fee_string = ethers::utils::format_units(max_fee, 18)?;

    #[cfg(not(test))]
    if !Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(format!("Aligned will use at most {max_fee_string} eth to verify your proof. Do you want to continue?"))
        .interact()
        .expect("Failed to read user input") {
        return Err(anyhow!(""))
    }

    let nonce = get_next_nonce(rpc_url, wallet.address(), network)
        .await
        .expect("Failed to get next nonce");

    println!("Submitting your proof...");

    let aligned_verification_data = submit_and_wait_verification(
        batcher_url,
        rpc_url,
        network,
        &verification_data,
        max_fee,
        wallet,
        nonce,
    )
    .await
    .unwrap();

    println!(
        "Proof submitted and verified successfully on batch {}",
        hex::encode(aligned_verification_data.batch_merkle_root)
    );

    let mut index_in_batch = [0; 32];
    U256::from(aligned_verification_data.index_in_batch).to_big_endian(&mut index_in_batch);

    let merkle_path = aligned_verification_data
        .batch_inclusion_proof
        .merkle_path
        .as_slice()
        .iter()
        .flatten();

    let mut verified_proof = pub_input;
    verified_proof.extend(
        aligned_verification_data
            .verification_data_commitment
            .proof_commitment,
    );
    verified_proof.extend(
        aligned_verification_data
            .verification_data_commitment
            .pub_input_commitment,
    );
    verified_proof.extend(
        aligned_verification_data
            .verification_data_commitment
            .proving_system_aux_data_commitment,
    );
    verified_proof.extend(
        aligned_verification_data
            .verification_data_commitment
            .proof_generator_addr,
    );
    verified_proof.extend(aligned_verification_data.batch_merkle_root);
    verified_proof.extend(merkle_path);
    verified_proof.extend(index_in_batch);

    Ok((
        Address::from_slice(&winner_addr),
        winner_amount,
        verified_proof,
    ))
}

pub fn encrypt_bidder_amount(amount: &u128, scheme: &Ecies, pbk: &PublicKey) -> Vec<u8> {
    scheme.encrypt(&mut OsRng, pbk, &amount.to_be_bytes())
}

pub fn get_encryption_key() -> Result<PublicKey> {
    Ok(PublicKey::from_bytes(hex::decode(ENCRYPTION_PUBLIC_KEY)?))
}

#[cfg(test)]
mod tests {
    use crate::{encrypt_bidder_amount, get_encryption_key};
    use aligned_sdk::core::types::Network;
    use aligned_sp1_prover::{AuctionData, Bidder};
    use ecies::private_key::PrivateKey;
    use ecies::Ecies;
    use ethers::core::rand::rngs::OsRng;
    use ethers::prelude::Signer;
    use ethers::signers::LocalWallet;
    use ethers::types::{Bytes, H160};
    use sp1_sdk::{ProverClient, SP1Stdin};
    use std::fs::File;
    use std::io::Read;
    use std::str::FromStr;
    use std::{env, fs};

    #[tokio::test]
    async fn test_submit_proof() {
        let rpc_url = "https://ethereum-holesky-rpc.publicnode.com";
        let network = Network::Holesky;
        let batcher_url = "wss://batcher.alignedlayer.com";
        let wallet = LocalWallet::from_str(&env::var("PRIVATE_KEY").unwrap())
            .unwrap()
            .with_chain_id(17000u64);

        let (_winner_addr, winner_amount, verified_proof) = super::get_winner_and_submit_proof(
            wallet,
            &auction_data(),
            rpc_url,
            network,
            batcher_url,
        )
        .await
        .unwrap();
        dbg!(winner_amount);
        fs::write("verified_proof", &verified_proof).unwrap();
    }

    #[test]
    fn test_sp1_prover() {
        let elf = {
            let mut buffer = Vec::new();
            File::open("../sp1-prover/elf/riscv32im-succinct-zkvm-elf")
                .unwrap()
                .read_to_end(&mut buffer)
                .unwrap();
            buffer
        };

        // let mut rng = rand::thread_rng();

        let mut stdin = SP1Stdin::new();
        stdin.write(&auction_data());

        let client = ProverClient::new();
        let (pk, vk) = client.setup(elf.as_slice());

        let Ok(mut proof) = client.prove(&pk, stdin).compressed().run() else {
            println!("Something went wrong!");
            return;
        };
        
        println!("Proof generated successfully. Verifying proof...");
        client.verify(&proof, &vk).expect("verification failed");
        println!("Proof verified successfully.");
        
        // println!("{:?}", proof.public_values);
        let hash_data = proof.public_values.read::<[u8; 32]>();
        dbg!(hash_data);
        let winner_addr = proof.public_values.read::<Vec<u8>>();
        dbg!(winner_addr);
        let winner_price = proof.public_values.read::<u128>();
        dbg!(winner_price);
        // Todo: validate with data
    }

    fn auction_data() -> AuctionData {
        let pbk = get_encryption_key().unwrap();

        let scheme = Ecies::from_pvk(PrivateKey::from_rng(&mut OsRng));
        AuctionData {
            bidders: vec![
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&3, &scheme, &pbk),
                    address: hex::decode("eDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap(),
                },
                Bidder {
                    encrypted_amount: encrypt_bidder_amount(&2, &scheme, &pbk),
                    address: hex::decode("eDe4C2b4BdBE580750a99F016b0A1581C3808FA3").unwrap(),
                },
            ],
            id: vec![0; 32],
        }
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
    }
}
