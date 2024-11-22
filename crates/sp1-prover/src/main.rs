#![no_main]

use auction_sp1_prover::{calc_auction_hash, decrypt_bidder_data, AuctionData};
use ecies::Ecies;
use ecies::private_key::PrivateKey;

sp1_zkvm::entrypoint!(main);

/// Entrypoint for the zkVM program.
pub fn main() {
    let auction_data = sp1_zkvm::io::read::<AuctionData>();

    let pvk = PrivateKey::from_bytes(&sp1_zkvm::io::read::<Vec<u8>>());
    let scheme = Ecies::from_pvk(pvk);

    // Find the winner
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
