#![no_main]

use aligned_sp1_prover::{calc_auction_hash, decrypt_bidder_data, AuctionData, PVK_HEX};
use ecies::Ecies;
use ecies::private_key::PrivateKey;

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let auction_data = sp1_zkvm::io::read::<AuctionData>();

    let pvk = PrivateKey::from_hex(PVK_HEX);
    let scheme = Ecies::from_pvk(pvk);
    
    let mut winner_addr = &vec![];
    let mut winner_amount = 0;
    for bidder in &auction_data.bidders {
        let bidder_amount = decrypt_bidder_data(&scheme, bidder);
        if winner_amount < bidder_amount {
            winner_amount = bidder_amount;
            winner_addr = &bidder.address;
        }
    }

    sp1_zkvm::io::commit(&calc_auction_hash(&auction_data));
    sp1_zkvm::io::commit(winner_addr);
    sp1_zkvm::io::commit(&winner_amount);
}
