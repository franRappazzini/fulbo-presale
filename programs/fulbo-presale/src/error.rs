use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Failed to read data from Chainlink feed")]
    ChainlinkReadError,
    #[msg("Chainlink feed is missing latest round data")]
    ChainlinkRoundDataMissing,
    #[msg("Mathematical operation overflowed")]
    MathOverflow,
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Invalid price from oracle")]
    InvalidPrice,
    #[msg("Exceeds maximum allowed per wallet for this stage")]
    ExceedsMaxPerWallet,
}
