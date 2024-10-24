use anyhow::Result;
use ethers::contract::Contract;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::erc::ERCNFTType::ERC721;
use ethers::prelude::*;
use ethers::prelude::{Http, LocalWallet, Provider};
use ethers::types::{Address, Bytes, U256};

abigen!(zkAuction, "./src/assets/zk_auction.json");

// pub async fn approve_nft(
//     signer: SignerMiddleware<Provider<Http>, LocalWallet>,
//     contract_address: &Address,
//     nft_contract_address: &Address,
//     token_id: U256
// ) -> Result<()> {
//     let auction_contract = zkAuction::new(*contract_address, signer.into());
//     let nft_contract = Contract::new(nft_contract_address, ERC721_ABI.clone(), signer.clone());
//
//
//
//     println!("Transaction successful: {:?}", receipt);
//     Ok(())
//
// }

pub async fn create_new_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
    public_key_hex: Bytes,
    name: String,
    description: String,
    nft_contract_address: Address,
    token_id: U256,
    target_price: U256,
    duration: U256,
) -> Result<()> {
    let contract = zkAuction::new(*contract_address, signer.into());
    let contract_caller = contract.create_auction(
        public_key_hex,
        nft_contract_address,
        token_id,
        name,
        description,
        target_price,
        duration,
    );
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Create auction successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

pub async fn get_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
    auction_id: U256,
) -> Result<()> {
    let contract = zkAuction::new(*contract_address, signer.into());
    let auction = contract.auctions(auction_id).call().await?;
    let (seller, pk, asset, item, winner, deposit_price, end_time, ended) = auction;
    println!("Auction Details:");
    println!("Name: {}", asset.name);
    println!("Seller: {:?}", seller);
    println!("Seller's public key: {:?}", pk);
    println!("Description: {}", asset.description);
    println!("Item:");
    println!("  Address of NFT Contract: {:?}", item.nft_contract);
    println!("  Token ID: {:?}", item.token_id);
    println!("Winner:");
    println!("  Address: {:?}", winner.winner);
    println!("  Encrypted Price: {:?}", winner.price);
    println!("Deposit price: {:?} USDT", deposit_price.low_u128());
    println!("End Time: {:?}", end_time.low_u128());
    println!("Ended: {}", ended);
    Ok(())
}

pub async fn get_total_auction(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
) -> Result<()> {
    let contract = zkAuction::new(*contract_address, signer.into());
    let total = contract.auction_count().call().await?;
    println!("Auctions total: {:?}", total);
    Ok(())
}

// pub fn encrypt_price(bid_price: U256) -> Bytes {
//
// }
pub async fn approve_deposit() {}
pub async fn create_bid(
    signer: SignerMiddleware<Provider<Http>, LocalWallet>,
    contract_address: &Address,
    auction_id: U256,
    bid_price: U256,
) -> Result<()> {
    let covert_price: [u8; 32] = bid_price.into();
    let covert_price_hex = hex::encode(covert_price);
    let covert_price_bytes: Bytes = hex::decode(&covert_price_hex)
        .expect("Failed to decode hex string") // Handle potential errors
        .into(); // Convert Vec<u8> to Bytes
    let contract = zkAuction::new(*contract_address, signer.into());
    let contract_caller = contract.new_bid(auction_id, covert_price_bytes);
    let tx = contract_caller.send().await?;
    let receipt = tx.await?.unwrap();
    let tx_hash = receipt.transaction_hash;
    println!(
        "Create bid successfully with transaction_hash : {:?}",
        tx_hash
    );
    Ok(())
}

// pub async fn list_bid(auction_id: U256) {}
//
// pub async fn submit_winner() {}
//
// pub async fn withdraw() {}
