mod core_error;

use clap::CommandFactory;
use clap::{Parser, Subcommand};

/// TAHKEN
#[derive(Parser, Debug)]
#[command(name = "tahken")]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[clap(short, long)]
    version: bool,
}
#[derive(Subcommand, Clone, Debug, PartialEq)]
enum Commands {
    /// Print current version
    Version,

    /// Create auction session
    CreateAuction {
        #[arg(short, long)]
        public_key: String,
    },

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
        private_key: String,
    },

    /// Claim item
    Claim {
        #[arg(short, long)]
        id: i32,
    }
}

#[tokio::main]
async fn main() {
    // // Will notify users in one day intervals if an update is available
    // if !check_latest_version(
    //     env!("CARGO_PKG_NAME"),
    //     env!("CARGO_PKG_VERSION"),
    //     REGISTRY_URL
    // ).unwrap() {
    //     return;
    // }

    let args = Cli::parse();

    // if args.version {
    //     println!(env!("APP_VERSION"));
    //     return;
    // }
    match args.command {
        Some(command) => match command {
            Commands::Version => {
                println!("version 1.0");
                return;
            }
            Commands::CreateAuction {public_key} => {
                create_auction(&public_key).await;
                return;
            }
            Commands::ListAuctions => {
                println!("have 3 auction opening");
                return;
            }
            Commands::GetAuctions {id_auction} => {
                println!("detail of id_auction: {}", id_auction);
                return;
            }
            Commands::Bid {price, id_auction} => {
                println!("bid itam with: (price: {}, id_auction: {})", price, id_auction);
                return;
            }
            Commands::Submit{id, private_key} => {
                println!("submit with: (id: {}, private_key: {})", id, private_key);
                return;
            }
            Commands::Claim {id}=> {
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

