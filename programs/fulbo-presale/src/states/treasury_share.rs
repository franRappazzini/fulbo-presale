use anchor_lang::prelude::*;

use crate::constants::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct TreasuryShare {
    pub sol_withdrawn: u64,
    pub last_sol_claim: i64, // timestamp
    pub presale_start: i64,
    pub withdraw_interval: i64,
    pub sol_share_bps: u16, // based on the total treasury
    pub instant_unlock: bool,
    pub bump: u8,
}

impl TreasuryShare {
    pub const SIZE: usize = DISCRIMINATOR + TreasuryShare::INIT_SPACE;
}
