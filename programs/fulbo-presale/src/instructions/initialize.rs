use anchor_lang::prelude::*;

use crate::{
    constants::{CONFIG_SEED, TREASURY_SEED},
    states::{Config, Stage, Treasury},
};

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

    #[account(
        init,
        payer = authority,
        space = Treasury::SIZE,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub treasury: Account<'info, Treasury>,

    /// CHECK: just saving the mint address
    pub mint: UncheckedAccount<'info>,

    /// CHECK: just saving the chainlink feed address
    pub chainlink_feed: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<Initialize>, stages: [Stage; 11]) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.mint = ctx.accounts.mint.key();
    config.chainlink_feed = ctx.accounts.chainlink_feed.key();
    config.stages = stages;
    config.bump = ctx.bumps.config;

    let treasury = &mut ctx.accounts.treasury;
    treasury.bump = ctx.bumps.treasury;

    Ok(())
}
