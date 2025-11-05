use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    message::{v0, AddressLookupTableAccount, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::str::FromStr;
use titan_swap_api_client::{quote::QuoteRequest, quote::SwapMode, TitanClient};

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const SWAP_AMOUNT: u64 = 100_000_000;
const SLIPPAGE_BPS: u16 = 50;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::from_path("examples/.env").ok();

    let auth_token = std::env::var("TITAN_AUTH_TOKEN")
        .expect("TITAN_AUTH_TOKEN environment variable must be set");
    let base_url = std::env::var("TITAN_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty());
    let user_pubkey =
        std::env::var("USER_PUBKEY").expect("USER_PUBKEY environment variable must be set");
    let private_key = std::env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY environment variable must be set (base58 encoded)");
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let send_tx = std::env::var("TITAN_SEND_TX")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let user_pubkey = Pubkey::from_str(&user_pubkey)?;
    let input_mint = Pubkey::from_str(SOL_MINT)?;
    let output_mint = Pubkey::from_str(USDC_MINT)?;

    let keypair = Keypair::from_base58_string(&private_key);
    if keypair.pubkey() != user_pubkey {
        anyhow::bail!("USER_PUBKEY does not match the keypair derived from PRIVATE_KEY");
    }

    let client = TitanClient::new(auth_token, base_url);

    let request = QuoteRequest {
        input_mint,
        output_mint,
        amount: SWAP_AMOUNT,
        user_pubkey,
        max_accounts: Some(50),
        swap_mode: Some(SwapMode::ExactIn),
        slippage_bps: SLIPPAGE_BPS,
        ..Default::default()
    };

    let quote = client.quote(&request).await?;
    println!(
        "Quote: {:.2} SOL -> {:.2} USDC ({} bps slippage, {} step{})",
        SWAP_AMOUNT as f64 / 1e9,
        quote.out_amount as f64 / 1e6,
        quote.slippage_bps,
        quote.route_plan.len(),
        if quote.route_plan.len() == 1 { "" } else { "s" }
    );

    let swap = client.swap(&quote)?;

    println!(
        "Swap: {} instructions, {} CU limit, {} ALT{}",
        swap.instructions.len(),
        swap.compute_unit_limit,
        swap.address_lookup_table_addresses.len(),
        if swap.address_lookup_table_addresses.len() == 1 {
            ""
        } else {
            "s"
        }
    );

    if !send_tx {
        println!("\nSet TITAN_SEND_TX=true to actually send the transaction");
        return Ok(());
    }

    let rpc_client = RpcClient::new(rpc_url);
    let tx = build_and_sign_transaction(
        &rpc_client,
        &keypair,
        swap.instructions,
        swap.address_lookup_table_addresses,
    )
    .await?;

    println!("\nSending transaction...");
    let signature = rpc_client.send_transaction(&tx).await?;

    println!("\nTransaction sent: {}", signature);
    println!("Explorer: https://solscan.io/tx/{}", signature);

    Ok(())
}

async fn build_and_sign_transaction(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    instructions: Vec<Instruction>,
    alt_addresses: Vec<Pubkey>,
) -> anyhow::Result<VersionedTransaction> {
    let blockhash = rpc_client.get_latest_blockhash().await?;

    let lookup_tables = if alt_addresses.is_empty() {
        vec![]
    } else {
        println!(
            "\nFetching {} address lookup tables...",
            alt_addresses.len()
        );

        let mut tables = Vec::with_capacity(alt_addresses.len());
        for alt_address in &alt_addresses {
            let account = rpc_client.get_account(alt_address).await?;
            let alt = AddressLookupTable::deserialize(&account.data)?;

            println!(
                "  Loaded ALT {} with {} addresses",
                alt_address,
                alt.addresses.len()
            );

            tables.push(AddressLookupTableAccount {
                key: *alt_address,
                addresses: alt.addresses.to_vec(),
            });
        }
        tables
    };

    let message =
        v0::Message::try_compile(&keypair.pubkey(), &instructions, &lookup_tables, blockhash)?;

    Ok(VersionedTransaction::try_new(
        VersionedMessage::V0(message),
        &[keypair],
    )?)
}
