use anchor_lang::prelude::*;

use crate::{
    constants::CONFIG_SEED,
    events::{SalePaused, SaleUnpaused},
    states::Config,
};

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority,
    )]
    pub config: Account<'info, Config>,
}

pub fn process(ctx: Context<Pause>) -> Result<()> {
    ctx.accounts.config.paused = !ctx.accounts.config.paused;

    if ctx.accounts.config.paused {
        emit!(SalePaused {});
    } else {
        emit!(SaleUnpaused {});
    }

    Ok(())
}
