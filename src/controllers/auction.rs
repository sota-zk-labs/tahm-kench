use aligned_sdk::core::types::Network;
use aligned_sp1_prover::{AuctionData, Bidder};
use anyhow::{Context, Result};
use ecies::private_key::PrivateKey;
use ecies::{Ecies};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::core::rand::rngs::OsRng;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::prelude::{Http, LocalWallet, Provider};
use ethers::types::{Address, Bytes, U256};
use ecies::public_key::PublicKey;
use prover_sdk::{encrypt_bidder_amount, get_winner_and_submit_proof};

use crate::entities::auction::AuctionEntity;

abigen!(erc721Contract, "./assets/erc721.json");
abigen!(erc20Contracts, "./assets/erc20.json");
abigen!(zkAuctionContract, "./assets/ZkAuction.json");

pub async fn create_new_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    pbk_encryption: &PublicKey,
    name: String,
    description: String,
    nft_contract_address: Address,
    token_id: U256,
    target_price: U256,
    duration: U256,
) -> Result<()> {
    // Approve NFT
    let erc721_contract = erc721Contract::new(nft_contract_address, signer.clone().into());
    let erc721_contract_caller = erc721_contract.approve(auction_contract_address, token_id);
    let approve_tx = erc721_contract_caller.send().await?;
    let _ = approve_tx.await?.unwrap();
    println!("approve success");
    // Create Auction
    let zk_auction_contract = zkAuctionContract::new(auction_contract_address, signer.into());
    println!("1");
    let contract_caller = zk_auction_contract.create_auction(
        Bytes::from(pbk_encryption.to_bytes().to_vec()),
        nft_contract_address,
        token_id,
        name,
        description,
        target_price,
        duration,
    );
    println!("2");
    let tx = contract_caller.send().await?;
    println!("3");
    let receipt = tx.await?.unwrap();
    println!("4");
    let tx_hash = receipt.transaction_hash;
    println!(
        "Create auction successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

pub async fn get_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<AuctionEntity> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let auction = contract.auctions(auction_id).call().await?;

    Ok(AuctionEntity::from(auction))
}

pub async fn get_total_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
) -> Result<()> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let total = contract.auction_count().call().await?;
    println!("Auctions total: {:?}", total);
    Ok(())
}

// pub fn encrypt_price(bid_price: U256) -> Bytes {
//
// }

pub async fn create_bid(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    token_address: Address,
    auction_id: U256,
    bid_price: U256,
) -> Result<()> {
    let auction = get_auction(signer.clone(), auction_contract_address, auction_id).await?;
    // Approve token
    let erc20_contract = erc20Contracts::new(token_address, signer.clone().into());
    let erc20_contract_caller =
        erc20_contract.approve(auction_contract_address, auction.asset.token_id);
    let approve_tx = erc20_contract_caller.send().await?;
    approve_tx.await?.unwrap();

    let encryption_key = PublicKey::from_bytes(auction.encryption_key.to_vec());
    // Fake encrypted price
    let encrypted_price = encrypt_bidder_amount(
        &bid_price.as_u128(),
        &Ecies::from_pvk(PrivateKey::from_rng(&mut OsRng)),
        &encryption_key,
    );

    // Create bid
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.place_bid(auction_id, Bytes::from(encrypted_price));
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Create bid successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

pub async fn get_list_bids(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<Vec<Bidder>> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let bids = contract.get_bids(auction_id).call().await?;
    let list_bids: Vec<_> = bids
        .into_iter()
        .map(|element| Bidder {
            encrypted_amount: element.encrypted_price.to_vec(),
            address: element.bidder.as_fixed_bytes().to_vec(),
        })
        .collect();
    Ok(list_bids)
}

pub async fn reveal_winner(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    auction_id: U256,
    wallet: Wallet<SigningKey>,
    rpc_url: &str,
    network: Network,
    batcher_url: &str,
) -> Result<()> {
    // Get list bids
    let bidders = get_list_bids(signer.clone(), auction_contract_address, auction_id)
        .await
        .context(format!(
            "Failed to get list bids from auction with id: {}",
            auction_id
        ))?;
    //Send to SP1
    let mut auc_id = [0; 32];
    auction_id.to_big_endian(&mut auc_id);
    let (winner_addr, winner_amount, verified_proof) = get_winner_and_submit_proof(
        wallet,
        &AuctionData {
            bidders,
            id: auc_id.to_vec(),
        },
        rpc_url,
        network,
        batcher_url,
    )
    .await?;

    // // Submit proof to SMC
    // let winner = Winner{
    //     winner_address: Default::default(),
    // //     price: Default::default()
    // };

    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.finalize_auction(
        auction_id,
        Winner {
            winner: winner_addr,
            price: U256::from(winner_amount),
        },
        Bytes::from(verified_proof),
    );
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Reveal winner successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

pub async fn withdraw(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    auction_contract_address: Address,
    auction_id: U256,
) -> Result<()> {
    let contract = zkAuctionContract::new(auction_contract_address, signer.into());
    let contract_caller = contract.withdraw(auction_id);
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Withdraw deposit successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}
