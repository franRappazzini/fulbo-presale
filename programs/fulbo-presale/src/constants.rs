use anchor_lang::constant;

pub const DISCRIMINATOR: usize = 8;

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const TREASURY_SEED: &[u8] = b"treasury";

#[constant]
pub const POSITION_SEED: &[u8] = b"position";

/// One "month" for vesting purposes = 30 days in seconds.
#[constant]
pub const SECONDS_PER_MONTH: u32 = 30 * 24 * 60 * 60; // 2_592_000

/// Each month unlocks an additional 5% of total tokens (500 bps).
#[constant]
pub const MONTHLY_UNLOCK_BPS: u16 = 500;
