use std::str::FromStr;

use crate::serde_helpers::field_as_string;
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Default, PartialEq, Clone, Debug)]
pub enum SwapMode {
    #[default]
    ExactIn,
    ExactOut,
}

impl FromStr for SwapMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ExactIn" => Ok(Self::ExactIn),
            "ExactOut" => Ok(Self::ExactOut),
            _ => Err(anyhow!("{} is not a valid SwapMode", s)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlanStep {
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    #[serde(with = "field_as_string")]
    pub amm_key: Pubkey,
    pub label: String,
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    #[serde(default)]
    pub alloc_ppb: u64,
    #[serde(with = "field_as_string", default)]
    pub fee_mint: Pubkey,
    #[serde(with = "field_as_string", default)]
    pub fee_amount: u64,
    #[serde(default)]
    pub context_slot: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    #[serde(with = "field_as_string")]
    pub amount: u64,
    pub fee_bps: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    pub swap_mode: SwapMode,
    pub slippage_bps: u16,
    pub platform_fee: Option<PlatformFee>,
    pub route_plan: Vec<RoutePlanStep>,
    #[serde(default)]
    pub context_slot: Option<u64>,
    #[serde(default)]
    pub time_taken: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

type Dexes = String;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub amount: u64,
    #[serde(with = "field_as_string")]
    pub user_pubkey: Pubkey,
    pub max_accounts: Option<usize>,
    pub swap_mode: Option<SwapMode>,
    pub slippage_bps: u16,
    pub only_direct_routes: Option<bool>,
    pub excluded_dexes: Option<Dexes>,
    pub size_constraints: Option<u64>,
    pub accounts_limit_writable: Option<u64>,
}

type MsgpackPubkey = [u8; 32];

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapQuotes {
    pub id: String,
    pub input_mint: MsgpackPubkey,
    pub output_mint: MsgpackPubkey,
    pub swap_mode: SwapMode,
    pub amount: u64,
    pub quotes: std::collections::HashMap<String, SwapRoute>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapRoute {
    pub in_amount: u64,
    pub out_amount: u64,
    pub slippage_bps: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) platform_fee: Option<PlatformFeeData>,
    pub(crate) steps: Vec<RoutePlanStepData>,
    pub(crate) instructions: Vec<InstructionData>,
    pub address_lookup_tables: Vec<MsgpackPubkey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_slot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_taken_ns: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after_slot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_units: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_units_safe: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RoutePlanStepData {
    pub amm_key: MsgpackPubkey,
    pub label: String,
    pub input_mint: MsgpackPubkey,
    pub output_mint: MsgpackPubkey,
    pub in_amount: u64,
    pub out_amount: u64,
    pub alloc_ppb: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_mint: Option<MsgpackPubkey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_slot: Option<u64>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub(crate) struct InstructionData {
    pub p: MsgpackPubkey,
    pub a: Vec<AccountMetaData>,
    pub d: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub(crate) struct AccountMetaData {
    pub p: MsgpackPubkey,
    pub s: bool,
    pub w: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub(crate) struct PlatformFeeData {
    pub amount: u64,
    pub fee_bps: u8,
}
