use trident_fuzz::fuzzing::*;

use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;

/// A named beneficiary withdraws their SOL share from the treasury.
/// Pick randomly among team / marketing / development / liquidity.
/// (Rewards has no TreasuryShare.)
///
/// Required accounts: beneficiary, treasury_share (PDA: BENEFICIARY_TREASURY_SEED + beneficiary),
///                    treasury, config.
pub fn withdraw_treasury(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let config = accounts.config.get(trident);
    let treasury = accounts.treasury.get(trident);

    // TODO: pick a beneficiary and build the instruction.
    // Example for team:
    // let beneficiary   = accounts.team_beneficiary.get();
    // let treasury_share = accounts.team_treasury_share.get();
    // let ix = fulbo_presale::WithdrawTreasuryInstruction::data(...)
    //     .accounts(fulbo_presale::WithdrawTreasuryInstructionAccounts {
    //         beneficiary, beneficiary_treasury: treasury_share, treasury, config, ...
    //     })
    //     .instruction();
    // trident.process_transaction(&[ix], Some("WithdrawTreasury"));
    let _ = (trident, config, treasury);
}

// Suppress unused import warning until flows are implemented.
fn _use_types() { let _ = fulbo_presale::program_id(); }
