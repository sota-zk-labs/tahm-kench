use ethers::types::{Address, Bytes, U256};
#[derive(Debug, Clone)]
pub struct AuctionEntity {
    pub owner: Address,          // Owner of the auction
    pub owner_public_key: Bytes, // Owner's public key
    pub asset: AssetEntity,      // Asset being auctioned
    pub winner: WinnerEntity,    // Winner of the auction
    pub deposit_price: U256,     // Deposit price when bidder start bid
    pub end_time: U256,          // Time when the bid phase end
    pub ended: bool,             // Status of the auction
}
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BidEntity {
    pub bidder: Address,        // Address of the bidder
    pub encrypted_price: Bytes, // Encrypted price submitted of bidder
}
#[derive(Debug, Clone)]
pub struct AssetEntity {
    pub name: String,                  // Name of the asset
    pub description: String,           // Description of the asset
    pub nft_contract_address: Address, // Address of nft contract
    pub token_id: U256,                // Id nft
}

#[derive(Debug, Clone)]
pub struct WinnerEntity {
    pub winner_address: Address, // Address of the winner
    pub price: U256,             // Price submitted of winner
}

impl AuctionEntity {
    pub fn print_info(&self) {
        println!("Auction Details:");
        println!("Name: {}", self.asset.name);
        println!("Seller: {:?}", self.owner);
        println!("Seller's public key: {:?}", self.owner_public_key);
        println!("Description: {}", self.asset.description);
        println!("Item:");
        println!(
            "  Address of NFT Contract: {:?}",
            self.asset.nft_contract_address
        );
        println!("  Token ID: {:?}", self.asset.token_id);
        println!("Winner:");
        println!("  Address: {:?}", self.winner.winner_address);
        println!("  Encrypted Price: {:?}", self.winner.price);
        println!("Deposit price: {:?} USDT", self.deposit_price.low_u128());
        println!("End Time: {:?}", self.end_time.low_u128());
        println!("Ended: {}", self.ended);
    }
}
