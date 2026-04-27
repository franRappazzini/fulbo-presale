use anchor_lang::prelude::*;

use crate::{constants::CONFIG_SEED, states::Config};

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

    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<Pause>) -> Result<()> {
    ctx.accounts.config.paused = !ctx.accounts.config.paused;

    Ok(())
}
