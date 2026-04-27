use anchor_lang::prelude::*;

use crate::{
    constants::{CONFIG_SEED, SECONDS_PER_MONTH},
    error::ErrorCode,
    states::Config,
};

#[derive(Accounts)]
pub struct AnnounceTge<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority,
        constraint = !config.sale_finalized @ ErrorCode::SaleAlreadyFinalized,
        constraint = config.tge_announced_timestamp == 0 @ ErrorCode::TgeAlreadyAnnounced,
        constraint = !config.paused @ ErrorCode::SalePaused,
    )]
    pub config: Account<'info, Config>,
}

pub fn process(ctx: Context<AnnounceTge>) -> Result<()> {
    ctx.accounts.config.tge_announced_timestamp =
        Clock::get()?.unix_timestamp + SECONDS_PER_MONTH as i64; // the time when the first claim will be available (1 month after TGE announcement)

    Ok(())
}
