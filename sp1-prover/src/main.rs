#![no_main]

use aligned_sp1_prover::{calc_auction_hash, find_winner, AuctionData};
use ecies::private_key::PrivateKey;

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let auction_data = sp1_zkvm::io::read::<AuctionData>();
    let pvk_bytes = sp1_zkvm::io::read::<Vec<u8>>();

    let pvk = PrivateKey::from_bytes(&pvk_bytes);

    // let (winner_addr, winner_amount) = (vec![0u128], 0u128);
    let (winner_addr, winner_amount) = find_winner(&auction_data, pvk);
    sp1_zkvm::io::commit(&calc_auction_hash(&auction_data));
    sp1_zkvm::io::commit(&winner_addr);
    sp1_zkvm::io::commit(&winner_amount);
}
