use trident_fuzz::fuzzing::*;

use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;
use crate::{constants, types};

/// Executes a buy_token instruction: a random buyer purchases tokens
/// from the current presale stage.
///
/// Required accounts: buyer, position (PDA: POSITION_SEED + buyer),
///                    config, treasury, chainlink_feed.
pub fn buy_token(trident: &mut Trident, accounts: &mut AccountAddresses) {
    let buyer = accounts.buyer.insert(trident, None);
    trident.airdrop(&buyer, 5 * LAMPORTS_PER_SOL);

    let position = accounts.position.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::POSITION_SEED, buyer.as_ref()],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let config = accounts.config.get(trident).unwrap();
    let treasury = accounts.treasury.get(trident).unwrap();
    let chainlink_feed = accounts.chainlink_feed.get(trident).unwrap();

    // get max tokens allowed for current stage
    let config_account = trident
        .get_account_with_type::<types::Config>(&config, 8)
        .unwrap();

    let current_stage = &config_account.stages[config_account.current_stage as usize];
    let max_tokens_allowed = current_stage
        .max_tokens
        .checked_mul(current_stage.max_wallet_pct_bps as u64)
        .unwrap()
        .checked_div(10_000)
        .unwrap();

    let rdm_amount = trident.random_from_range(0..=max_tokens_allowed);
    let ix = fulbo_presale::BuyTokenInstruction::data(fulbo_presale::BuyTokenInstructionData {
        amount: rdm_amount,
    })
    .accounts(fulbo_presale::BuyTokenInstructionAccounts {
        buyer,
        position,
        config,
        treasury,
        chainlink_feed,
    })
    .instruction();

    let tx = trident.process_transaction(&[ix], Some("BuyToken"));
    assert!(
        tx.is_success(),
        "BuyToken transaction failed: {:?}",
        tx.logs()
    );
}
