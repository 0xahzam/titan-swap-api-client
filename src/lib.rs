use crate::quote::{QuoteRequest, QuoteResponse, SwapMode, SwapQuotes};
use reqwest::Response;
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

pub mod quote;
pub mod serde_helpers;
pub mod swap;

const TITAN_API_URL: &str = "https://api.titan.exchange";

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Request failed with status {status}: {body}")]
    RequestFailed {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("HTTP client error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("No routes available")]
    NoRoutesAvailable,
    #[error("Failed to decode msgpack: {0}")]
    MsgpackError(#[from] rmp_serde::decode::Error),
}

pub struct TitanClient {
    client: reqwest::Client,
    base_path: String,
    auth_header: String,
}

impl TitanClient {
    pub fn new(auth_token: String, base_path: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_path: base_path.unwrap_or_else(|| TITAN_API_URL.to_string()),
            auth_header: format!("Bearer {}", auth_token),
        }
    }

    async fn fetch_swap_quotes(
        &self,
        params: &[(&str, String)],
    ) -> Result<SwapQuotes, ClientError> {
        let response = self
            .client
            .get(format!("{}/api/v1/quote/swap", self.base_path))
            .query(params)
            .header("Accept", "application/vnd.msgpack")
            .header("Authorization", &self.auth_header)
            .send()
            .await?;

        let response = check_response(response).await?;
        let buffer = response.bytes().await?;
        Ok(rmp_serde::from_slice(&buffer)?)
    }

    pub async fn quote(&self, request: &QuoteRequest) -> Result<QuoteResponse, ClientError> {
        let params = build_query_params(request);
        let quotes = self.fetch_swap_quotes(&params).await?;

        let route = quotes
            .quotes
            .into_values()
            .next()
            .ok_or(ClientError::NoRoutesAvailable)?;

        let context_slot = route.context_slot.unwrap_or(0);
        let route_plan: Vec<_> = route
            .steps
            .iter()
            .map(|step| transform_step(step, context_slot))
            .collect();

        Ok(QuoteResponse {
            input_mint: request.input_mint,
            in_amount: request.amount,
            output_mint: request.output_mint,
            out_amount: route.out_amount,
            swap_mode: request.swap_mode.clone().unwrap_or_default(),
            slippage_bps: route.slippage_bps,
            platform_fee: route.platform_fee.as_ref().map(|pf| quote::PlatformFee {
                amount: pf.amount,
                fee_bps: pf.fee_bps,
            }),
            raw_route: route.clone(),
            route_plan,
            context_slot: route.context_slot,
            time_taken: route.time_taken_ns.map(|ns| ns as f64 / 1e9),
        })
    }

    pub fn swap(&self, quote: &QuoteResponse) -> Result<swap::SwapResponse, ClientError> {
        let route = &quote.raw_route;

        if route.instructions.is_empty() {
            return Err(ClientError::NoRoutesAvailable);
        }

        let solana_instructions: Vec<solana_sdk::instruction::Instruction> = route
            .instructions
            .iter()
            .map(|inst| solana_sdk::instruction::Instruction {
                program_id: pubkey_from_bytes(&inst.p),
                accounts: inst
                    .a
                    .iter()
                    .map(|meta| solana_sdk::instruction::AccountMeta {
                        pubkey: pubkey_from_bytes(&meta.p),
                        is_signer: meta.s,
                        is_writable: meta.w,
                    })
                    .collect(),
                data: inst.d.clone(),
            })
            .collect();

        let address_lookup_tables: Vec<Pubkey> = route
            .address_lookup_tables
            .iter()
            .map(pubkey_from_bytes)
            .collect();

        Ok(swap::SwapResponse {
            instructions: solana_instructions,
            address_lookup_table_addresses: address_lookup_tables,
            compute_unit_limit: route.compute_units.unwrap_or(0) as u32,
            compute_units_safe: route.compute_units_safe,
            context_slot: route.context_slot,
            expires_at_ms: route.expires_at_ms,
            expires_after_slot: route.expires_after_slot,
        })
    }
}

fn build_query_params(request: &QuoteRequest) -> Vec<(&'static str, String)> {
    let mut params = vec![
        ("inputMint", request.input_mint.to_string()),
        ("outputMint", request.output_mint.to_string()),
        ("amount", request.amount.to_string()),
        ("userPublicKey", request.user_pubkey.to_string()),
    ];

    if let Some(max_accounts) = request.max_accounts {
        params.push(("accountsLimitTotal", max_accounts.to_string()));
    }
    if let Some(swap_mode) = &request.swap_mode {
        params.push((
            "swapMode",
            match swap_mode {
                SwapMode::ExactIn => "ExactIn",
                SwapMode::ExactOut => "ExactOut",
            }
            .to_string(),
        ));
    }
    if request.slippage_bps > 0 {
        params.push(("slippageBps", request.slippage_bps.to_string()));
    }
    if let Some(only_direct_routes) = request.only_direct_routes {
        params.push(("onlyDirectRoutes", only_direct_routes.to_string()));
    }
    if let Some(ref excluded_dexes) = request.excluded_dexes {
        params.push(("excludeDexes", excluded_dexes.clone()));
    }
    if let Some(size_constraints) = request.size_constraints {
        params.push(("sizeConstraint", size_constraints.to_string()));
    }
    if let Some(accounts_limit_writable) = request.accounts_limit_writable {
        params.push(("accountsLimitWritable", accounts_limit_writable.to_string()));
    }

    if let Some(ref providers) = request.providers {
        params.push(("providers", providers.to_string()));
    }

    params
}

fn transform_step(
    step: &crate::quote::RoutePlanStepData,
    default_context_slot: u64,
) -> crate::quote::RoutePlanStep {
    crate::quote::RoutePlanStep {
        swap_info: crate::quote::SwapInfo {
            amm_key: pubkey_from_bytes(&step.amm_key),
            label: step.label.clone(),
            input_mint: pubkey_from_bytes(&step.input_mint),
            output_mint: pubkey_from_bytes(&step.output_mint),
            in_amount: step.in_amount,
            out_amount: step.out_amount,
            alloc_ppb: step.alloc_ppb as u64,
            fee_mint: step
                .fee_mint
                .as_ref()
                .map_or_else(Pubkey::default, pubkey_from_bytes),
            fee_amount: step.fee_amount.unwrap_or(0),
            context_slot: step.context_slot.unwrap_or(default_context_slot),
        },
        percent: 100,
    }
}

async fn check_response(response: Response) -> Result<Response, ClientError> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if status == reqwest::StatusCode::NOT_FOUND && body.contains("No routes") {
        return Err(ClientError::NoRoutesAvailable);
    }

    Err(ClientError::RequestFailed { status, body })
}

#[inline]
fn pubkey_from_bytes(bytes: &[u8; 32]) -> Pubkey {
    Pubkey::from(*bytes)
}
