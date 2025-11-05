# Examples

## titan-swap-test

A full integration test that tests the swap functionality and can optionally send transactions.

### Setup

1. Create a `.env` file in the `examples/` directory:

```bash
cd examples
touch .env
```

2. Edit `examples/.env` with your credentials:

```env
TITAN_AUTH_TOKEN=your-auth-token
TITAN_BASE_URL=https://api.titan.exchange
USER_PUBKEY=your-wallet-address
PRIVATE_KEY=your-base58-encoded-private-key
RPC_URL=https://api.mainnet-beta.solana.com
TITAN_SEND_TX=false
```

**Note:** The `.env` file must be located in the `examples/` directory.

### Running

From the workspace root:

```bash
# Test without sending transaction
cargo run --package titan-swap-test

# Test with release build
cargo run --release --package titan-swap-test

# Test and send transaction (WARNING: uses real funds!)
# Set TITAN_SEND_TX=true in examples/.env
```

### Environment Variables

- `TITAN_AUTH_TOKEN` (required): Your Titan API authentication token
- `TITAN_BASE_URL` (optional): Base URL for the API (defaults to production if not set)
- `USER_PUBKEY` (required): Your wallet public key (base58 encoded)
- `PRIVATE_KEY` (required): Your wallet private key (base58 encoded)
- `RPC_URL` (optional): Solana RPC endpoint (defaults to `https://api.mainnet-beta.solana.com`)
- `TITAN_SEND_TX` (optional): Set to `true` to actually send the transaction (defaults to `false`)

### Output

The example will output:

- Quote information (SOL amount, USDC amount, slippage, route steps)
- Swap details (number of instructions, compute unit limit, address lookup tables)
- Transaction signature and explorer link (if `TITAN_SEND_TX=true`)
