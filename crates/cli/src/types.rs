use std::sync::Arc;

use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, LocalWallet, Provider};

pub type EthSigner = SignerMiddleware<Arc<Provider<Http>>, LocalWallet>;
