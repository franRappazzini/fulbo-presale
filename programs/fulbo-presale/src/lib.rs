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

    pub fn initialize(ctx: Context<Initialize>, stages: [Stage; 11]) -> Result<()> {
        initialize::process(ctx, stages)
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
}
