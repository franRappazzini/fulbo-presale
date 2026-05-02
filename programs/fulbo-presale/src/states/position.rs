use anchor_lang::prelude::*;

use crate::{constants::DISCRIMINATOR, error::ErrorCode, states::PurchaseResult};

#[account]
#[derive(InitSpace)]
pub struct Position {
    pub total_tokens: u64,
    pub total_sol: u64,

    pub tokens_claimed: u64,
    pub stage_allocations: [StageAllocation; 11],

    pub is_initialized: bool,
    pub bump: u8,
}

impl Position {
    pub const SIZE: usize = DISCRIMINATOR + Position::INIT_SPACE;

    pub fn purchase(&mut self, result: &PurchaseResult) -> Result<()> {
        msg!(
            "recording purchase: stage {}, tokens {}, lamports {}, overflow {:?}",
            result.stage,
            result.tokens,
            result.lamports,
            result.overflow
        );

        let total_tokens = result.tokens + result.overflow.map(|(_, t, _, _)| t).unwrap_or(0);
        let total_lamports = result.lamports + result.overflow.map(|(_, _, l, _)| l).unwrap_or(0);

        self.total_tokens = self
            .total_tokens
            .checked_add(total_tokens)
            .ok_or(ErrorCode::MathOverflow)?;

        self.total_sol = self
            .total_sol
            .checked_add(total_lamports)
            .ok_or(ErrorCode::MathOverflow)?;

        // update current stage
        let alloc = &mut self.stage_allocations[result.stage as usize];
        alloc.tokens = alloc
            .tokens
            .checked_add(result.tokens)
            .ok_or(ErrorCode::MathOverflow)?;
        alloc.sol_paid = alloc
            .sol_paid
            .checked_add(result.lamports)
            .ok_or(ErrorCode::MathOverflow)?;

        if alloc.locked_pct_bps == 0 {
            alloc.locked_pct_bps = result.locked_pct_bps;
        }

        // update overflow stage if present
        if let Some((stage, tokens, lamports, locked_pct_bps)) = result.overflow {
            let overflow_alloc = &mut self.stage_allocations[stage as usize];
            overflow_alloc.tokens = overflow_alloc
                .tokens
                .checked_add(tokens)
                .ok_or(ErrorCode::MathOverflow)?;

            overflow_alloc.sol_paid = overflow_alloc
                .sol_paid
                .checked_add(lamports)
                .ok_or(ErrorCode::MathOverflow)?;

            overflow_alloc.locked_pct_bps = locked_pct_bps;
        }

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct StageAllocation {
    pub tokens: u64,
    pub sol_paid: u64,
    pub claimed: u64,
    pub locked_pct_bps: u16,
}
