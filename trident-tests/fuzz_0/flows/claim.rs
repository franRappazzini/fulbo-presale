use trident_fuzz::fuzzing::*;

use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;

/// Buyer claims their vested tokens based on elapsed time since TGE.
///
/// Required accounts: buyer (claimer), position, config, mint,
///                    treasury_ata, buyer_ata.
pub fn claim_token(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let buyer = accounts.buyer.get(trident);
    let position = accounts.position.get(trident);
    let config = accounts.config.get(trident);
    let mint = accounts.mint.get(trident);
    let treasury_ata = accounts.treasury_ata.get(trident);
    let buyer_ata = accounts.buyer_ata.get(trident);

    // TODO: build and send claim_token instruction
    // let ix = fulbo_presale::ClaimTokenInstruction::data(...)
    //     .accounts(fulbo_presale::ClaimTokenInstructionAccounts {
    //         claimer: buyer, position, config, mint, treasury_ata,
    //         claimer_ata: buyer_ata, ...
    //     })
    //     .instruction();
    // trident.process_transaction(&[ix], Some("ClaimToken"));
    let _ = (trident, buyer, position, config, mint, treasury_ata, buyer_ata);
}

/// A named beneficiary claims their vested token allocation.
/// Call once per beneficiary or pick randomly among team/marketing/development/liquidity/rewards.
///
/// Required accounts: beneficiary, beneficiary_allocation, config,
///                    mint, treasury_ata, beneficiary_ata.
pub fn beneficiary_claim(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let config = accounts.config.get(trident);
    let mint = accounts.mint.get(trident);
    let treasury_ata = accounts.treasury_ata.get(trident);

    // TODO: pick a beneficiary (e.g. randomly) and build the instruction.
    // Example for team:
    // let beneficiary = accounts.team_beneficiary.get();
    // let allocation  = accounts.team_allocation.get();
    // let beneficiary_ata = accounts.beneficiary_ata.get();
    // let ix = fulbo_presale::BeneficiaryClaimInstruction::data(...)
    //     .accounts(fulbo_presale::BeneficiaryClaimInstructionAccounts {
    //         beneficiary, beneficiary_allocation: allocation, config,
    //         mint, treasury_ata, beneficiary_ata, ...
    //     })
    //     .instruction();
    // trident.process_transaction(&[ix], Some("BeneficiaryClaim"));
    let _ = (trident, config, mint, treasury_ata);
}

// Suppress unused import warning until flows are implemented.
fn _use_types() { let _ = fulbo_presale::program_id(); }
