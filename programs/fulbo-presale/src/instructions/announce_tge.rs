use anchor_lang::prelude::*;

use crate::{
    constants::{CONFIG_SEED, SECONDS_PER_MONTH},
    error::ErrorCode,
    events::TgeAnnounced,
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
        constraint = config.tge_timestamp == 0 @ ErrorCode::TgeAlreadyAnnounced,
        constraint = !config.paused @ ErrorCode::SalePaused,
    )]
    pub config: Account<'info, Config>,
}

pub fn process(ctx: Context<AnnounceTge>) -> Result<()> {
    // claim open 1 month after the tge announcement
    let tge_timestamp = Clock::get()?.unix_timestamp + SECONDS_PER_MONTH as i64;
    ctx.accounts.config.tge_timestamp = tge_timestamp;

    emit!(TgeAnnounced { tge_timestamp });

    Ok(())
}
