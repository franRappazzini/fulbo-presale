use anchor_lang::prelude::*;

use crate::constants::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct BeneficiaryAllocation {
    pub total_tokens: u64,
    pub withdrawn_tokens: u64,
    /// Fixed monthly unlock amount (5 % of `total_tokens`), pre-computed at initialization to avoid repeated division
    pub monthly_unlocked: u64,
    /// Percentage of `total_tokens` unlocked immediately at TGE (in bps)
    pub tge_unlock_bps: u16,
    /// Liquidity beneficiary receives their full allocation in one shot at TGE
    pub is_liquidity: bool,
    pub bump: u8,
}

impl BeneficiaryAllocation {
    pub const SIZE: usize = DISCRIMINATOR + BeneficiaryAllocation::INIT_SPACE;
}
