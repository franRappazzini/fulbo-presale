use anchor_lang::prelude::*;

use crate::constants::DISCRIMINATOR;

#[account]
#[derive(InitSpace)]
pub struct Treasury {
    pub total_sol: u64,
    pub bump: u8,
}

impl Treasury {
    pub const SIZE: usize = DISCRIMINATOR + Treasury::INIT_SPACE;
}
