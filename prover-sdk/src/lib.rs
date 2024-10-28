use std::fs;

use aligned_sdk::core::types::{Network, PriceEstimate, ProvingSystemId, VerificationData};
use aligned_sdk::sdk::{estimate_fee, get_next_nonce, submit_and_wait_verification};
use aligned_sp1_prover::AuctionData;
use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use ecies::PublicKey;
use ethers::abi::{encode, Token, Uint};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::Signer;
use ethers::signers::Wallet;
use ethers::types::{Address, U256};
use sp1_sdk::{ProverClient, SP1Stdin};

pub const ELF: &[u8] = include_bytes!("../../sp1-prover/elf/riscv32im-succinct-zkvm-elf");
pub const ENCRYPTION_PUBLIC_KEY: &str = include_str!("../../sp1-prover/encryption_key");
pub const ENCRYPTION_PRIVATE_KEY: &str = include_str!("../../sp1-prover/private_encryption_key");

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
    stdin.write(&hex::decode(ENCRYPTION_PRIVATE_KEY)?);

    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);

    println!("Creating proof...");
    let mut proof = client.prove(&pk, stdin).compressed().run()?;
    println!("Proof created successfully");

    client.verify(&proof, &vk)?;

    let pub_input = proof.public_values.to_vec();
    let _hash_data = proof.public_values.read::<[u8; 32]>().to_vec(); // hash(auctionData)
    let winner_addr = Address::from_slice(proof.public_values.read::<Vec<u8>>().as_slice()); // winner address
    let winner_amount = proof.public_values.read::<u128>(); // winner amount

    let proof = bincode::serialize(&proof).expect("Failed to serialize proof");

    // let proof = include_bytes!("../proof").to_vec();
    // let pub_input = include_bytes!("../pub_input").to_vec();
    // println!("{:?}", hex::encode(&pub_input));
    // let winner_addr = vec![1u8];
    // let winner_amount = 0; // winner amount
    // // dbg!(proof.len());
    // let hash = [202u8, 36, 143, 90, 16, 137, 94, 111, 213, 3, 201, 186, 171, 70, 43, 164, 32, 123, 86, 217, 241, 250, 209, 191, 120, 60, 15, 217, 120, 122, 228, 86];
    // let addr = [237u8, 228, 194, 180, 189, 190, 88, 7, 80, 169, 159, 1, 107, 10, 21, 129, 195, 128, 143, 163];
    // let amount = 3u128;
    // let g = encode(&[Token::FixedBytes(hash.to_vec()), Token::Address(Address::from(addr)), Token::Uint(Uint::from(amount))]);
    // println!("{}", hex::encode(hash));
    // println!("{}", hex::encode(addr));
    // println!("{:?}", g);

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

    let merkle_path: Vec<u8> = flatten(
        aligned_verification_data
            .batch_inclusion_proof
            .merkle_path
            .as_slice(),
    );

    let verified_proof = encode(&[
        Token::Bytes(pub_input),
        Token::FixedBytes(
            aligned_verification_data
                .verification_data_commitment
                .proof_commitment
                .to_vec(),
        ),
        Token::FixedBytes(
            aligned_verification_data
                .verification_data_commitment
                .pub_input_commitment
                .to_vec(),
        ),
        Token::FixedBytes(
            aligned_verification_data
                .verification_data_commitment
                .proving_system_aux_data_commitment
                .to_vec(),
        ),
        Token::FixedBytes(
            aligned_verification_data
                .verification_data_commitment
                .proof_generator_addr
                .to_vec(),
        ),
        Token::FixedBytes(aligned_verification_data.batch_merkle_root.to_vec()),
        Token::Bytes(merkle_path),
        Token::Uint(Uint::from(index_in_batch)),
    ]);

    fs::write("verified_proof", &verified_proof)
        .expect("Failed to write verified proof to file");

    Ok((winner_addr, winner_amount, verified_proof))
}

pub fn encrypt_bidder_amount(amount: &u128, pbk: &PublicKey) -> Vec<u8> {
    ecies::encrypt(&pbk.serialize(), &amount.to_be_bytes()).expect("failed to encrypt bidder data")
}

pub fn get_encryption_key() -> Result<PublicKey> {
    Ok(
        PublicKey::parse((*hex::decode(ENCRYPTION_PUBLIC_KEY).unwrap()).try_into()?)
            .expect("parsing encryption public key failed"),
    )
}

pub fn flatten(vec: &[[u8; 32]]) -> Vec<u8> {
    let mut res = vec![];
    for v in vec.iter() {
        res.extend_from_slice(v);
    }
    res
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::env;

    use aligned_sdk::core::types::Network;
    use aligned_sp1_prover::{AuctionData, Bidder};
    use ecies::PublicKey;
    use ethers::prelude::Signer;
    use ethers::signers::LocalWallet;
    use ethers::types::{Bytes, H160};

    use crate::{encrypt_bidder_amount, ENCRYPTION_PUBLIC_KEY};

    #[tokio::test]
    async fn test_submit_proof() {
        let rpc_url = "https://ethereum-holesky-rpc.publicnode.com";
        let network = Network::Holesky;
        let batcher_url = "wss://batcher.alignedlayer.com";
        let wallet = LocalWallet::from_str(&env::var("PRIVATE_KEY").unwrap())
            .unwrap()
            .with_chain_id(17000u64);
        let pbk = PublicKey::parse(
            (*hex::decode(ENCRYPTION_PUBLIC_KEY).unwrap())
                .try_into()
                .unwrap(),
        )
        .unwrap();

        let data = AuctionData {
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
        };

        let (_winner_addr, winner_amount, _verified_proof) =
            super::get_winner_and_submit_proof(wallet, &data, rpc_url, network, batcher_url)
                .await
                .unwrap();
        dbg!(winner_amount);
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

        let g = include_bytes!("../pub_input");
        println!("{:?}", g);
    }
}
