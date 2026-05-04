use anchor_lang::constant;

pub const DISCRIMINATOR: usize = 8;

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const TREASURY_SEED: &[u8] = b"treasury";

#[constant]
pub const POSITION_SEED: &[u8] = b"position";

#[constant]
pub const BENEFICIARY_ALLOCATION_SEED: &[u8] = b"beneficiary_allocation";

#[constant]
pub const BENEFICIARY_TREASURY_SEED: &[u8] = b"beneficiary_treasury";

/// one "month" for vesting purposes = 30 days in seconds.
#[constant]
pub const SECONDS_PER_MONTH: u32 = 30 * 24 * 60 * 60; // 2_592_000

/// each month unlocks an additional 5% of total tokens (500 bps).
#[constant]
pub const MONTHLY_UNLOCK_BPS: u16 = 500;

/// maximum age in seconds of a chainlink price feed round
#[constant]
pub const MAX_ORACLE_STALENESS: i64 = 3_600;
