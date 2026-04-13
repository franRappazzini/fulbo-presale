use anchor_lang::prelude::*;

use crate::{Config, Stage, CONFIG_SEED};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Config::SIZE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    pub mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<Initialize>, stages: [Stage; 11]) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.authority = ctx.accounts.authority.key();
    config.mint = ctx.accounts.mint.key();
    config.stages = stages;
    config.bump = ctx.bumps.config;

    Ok(())
}
