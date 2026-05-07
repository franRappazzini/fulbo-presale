use trident_fuzz::fuzzing::*;

use crate::fuzz_accounts::AccountAddresses;
use crate::types::{fulbo_presale, Config};

/// Sends a BuyToken instruction for the given buyer.
///
/// The amount is chosen randomly in the range `1..=max_per_wallet` for the current
/// stage, so it is always a structurally valid amount. The program may still reject
/// the transaction (e.g. the buyer already hit the per-wallet cap in this stage).
/// Failed transactions are intentionally ignored — the `end` invariants will detect
/// any state inconsistency they would cause.
pub fn execute(
    trident: &mut Trident,
    accounts: &AccountAddresses,
    buyer_storage: fn(&AccountAddresses) -> &AddressStorage,
    position_storage: fn(&AccountAddresses) -> &AddressStorage,
) {
    let buyer = match buyer_storage(accounts).get(trident) {
        Some(pk) => pk,
        None => return,
    };
    let config = match accounts.config.get(trident) {
        Some(pk) => pk,
        None => return,
    };
    let treasury = match accounts.treasury.get(trident) {
        Some(pk) => pk,
        None => return,
    };
    let position = match position_storage(accounts).get(trident) {
        Some(pk) => pk,
        None => return,
    };
    let chainlink_feed = match accounts.chainlink_feed.get(trident) {
        Some(pk) => pk,
        None => return,
    };

    // Read the config to derive the current stage's per-wallet cap.
    let config_account = match trident.get_account_with_type::<Config>(&config, 8) {
        Some(acc) => acc,
        None => return,
    };

    let current_stage = &config_account.stages[config_account.current_stage as usize];
    let max_per_wallet = current_stage
        .max_tokens
        .saturating_mul(current_stage.max_wallet_pct_bps as u64)
        .saturating_div(10_000);

    if max_per_wallet == 0 {
        return;
    }

    // Pick a random amount within the per-wallet cap.
    // The program may still reject if the buyer's position already holds the maximum.
    let amount = trident.random_from_range(1..=max_per_wallet);

    let ix =
        fulbo_presale::BuyTokenInstruction::data(fulbo_presale::BuyTokenInstructionData { amount })
            .accounts(fulbo_presale::BuyTokenInstructionAccounts {
                buyer,
                config,
                treasury,
                position,
                chainlink_feed,
            })
            .instruction();

    // Result is intentionally ignored — failed buys are expected (per-wallet cap,
    // stage exhaustion, etc.) and the end invariants verify overall consistency.
    let tx = trident.process_transaction(&[ix], Some("BuyToken"));
    assert!(
        tx.is_success(),
        "Failed to execute BuyToken instruction: {:?}",
        tx.logs()
    );
}
