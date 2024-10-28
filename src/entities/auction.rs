use chrono::{TimeZone, Utc};
use ethers::types::{Address, Bytes, U256};

use crate::controllers::auction::{Asset, Winner};

#[derive(Debug, Clone)]
pub struct AuctionEntity {
    pub owner: Address,        // Owner of the auction
    pub encryption_key: Bytes, // Owner's public key
    pub asset: Asset,          // Asset being auctioned
    pub winner: Winner,        // Winner of the auction
    pub deposit_price: U256,   // Deposit price when bidder start bid
    pub end_time: U256,        // Time when the bid phase end
    pub ended: bool,           // Status of the auction
}

impl From<(Address, Bytes, Asset, Winner, U256, U256, bool)> for AuctionEntity {
    fn from(value: (Address, Bytes, Asset, Winner, U256, U256, bool)) -> Self {
        let (owner, encryption_key, asset, winner, deposit_price, end_time, ended) = value;
        AuctionEntity {
            owner,
            encryption_key,
            asset,
            winner,
            deposit_price,
            end_time,
            ended,
        }
    }
}

impl AuctionEntity {
    pub fn print_info(&self) {
        println!("Auction Details:");
        println!("Name: {}", self.asset.name);
        println!("Seller: {:?}", self.owner);
        println!("Seller's public encryption key: {:?}", self.encryption_key);
        println!("Description: {}", self.asset.description);
        println!("Item:");
        println!("  Address of NFT Contract: {:?}", self.asset.nft_contract);
        println!("  Token ID: {:?}", self.asset.token_id);
        println!("Winner:");
        println!("  Address: {:?}", self.winner.winner);
        println!("  Encrypted Price: {:?}", self.winner.price);
        println!("Deposit price: {:?} USDT", self.deposit_price);
        println!("End Time: {:?}", Utc.timestamp_opt(self.end_time.as_u128() as i64, 0).unwrap());
        println!("Ended: {}", self.ended);
    }
}
