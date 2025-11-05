use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

#[derive(Debug, Clone)]
pub struct SwapResponse {
    pub instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub compute_unit_limit: u32,
    pub compute_units_safe: Option<u64>,
    pub context_slot: Option<u64>,
    pub expires_at_ms: Option<u64>,
    pub expires_after_slot: Option<u64>,
    pub transaction: Option<Vec<u8>>,
}
