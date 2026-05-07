use trident_fuzz::fuzzing::*;

use crate::constants;
use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;

/// Buyer claims their vested tokens based on elapsed time since TGE.
///
/// Required accounts: buyer (claimer), position, config, mint,
///                    treasury_ata, buyer_ata.
pub fn claim_token(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let buyer = accounts.buyer.get(trident).unwrap();
    let position = accounts.position.get(trident).unwrap();
    let config = accounts.config.get(trident).unwrap();
    let mint = accounts.mint.get(trident).unwrap();
    let treasury_ata = accounts.treasury_ata.get(trident).unwrap();
    let buyer_ata = accounts.buyer_ata.get(trident).unwrap();

    let ix =
        fulbo_presale::ClaimTokenInstruction::data(fulbo_presale::ClaimTokenInstructionData {})
            .accounts(fulbo_presale::ClaimTokenInstructionAccounts {
                claimer: buyer,
                position,
                config,
                mint,
                treasury_ata,
                claimer_ata: buyer_ata,
                token_program: constants::TOKEN_PROGRAM_ID,
            })
            .instruction();

    let tx = trident.process_transaction(&[ix], Some("ClaimToken"));
    assert!(
        tx.is_success(),
        "Failed to execute ClaimToken instruction: {:?}",
        tx.logs()
    );
}

/// A named beneficiary claims their vested token allocation.
/// Call once per beneficiary or pick randomly among team/marketing/development/liquidity/rewards.
///
/// Required accounts: beneficiary, beneficiary_allocation, config,
///                    mint, treasury_ata, beneficiary_ata.
pub fn beneficiary_claim(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let config = accounts.config.get(trident).unwrap();
    let mint = accounts.mint.get(trident).unwrap();
    let treasury_ata = accounts.treasury_ata.get(trident).unwrap();

    let beneficiary = accounts.team_beneficiary.get(trident).unwrap();
    let allocation = accounts.team_allocation.get(trident).unwrap();
    let beneficiary_ata = accounts.beneficiary_ata.get(trident).unwrap();

    let ix = fulbo_presale::BeneficiaryClaimInstruction::data(
        fulbo_presale::BeneficiaryClaimInstructionData {},
    )
    .accounts(fulbo_presale::BeneficiaryClaimInstructionAccounts {
        beneficiary,
        beneficiary_allocation: allocation,
        config,
        mint,
        treasury_ata,
        beneficiary_ata,
        token_program: constants::TOKEN_PROGRAM_ID,
    })
    .instruction();

    let tx = trident.process_transaction(&[ix], Some("BeneficiaryClaim"));
    assert!(
        tx.is_success(),
        "Failed to execute BeneficiaryClaim instruction: {:?}",
        tx.logs()
    );
}

// Suppress unused import warning until flows are implemented.
fn _use_types() {
    let _ = fulbo_presale::program_id();
}
