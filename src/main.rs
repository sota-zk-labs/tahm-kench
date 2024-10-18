mod core_error;
mod config;

use std::sync::Arc;
use config::Config;
use clap::CommandFactory;
use clap::{Parser, Subcommand};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, Bytes, H160, U256};


abigen!(
    zkAuctionContract,
    "./src/assets/ZkAuction.json"
);
/// TAHKEN
#[derive(Parser, Debug)]
#[command(name = "tahken")]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(short, long)]
    version: bool,
    #[clap(short, long, default_value = "config.toml")]
    config_path: String,
    /// Keystore path
    #[clap(short, long)]
    keystore_path: String,
}
#[derive(Subcommand, Clone, Debug, PartialEq)]
enum Commands {
    /// Print current version
    Version,
    /// Create auction session
    CreateAuction,
    /// Get list auctions opening
    ListAuctions,
    /// Get detail auctions
    GetAuctions {
        #[arg(short, long)]
        id_auction: i32,
    },
    /// Bid item
    Bid {
        #[arg(short, long)]
        price: i32,
        #[arg(short, long)]
        id_auction: i32,
    },
    /// Submit
    Submit {
        #[arg(short, long)]
        id: i32,
        #[arg(short, long)]
        private_key: Bytes,
    },
    /// Claim item
    Claim {
        #[arg(short, long)]
        id: i32,
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let config = Config::new(&args.config_path).expect("Config env not set");

    let rpc_url = &config.chain.rpc_url;
    let provider = Provider::<Http>::try_from(rpc_url.as_str()).expect("Failed to connect to provider");
    let chain_id = provider.get_chainid().await.expect("Failed to get chain_id");

    let keystore_password = rpassword::prompt_password("Enter keystore password: ")
        .expect("Failed to read keystore password");
    println!("key: {:?}", &args.keystore_path);
    let wallet = LocalWallet::decrypt_keystore(&args.keystore_path, &keystore_password)
        .expect("Failed to decrypt keystore")
        .with_chain_id(chain_id.as_u64());

    // Get the public key and address
    let address = wallet.address();
    let public_key_bytes = wallet.signer().verifying_key().to_encoded_point(false);
    let public_key_hex = hex::encode(public_key_bytes);

    let signer = SignerMiddleware::new(provider.clone(), wallet.clone());
    let client = Arc::new(signer);

    match args.command {
        Some(command) => match command {
            Commands::Version => {
                println!("version 1.0");
                return;
            }
            Commands::CreateAuction => {
                create_auction(client.clone()).await;
                return;
            }
            Commands::ListAuctions => {
                println!("have 3 auction opening");
                return;
            }
            Commands::GetAuctions { id_auction } => {
                println!("detail of id_auction: {}", id_auction);
                return;
            }
            Commands::Bid { price, id_auction } => {
                println!("bid itam with: (price: {}, id_auction: {})", price, id_auction);
                return;
            }
            Commands::Submit { id, private_key } => {
                println!("submit with: (id: {}, private_key: {})", id, private_key);
                return;
            }
            Commands::Claim { id } => {
                println!("claim item: {}", id);
                return;
            }
        },
        None => {
            Cli::command().print_help().unwrap();
            return;
        }
    }
}

async fn create_auction(client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>) {
    let contract_address = "0xaa80ab74e426a0d2c19178db78649eebd05d05c5".parse::<Address>().unwrap();
    let contract = zkAuctionContract::new(contract_address, client);

    let zk_count = contract.auction_count().call().await.unwrap();
    println!("zk_count: {:?}", zk_count);
}

