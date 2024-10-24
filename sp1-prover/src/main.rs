#![no_main]

use ecies::SecretKey;
use aligned_sp1_prover::{decrypt_bidder_data, AuctionData, PVK_HEX};
// use rsa::pkcs8::DecodePrivateKey;
// use rsa::RsaPrivateKey;
use tiny_keccak::{Hasher, Keccak};

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let auction_data = sp1_zkvm::io::read::<AuctionData>();

    let pvk = SecretKey::parse_slice(&hex::decode(PVK_HEX).unwrap())
        .expect("missing private key to encode bidder data");
    
    let mut winner_addr = &vec![];
    let mut winner_amount = 0;
    for bidder in &auction_data.bidders {
        let bidder_amount = decrypt_bidder_data(&pvk, bidder);
        if winner_amount < bidder_amount {
            winner_amount = bidder_amount;
            winner_addr = &bidder.address;
        }
    }

    sp1_zkvm::io::commit(&calc_auction_hash(&auction_data));
    sp1_zkvm::io::commit(winner_addr);
}

fn calc_auction_hash(auction_data: &AuctionData) -> [u8; 32] {
    let mut input = vec![];
    let mut hasher = Keccak::v256();

    input.extend(&auction_data.id);
    for bidder in &auction_data.bidders {
        input.extend(&bidder.address);
        input.extend(&bidder.encrypted_amount);
    }

    let mut output = [0u8; 32];
    hasher.update(&input);
    hasher.finalize(&mut output);
    output
}
