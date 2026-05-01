pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use states::Stage;

declare_id!("4q6B6GdbbijEHxnhXVx8mDsiHfgk5b9bCVB8PqtA1bDJ");

#[program]
pub mod fulbo_presale {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        total_tokens_for_sale: u64,
        stages: [Stage; 11],
    ) -> Result<()> {
        initialize::process(ctx, total_tokens_for_sale, stages)
    }

    /// Purchases tokens during the presale.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context containing accounts and program state
    /// * `amount` - The amount of tokens to purchase (not SOL amount)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful purchase, or an error if the transaction fails.
    pub fn buy_token(ctx: Context<BuyToken>, amount: u64) -> Result<()> {
        buy_token::process(ctx, amount)
    }

    pub fn claim_token(ctx: Context<ClaimToken>) -> Result<()> {
        claim_token::process(ctx)
    }

    pub fn announce_tge(ctx: Context<AnnounceTge>) -> Result<()> {
        announce_tge::process(ctx)
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        pause::process(ctx)
    }

    pub fn initialize_beneficiary(
        ctx: Context<InitializeBeneficiary>,
        total_tokens: u64,
        tge_unlock_bps: u16,
        is_liquidity: bool,
        withdraw_interval: i64,
        sol_share_bps: u16,
    ) -> Result<()> {
        initialize_beneficiary::process(
            ctx,
            total_tokens,
            tge_unlock_bps,
            is_liquidity,
            withdraw_interval,
            sol_share_bps,
        )
    }

    pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>) -> Result<()> {
        withdraw_treasury::process(ctx)
    }

    pub fn beneficiary_claim(ctx: Context<BeneficiaryClaim>) -> Result<()> {
        beneficiary_claim::process(ctx)
    }
}
