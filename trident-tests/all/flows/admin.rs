use trident_fuzz::fuzzing::*;

use crate::constants;
use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;

/// Toggles the paused state of the presale.
///
/// Required accounts: authority, config.
pub fn pause(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let authority = accounts.authority.get(trident).unwrap();
    let config = accounts.config.get(trident).unwrap();

    let ix = fulbo_presale::PauseInstruction::data(fulbo_presale::PauseInstructionData {})
        .accounts(fulbo_presale::PauseInstructionAccounts { authority, config })
        .instruction();

    let tx = trident.process_transaction(&[ix], Some("Pause"));
    assert!(tx.is_success(), "Pause transaction failed: {:?}", tx.logs());
}

/// Sets the TGE timestamp, starting the vesting clock.
/// Should only be called once after the presale ends.
///
/// Required accounts: authority, config.
pub fn announce_tge(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let authority = accounts.authority.get(trident).unwrap();
    let config = accounts.config.get(trident).unwrap();

    let announce_tge_ix =
        fulbo_presale::AnnounceTgeInstruction::data(fulbo_presale::AnnounceTgeInstructionData {})
            .accounts(fulbo_presale::AnnounceTgeInstructionAccounts { authority, config })
            .instruction();

    let tx = trident.process_transaction(&[announce_tge_ix], Some("AnnounceTGE"));
    assert!(
        tx.is_success(),
        "AnnounceTGE transaction failed: {:?}",
        tx.logs()
    );
}

/// Burns 60 % of unsold tokens and routes 40 % to the rewards pool.
/// Should only be called once after TGE.
///
/// Required accounts: authority, config, rewards_beneficiary,
///                    rewards_allocation, mint, treasury_ata.
pub fn finalize_unsold(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let authority = accounts.authority.get(trident).unwrap();
    let config = accounts.config.get(trident).unwrap();
    let rewards_beneficiary = accounts.rewards_beneficiary.get(trident).unwrap();
    let rewards_allocation = accounts.rewards_allocation.get(trident).unwrap();
    let mint = accounts.mint.get(trident).unwrap();
    let treasury_ata = accounts.treasury_ata.get(trident).unwrap();

    let finalize_unsold_ix = fulbo_presale::FinalizeUnsoldInstruction::data(
        fulbo_presale::FinalizeUnsoldInstructionData {},
    )
    .accounts(fulbo_presale::FinalizeUnsoldInstructionAccounts {
        authority,
        rewards_beneficiary,
        config,
        beneficiary_allocation: rewards_allocation,
        mint,
        treasury_ata,
        token_program: constants::TOKEN_PROGRAM_ID,
    })
    .instruction();

    let tx = trident.process_transaction(&[finalize_unsold_ix], Some("FinalizeUnsold"));
    assert!(
        tx.is_success(),
        "FinalizeUnsold transaction failed: {:?}",
        tx.logs()
    );
}

// Suppress unused import warning until flows are implemented.
fn _use_types() {
    let _ = fulbo_presale::program_id();
}
