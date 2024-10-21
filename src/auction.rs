use anyhow::Result;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::prelude::{Http, LocalWallet, Provider};
use ethers::types::{Address, Bytes, U256};
abigen!(zkAuctionContract, "./src/assets/ZkAuction.json");

pub async fn get_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
    id_auction: U256,
) -> Result<()> {
    let contract = zkAuctionContract::new(*contract_address, signer.into());
    let auction = contract.auctions(id_auction).call().await?;
    let (seller, pk, asset, winner, deposit_price, end_time, ended) = auction;
    println!("Auction Details:");
    println!("Name: {}", asset.name);
    println!("Seller: {:?}", seller);
    println!("Seller's public key: {:?}", pk);
    println!("Description: {}", asset.description);
    println!("Winner:");
    println!("  Address: {:?}", winner.winner);
    println!("  Encrypted Price: {:?}", winner.encrypted_price);
    println!("Deposit price: {:?} USDT", deposit_price.low_u128());
    println!("End Time: {:?}", end_time.low_u128());
    println!("Ended: {}", ended);
    Ok(())
}

pub async fn get_total_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
) -> Result<()> {
    let contract = zkAuctionContract::new(*contract_address, signer.into());
    let total = contract.auction_count().call().await?;
    println!("Auctions total: {:?}", total);
    Ok(())
}

pub async fn create_new_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
    public_key_hex: Bytes,
    name: String,
    description: String,
    target_price: U256,
    duration: U256,
) -> Result<()> {
    let contract = zkAuctionContract::new(*contract_address, signer.into());
    let contract_caller =
        contract.create_auction(public_key_hex, name, description, target_price, duration);
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Create auction successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}