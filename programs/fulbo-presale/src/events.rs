use anchor_lang::prelude::*;

#[event]
pub struct TokenPurchased {
    pub buyer: Pubkey,
    pub stage: u8,
    pub tokens: u64,
    pub lamports: u64,
}

#[event]
pub struct TokensClaimed {
    pub claimer: Pubkey,
    pub total_claimable: u64,
}

#[event]
pub struct TgeAnnounced {
    pub tge_timestamp: i64,
}

#[event]
pub struct SaleFinalized {
    pub tge_timestamp: i64,
    pub total_tokens_sold: u64,
    pub total_sol_raised: u64,
}

#[event]
pub struct SalePaused {}

#[event]
pub struct SaleUnpaused {}

#[event]
pub struct BeneficiaryInitialized {
    pub beneficiary: Pubkey,
    pub total_tokens: u64,
    pub tge_unlock_bps: u16,
    pub sol_share_bps: u16,
}

#[event]
pub struct TreasuryWithdrawn {
    pub beneficiary: Pubkey,
    pub amount: u64,
}

#[event]
pub struct BeneficiaryTokensClaimed {
    pub beneficiary: Pubkey,
    pub amount: u64,
}

#[event]
pub struct UnsoldTokensFinalized {
    pub burned: u64,
    pub rewarded: u64,
}
