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
    #[msg("TGE has not started yet")]
    TgeNotStarted,
    #[msg("No tokens available to claim at this time")]
    NothingToClaim,
    #[msg("Sale has already been finalized")]
    SaleAlreadyFinalized,
    #[msg("TGE has already been announced")]
    TgeAlreadyAnnounced,
    #[msg("Sale is currently paused")]
    SalePaused,
    #[msg("Oracle price feed data is stale")]
    StalePriceFeed,
    #[msg("Insufficient token supply in the current or next stage")]
    InsufficientStageSupply,
}
