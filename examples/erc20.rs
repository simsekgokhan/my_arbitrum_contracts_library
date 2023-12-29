//! Example on how to interact with a deployed `stylus-hello-world` program using defaults.
//! This example uses ethers-rs to instantiate the program using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed program is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

// e.g. usage:
// PRIV_KEY_PATH=/opt/7d3f.pri \
// RPC_URL=https://stylus-testnet.arbitrum.io/rpc \
// STYLUS_PROGRAM_ADDRESS=0xC4CA13280b8EafD7A033670E620B1AF74950E147 \
// cargo run --example erc20

// Contracts:
// interface IErc20 {
//     function name() external pure returns (string memory);
//     function symbol() external pure returns (string memory);
//     function decimals() external pure returns (uint8);
//     function balanceOf(address _address) external view returns (uint256);
//     function transfer(address to, uint256 value) external returns (bool);
//     function approve(address spender, uint256 value) external returns (bool);
//     function transferFrom(address from, address to, uint256 value) external returns (bool);
//     function allowance(address owner, address spender) external view returns (uint256);
// }

// interface IWeth is IErc20 {
//     function deposit() external payable;
//     function withdraw(uint256 amount) external;
//     function sum(uint256[] memory values) external pure returns (string memory, uint256);
//     function sumWithHelper(address helper, uint256[] memory values) external view returns (uint256);
// }

use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use eyre::eyre;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Arc;
// /// Import the Stylus SDK along with alloy primitive types for use in our program.
// use stylus_sdk::{alloy_primitives::U256, prelude::*};

/// Your private key file path.
const ENV_PRIV_KEY_PATH: &str = "PRIV_KEY_PATH";

/// Stylus RPC endpoint url.
const ENV_RPC_URL: &str = "RPC_URL";

/// Deployed pragram address.
const ENV_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let priv_key_path = std::env::var(ENV_PRIV_KEY_PATH)
        .map_err(|_| eyre!("No {} env var set", ENV_PRIV_KEY_PATH))?;
    let rpc_url =
        std::env::var(ENV_RPC_URL).map_err(|_| eyre!("No {} env var set", ENV_RPC_URL))?;
    let program_address = std::env::var(ENV_PROGRAM_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", ENV_PROGRAM_ADDRESS))?;
    abigen!(
        Weth,
        r#"[
            function deposit() external payable
            function withdraw(uint256 amount) external
            function sum(uint256[] memory values) external pure returns (string memory, uint256)
            function sumWithHelper(address helper, uint256[] memory values) external view returns (uint256)
            function decimals() external pure returns (uint8)
        ]"#
    );

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let address: Address = program_address.parse()?;

    let privkey = read_secret_from_file(&priv_key_path)?;
    let wallet = LocalWallet::from_str(&privkey)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    // ====
    let ww = Weth::new(address, client);

    // call fn from Weth
    let xx: U256 = U256::from(16);
    let num = ww.sum(vec![xx]).call().await;
    println!("\n--- sum = {:?}\n", num); // todo

    // Call fn from base Erc20
    let decimals = ww.decimals().call().await;
    println!("\n--- decimals = {:?}\n", decimals);
    // ====

    Ok(())
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}
