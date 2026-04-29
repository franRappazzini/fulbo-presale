use anchor_lang::prelude::*;

use crate::constants::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct BeneficiaryAllocation {
    pub total_tokens: u64,
    pub withdrawn_tokens: u64,
    // pub tge_timestamp: i64,
    pub last_vesting_claim: i64, // timestamp
    pub tge_start: i64,          // timestamp

    pub monthly_unlocked: u64, // 5% fixed expresed in amount, not percentage (if the first month 5% = 100 tokens, every month will be 100 tokens)
    pub tge_unlock_bps: u16,   // % claimable at tge

    pub tge_claimed: bool, // unlocked tokens
    pub is_manual: bool,   // for liquidity
    pub bump: u8,
}

impl BeneficiaryAllocation {
    pub const SIZE: usize = DISCRIMINATOR + BeneficiaryAllocation::INIT_SPACE;
}
