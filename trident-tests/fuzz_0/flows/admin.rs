use trident_fuzz::fuzzing::*;

use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;

/// Toggles the paused state of the presale.
///
/// Required accounts: authority, config.
pub fn pause(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let authority = accounts.authority.get(trident);
    let config = accounts.config.get(trident);

    // TODO: build and send pause instruction
    // let ix = fulbo_presale::PauseInstruction::data(fulbo_presale::PauseInstructionData {})
    //     .accounts(fulbo_presale::PauseInstructionAccounts { authority, config })
    //     .instruction();
    // trident.process_transaction(&[ix], Some("Pause"));
    let _ = (trident, authority, config);
}

/// Sets the TGE timestamp, starting the vesting clock.
/// Should only be called once after the presale ends.
///
/// Required accounts: authority, config.
pub fn announce_tge(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let authority = accounts.authority.get(trident);
    let config = accounts.config.get(trident);

    // TODO: build and send announce_tge instruction
    let _ = (trident, authority, config);
}

/// Burns 60 % of unsold tokens and routes 40 % to the rewards pool.
/// Should only be called once after TGE.
///
/// Required accounts: authority, config, rewards_beneficiary,
///                    rewards_allocation, mint, treasury_ata.
pub fn finalize_unsold(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let authority = accounts.authority.get(trident);
    let config = accounts.config.get(trident);
    let rewards_beneficiary = accounts.rewards_beneficiary.get(trident);
    let rewards_allocation = accounts.rewards_allocation.get(trident);
    let mint = accounts.mint.get(trident);
    let treasury_ata = accounts.treasury_ata.get(trident);

    // TODO: build and send finalize_unsold instruction
    let _ = (trident, authority, config, rewards_beneficiary, rewards_allocation, mint, treasury_ata);
}

// Suppress unused import warning until flows are implemented.
fn _use_types() { let _ = fulbo_presale::program_id(); }
