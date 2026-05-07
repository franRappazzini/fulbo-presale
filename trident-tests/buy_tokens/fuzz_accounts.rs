use trident_fuzz::fuzzing::*;

/// Storage for all account addresses used in the buy_tokens fuzz test.
///
/// Singletons (config, treasury, mint, chainlink) are set up once in `start`.
/// Beneficiary accounts are set up once in `start` and never used in flows.
/// Three named buyers (A, B, C) buy tokens in parallel across flow calls.
#[derive(Default)]
pub struct AccountAddresses {
    // Program admin + core PDAs
    pub authority: AddressStorage,
    pub config: AddressStorage,
    pub mint: AddressStorage,
    pub treasury: AddressStorage,
    pub treasury_ata: AddressStorage,
    pub chainlink_feed: AddressStorage,

    // Vested beneficiaries (setup only — not used in buy flows)
    pub team_beneficiary: AddressStorage,
    pub team_allocation: AddressStorage,
    pub team_treasury_share: AddressStorage,
    pub marketing_beneficiary: AddressStorage,
    pub marketing_allocation: AddressStorage,
    pub marketing_treasury_share: AddressStorage,
    pub development_beneficiary: AddressStorage,
    pub development_allocation: AddressStorage,
    pub development_treasury_share: AddressStorage,
    pub liquidity_beneficiary: AddressStorage,
    pub liquidity_allocation: AddressStorage,
    pub liquidity_treasury_share: AddressStorage,
    pub rewards_beneficiary: AddressStorage,
    pub rewards_allocation: AddressStorage,

    // Buyer A with its position PDA
    pub buyer_a: AddressStorage,
    pub position_a: AddressStorage,

    // Buyer B with its position PDA
    pub buyer_b: AddressStorage,
    pub position_b: AddressStorage,

    // Buyer C with its position PDA
    pub buyer_c: AddressStorage,
    pub position_c: AddressStorage,
}
