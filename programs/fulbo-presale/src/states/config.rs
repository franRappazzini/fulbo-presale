use anchor_lang::prelude::*;

use crate::{constants::DISCRIMINATOR, error::ErrorCode};

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub chainlink_feed: Pubkey,

    pub stages: [Stage; 11],

    pub tge_timestamp: i64,
    pub tge_announced_timestamp: i64,

    pub total_sol_raised: u64,
    pub total_tokens_for_sale: u64,
    pub total_tokens_sold: u64,
    pub total_tokens_claimed: u64,

    pub current_stage: u8,
    pub sale_finalized: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Stage {
    pub price_usd: u64,
    pub max_tokens: u64,
    pub tokens_sold: u64,
    pub raised_sol: u64,
    pub locked_pct_bps: u16,
    pub max_wallet_pct_bps: u16,
}

/// Per-stage breakdown of a single purchase, returned by [`Config::add_purchase`].
/// Used by [`Position::purchase`] to record wallet-level allocations correctly.
pub struct PurchaseResult {
    // stage where the first (or only) portion of the purchase lands.
    pub stage: u8,
    // tokens bought in `stage`.
    pub tokens: u64,
    // lamports attributed to `stage`.
    pub lamports: u64,
    // present only when the purchase spilled into the next stage.
    // contains `(stage_index, tokens, lamports, locked_pct_bps)` for the overflow portion.
    pub overflow: Option<(u8, u64, u64, u16)>,

    pub locked_pct_bps: u16,
}

impl Config {
    pub const SIZE: usize = DISCRIMINATOR + Config::INIT_SPACE;

    /// Records a token purchase, updating token and SOL accounting for the affected stage(s).
    ///
    /// ## Validations
    /// - `tokens` must not exceed the max-per-wallet cap of the current stage
    ///   (`max_tokens * max_wallet_pct_bps / 10_000`).
    ///
    /// ## Stage advancement
    /// At most two stages are touched per call:
    /// 1. **Current stage has enough supply** → all `tokens` and `lamports` are recorded there.
    /// 2. **Current stage runs out** → the remaining supply of the current stage is filled,
    ///    `current_stage` advances by one, and the leftover tokens are recorded in the new stage.
    ///    SOL is split proportionally between the two stages
    ///    (`stage1_lamports = lamports * available / tokens`), since the entire purchase was
    ///    priced at the current-stage rate.
    ///
    /// ## Global counters
    /// `total_tokens_sold` and `total_sol_raised` are always incremented by the full amounts,
    /// regardless of how many stages were touched.
    pub fn add_purchase(&mut self, tokens: u64, lamports: u64) -> Result<PurchaseResult> {
        let current_stage = self.current_stage as usize;

        let max_per_stage = (self.stages[current_stage].max_tokens as u128)
            .checked_mul(self.stages[current_stage].max_wallet_pct_bps as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(10_000)
            .ok_or(ErrorCode::MathOverflow)?;

        msg!("Tokens: {} | Max per stage: {}", tokens, max_per_stage);

        require!(
            tokens as u128 <= max_per_stage,
            ErrorCode::ExceedsMaxPerWallet
        );

        let available_stage_amount = self.stages[current_stage]
            .max_tokens
            .checked_sub(self.stages[current_stage].tokens_sold)
            .ok_or(ErrorCode::MathOverflow)?;

        let result = if tokens <= available_stage_amount {
            self.stages[current_stage].tokens_sold = self.stages[current_stage]
                .tokens_sold
                .checked_add(tokens)
                .ok_or(ErrorCode::MathOverflow)?;

            self.stages[current_stage].raised_sol = self.stages[current_stage]
                .raised_sol
                .checked_add(lamports)
                .ok_or(ErrorCode::MathOverflow)?;

            if self.stages[current_stage].tokens_sold == self.stages[current_stage].max_tokens
                && (self.current_stage as usize) < self.stages.len() - 1
            {
                self.current_stage += 1;
            }

            PurchaseResult {
                stage: current_stage as u8,
                tokens,
                lamports,
                overflow: None,
                locked_pct_bps: self.stages[current_stage].locked_pct_bps,
            }
        } else {
            // check max 11 stages (0-10 index)
            require!(
                self.current_stage < self.stages.len() as u8 - 1,
                ErrorCode::InvalidAmount
            );

            let remaining_tokens = tokens
                .checked_sub(available_stage_amount)
                .ok_or(ErrorCode::MathOverflow)?;

            msg!(
                "tokens: {} | available in stage: {} | remaining: {}",
                tokens,
                available_stage_amount,
                remaining_tokens
            );

            // split SOL proportionally: current_stage_lamports / total_lamports = available_tokens / total_tokens
            let current_stage_lamports = (lamports as u128)
                .checked_mul(available_stage_amount as u128)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_div(tokens as u128)
                .ok_or(ErrorCode::MathOverflow)? as u64;

            let new_stage_lamports = lamports
                .checked_sub(current_stage_lamports)
                .ok_or(ErrorCode::MathOverflow)?;

            self.stages[current_stage].tokens_sold = self.stages[current_stage].max_tokens;
            self.stages[current_stage].raised_sol = self.stages[current_stage]
                .raised_sol
                .checked_add(current_stage_lamports)
                .ok_or(ErrorCode::MathOverflow)?;

            self.current_stage += 1;
            let new_current_stage = self.current_stage as usize;

            self.stages[new_current_stage].tokens_sold = self.stages[new_current_stage]
                .tokens_sold
                .checked_add(remaining_tokens)
                .ok_or(ErrorCode::MathOverflow)?;

            self.stages[new_current_stage].raised_sol = self.stages[new_current_stage]
                .raised_sol
                .checked_add(new_stage_lamports)
                .ok_or(ErrorCode::MathOverflow)?;

            PurchaseResult {
                stage: current_stage as u8,
                tokens: available_stage_amount,
                lamports: current_stage_lamports,
                overflow: Some((
                    self.current_stage,
                    remaining_tokens,
                    new_stage_lamports,
                    self.stages[new_current_stage].locked_pct_bps,
                )),
                locked_pct_bps: self.stages[current_stage].locked_pct_bps,
            }
        };

        self.total_tokens_sold = self
            .total_tokens_sold
            .checked_add(tokens)
            .ok_or(ErrorCode::MathOverflow)?;

        self.total_sol_raised = self
            .total_sol_raised
            .checked_add(lamports)
            .ok_or(ErrorCode::MathOverflow)?;

        msg!(
            "Total tokens sold: {} / {} | Total SOL raised: {}",
            self.total_tokens_sold,
            self.total_tokens_for_sale,
            self.total_sol_raised
        );

        self.check_finalize_sale()?;

        Ok(result)
    }

    pub fn check_finalize_sale(&mut self) -> Result<()> {
        if self.total_tokens_sold == self.total_tokens_for_sale {
            self.sale_finalized = true;
            self.tge_timestamp = Clock::get()?.unix_timestamp;
        }
        Ok(())
    }
}
