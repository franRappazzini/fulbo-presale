pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use states::Stage;

declare_id!("BDynxxaaLpprmBdoRaAtsrbXpwfVjXuhXH1Ghhyv5khT");

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
}
