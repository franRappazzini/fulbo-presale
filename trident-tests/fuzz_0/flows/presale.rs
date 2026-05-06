use trident_fuzz::fuzzing::*;

use crate::constants;
use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;

/// Executes a buy_token instruction: a random buyer purchases tokens
/// from the current presale stage.
///
/// Required accounts: buyer, position (PDA: POSITION_SEED + buyer),
///                    config, treasury, chainlink_feed.
pub fn buy_token(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let buyer = accounts.buyer.insert(trident, None);

    let position = accounts.position.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::POSITION_SEED, buyer.as_ref()],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let config = accounts.config.get(trident);
    let treasury = accounts.treasury.get(trident);
    let chainlink_feed = accounts.chainlink_feed.get(trident);

    // TODO: build and send buy_token instruction
    // Example:
    // let ix = fulbo_presale::BuyTokenInstruction::data(
    //     fulbo_presale::BuyTokenInstructionData { amount: <random_amount> },
    // )
    // .accounts(fulbo_presale::BuyTokenInstructionAccounts {
    //     buyer,
    //     position,
    //     config,
    //     treasury,
    //     chainlink_feed,
    //     system_program: system_program::id(),
    // })
    // .instruction();
    // trident.process_transaction(&[ix], Some("BuyToken"));
    let _ = (buyer, position, config, treasury, chainlink_feed);
}
