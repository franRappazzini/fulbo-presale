use trident_fuzz::fuzzing::*;

/// Centralized storage for all account addresses used across fuzz flows.
///
/// Grouped by role so every flow can access exactly the accounts it needs:
/// - Program singletons: initialized once in setup, shared across all flows.
/// - Named beneficiaries: each has its own pubkey + BeneficiaryAllocation PDA
///   and (for vested ones) a TreasuryShare PDA.
/// - Buyer/claimer: the wallet used in presale and claim flows.
///
/// Docs: https://ackee.xyz/trident/docs/latest/trident-api-macro/trident-types/fuzz-accounts/
#[derive(Default)]
pub struct AccountAddresses {
    // ── Program singletons ───────────────────────────────────────────────
    pub authority: AddressStorage,
    pub config: AddressStorage,
    pub mint: AddressStorage,
    pub treasury: AddressStorage,
    pub treasury_ata: AddressStorage,
    pub chainlink_feed: AddressStorage,

    // ── Named beneficiaries ──────────────────────────────────────────────
    // team
    pub team_beneficiary: AddressStorage,
    pub team_allocation: AddressStorage,
    pub team_treasury_share: AddressStorage,
    // marketing
    pub marketing_beneficiary: AddressStorage,
    pub marketing_allocation: AddressStorage,
    pub marketing_treasury_share: AddressStorage,
    // development
    pub development_beneficiary: AddressStorage,
    pub development_allocation: AddressStorage,
    pub development_treasury_share: AddressStorage,
    // liquidity
    pub liquidity_beneficiary: AddressStorage,
    pub liquidity_allocation: AddressStorage,
    pub liquidity_treasury_share: AddressStorage,
    // rewards pool (no TreasuryShare)
    pub rewards_beneficiary: AddressStorage,
    pub rewards_allocation: AddressStorage,

    // ── Buyer / claimer ──────────────────────────────────────────────────
    pub buyer: AddressStorage,
    pub position: AddressStorage,
    pub buyer_ata: AddressStorage,

    // ── SPL / system programs ────────────────────────────────────────────
    pub beneficiary_ata: AddressStorage,
    pub associated_token_program: AddressStorage,
    pub token_program: AddressStorage,
    pub system_program: AddressStorage,
}
