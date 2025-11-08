# titan-swap-api-client

A Rust client library for the Titan Swap API on Solana.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
titan-swap-api-client = { git = "https://github.com/0xahzam/titan-swap-api-client.git" }
```

## Usage

### Basic Example

```rust
use titan_swap_api_client::{quote::{QuoteRequest, SwapMode, Provider}, TitanClient};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let auth_token = "your-auth-token".to_string();
    let client = TitanClient::new(auth_token, None);

    let request = QuoteRequest {
        input_mint: Pubkey::from_str("So11111111111111111111111111111111111111112")?,
        output_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?,
        amount: 1_000_000_000, // 1 SOL in lamports
        user_pubkey: Pubkey::from_str("YOUR_WALLET_ADDRESS")?,
        max_accounts: Some(50),
        swap_mode: Some(SwapMode::ExactIn),
        slippage_bps: 50,
        providers: Some(Provider::Titan), // Optional: force Titan routing only
        ..Default::default()
    };

    // Get a quote
    let quote = client.quote(&request).await?;
    println!("Quote: {} -> {}", quote.in_amount, quote.out_amount);

    // Get swap instructions
    let swap = client.swap(&quote)?;
    println!("Instructions: {}", swap.instructions.len());

    Ok(())
}
```

### QuoteRequest Fields

- `input_mint`: Input token mint address
- `output_mint`: Output token mint address
- `amount`: Amount to swap (in lamports for native SOL)
- `user_pubkey`: User's wallet public key
- `max_accounts`: Maximum number of accounts (optional)
- `swap_mode`: `SwapMode::ExactIn` or `SwapMode::ExactOut` (optional)
- `slippage_bps`: Slippage tolerance in basis points (e.g., 50 = 0.5%)
- `providers`: `Some(Provider::Titan)` to force Titan only routing, `None` for best route across all providers (optional)
- `only_direct_routes`: Only direct routes (optional)
- `excluded_dexes`: Comma-separated list of DEXes to exclude (optional)

## Examples

See the [examples](./examples/) directory for a complete working example that demonstrates:

- Getting quotes
- Getting swap instructions
- Sending transactions to Solana

Run the example:

```bash
cargo run --package titan-swap-test
```

See [examples/README.md](./examples/README.md) for detailed setup instructions.

## License

MIT
