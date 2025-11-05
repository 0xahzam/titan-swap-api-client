# titan-swap-api-client

## Introduction

The titan-swap-api-client is a Rust client library designed to simplify the integration of the Titan Swap API.

## Getting Started

To use the titan-swap-api-client crate in your Rust project, follow these simple steps:

Add the crate to your Cargo.toml:

```toml
[dependencies]
titan-swap-api-client = { git = "https://github.com/0xahzam/titan-swap-api-client.git", package = "titan-swap-api-client" }
```

## Examples

Here's a simplified example of how to use the titan-swap-api-client in your Rust application:

```rust
use titan_swap_api_client::{quote::QuoteRequest, quote::SwapMode, TitanClient};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const NATIVE_MINT: &str = "So11111111111111111111111111111111111111112";
const TEST_WALLET: &str = "YOUR_WALLET_ADDRESS";

#[tokio::main]
async fn main() {
    let auth_token = "your-auth-token-here".to_string();
    let client = TitanClient::new(auth_token, None);

    let quote_request = QuoteRequest {
        input_mint: Pubkey::from_str(NATIVE_MINT).unwrap(),
        output_mint: Pubkey::from_str(USDC_MINT).unwrap(),
        amount: 1_000_000_000, // 1 SOL
        user_pubkey: Pubkey::from_str(TEST_WALLET).unwrap(),
        max_accounts: Some(50),
        swap_mode: Some(SwapMode::ExactIn),
        slippage_bps: 50,
        ..QuoteRequest::default()
    };

    // GET /api/v1/quote/swap
    let quote_response = client.quote(&quote_request).await.unwrap();
    println!("{quote_response:#?}");

    // GET /api/v1/quote/swap (returns swap instructions)
    let swap_response = client.swap(&quote_request).await.unwrap();
    println!("Number of instructions: {}", swap_response.instructions.len());
    println!("Compute unit limit: {}", swap_response.compute_unit_limit);
    println!("Address lookup tables: {}", swap_response.address_lookup_table_addresses.len());

    // Perform further actions as needed...
}
```
