use anchor_lang::prelude::*;

use crate::constants::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub chainlink_feed: Pubkey,

    pub stages: [Stage; 11],

    pub tge_timestamp: i64,

    pub total_sol_raised: u64,
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

impl Config {
    pub const SIZE: usize = DISCRIMINATOR + Config::INIT_SPACE;
}
