use anchor_lang::prelude::*;

use crate::{constants::DISCRIMINATOR, error::ErrorCode};

#[account]
#[derive(InitSpace)]
pub struct Position {
    pub total_tokens: u64,
    pub total_sol: u64,

    pub tokens_claimed: u64,
    pub last_claim_ts: i64,
    pub stage_allocations: [StageAllocation; 11],

    pub is_initialized: bool,
    pub bump: u8,
}

impl Position {
    pub const SIZE: usize = DISCRIMINATOR + Position::INIT_SPACE;

    pub fn purchase(&mut self, current_stage: u8, amount: u64, sol_paid: u64) -> Result<()> {
        self.total_tokens = self
            .total_tokens
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;

        self.total_sol = self
            .total_sol
            .checked_add(sol_paid)
            .ok_or(ErrorCode::MathOverflow)?;

        let stage_alloc = &mut self.stage_allocations[current_stage as usize];
        stage_alloc.tokens = stage_alloc
            .tokens
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        stage_alloc.sol_paid = stage_alloc
            .sol_paid
            .checked_add(sol_paid)
            .ok_or(ErrorCode::MathOverflow)?;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct StageAllocation {
    pub tokens: u64,
    pub claimed: u64,
    pub sol_paid: u64,
    pub locked_pct_bps: u16,
}
