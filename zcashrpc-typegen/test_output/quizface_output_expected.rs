//proceedurally generated response types, note that zcashrpc-typegen
//is in early alpha, and output is subject to change at any time.
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Consensus {
    pub chaintip: String,
    pub nextblock: String,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Enforce {
    pub found: rust_decimal::Decimal,
    pub required: rust_decimal::Decimal,
    pub status: bool,
    pub window: rust_decimal::Decimal,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Reject {
    pub found: rust_decimal::Decimal,
    pub required: rust_decimal::Decimal,
    pub status: bool,
    pub window: rust_decimal::Decimal,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Softforks {
    pub enforce: Enforce,
    pub id: String,
    pub reject: Reject,
    pub version: rust_decimal::Decimal,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Upgrades {
    pub activationheight: rust_decimal::Decimal,
    pub info: String,
    pub name: String,
    pub status: String,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Getblockchaininfo {
    pub bestblockhash: String,
    pub blocks: rust_decimal::Decimal,
    pub chain: String,
    pub chainwork: String,
    pub commitments: rust_decimal::Decimal,
    pub consensus: Consensus,
    pub difficulty: rust_decimal::Decimal,
    pub estimatedheight: rust_decimal::Decimal,
    pub headers: rust_decimal::Decimal,
    pub initial_block_download_complete: bool,
    pub size_on_disk: rust_decimal::Decimal,
    pub softforks: Vec<Softforks>,
    pub upgrades: std::collections::HashMap<String, Upgrades>,
    pub verificationprogress: rust_decimal::Decimal,
}
#[derive(Debug, serde :: Deserialize, serde :: Serialize)]
pub struct Getinfo {
    pub balance: rust_decimal::Decimal,
    pub blocks: rust_decimal::Decimal,
    pub connections: rust_decimal::Decimal,
    pub difficulty: rust_decimal::Decimal,
    pub errors: String,
    pub keypoololdest: rust_decimal::Decimal,
    pub keypoolsize: rust_decimal::Decimal,
    pub paytxfee: rust_decimal::Decimal,
    pub protocolversion: rust_decimal::Decimal,
    pub proxy: Option<String>,
    pub relayfee: rust_decimal::Decimal,
    pub testnet: bool,
    pub timeoffset: rust_decimal::Decimal,
    pub unlocked_until: rust_decimal::Decimal,
    pub version: rust_decimal::Decimal,
    pub walletversion: rust_decimal::Decimal,
}
