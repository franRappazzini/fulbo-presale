use trident_fuzz::fuzzing::*;

use crate::fuzz_accounts::AccountAddresses;
use crate::types::{Config, Position};
use crate::utils::data;

/// Verifies all global and per-buyer invariants at the end of each fuzz iteration.
///
/// Invariants checked:
/// 1. `config.total_tokens_sold` <= `config.total_tokens_for_sale`  (no overselling)
/// 2. `config.total_tokens_sold` == sum of all buyer position `total_tokens`  (bookkeeping)
/// 3. Per-stage: `stage.tokens_sold` == sum of positions' `stage_allocations[i].tokens`
/// 4. Per-buyer per-stage: allocation <= `stage.max_wallet_pct_bps` cap
/// 5. `config.current_stage` is within bounds (0..=10)
/// 6. `config.total_tokens_for_sale` was not modified after initialization
pub fn check(trident: &mut Trident, accounts: &AccountAddresses) {
    println!("--- Checking invariants ---");

    let config_key = accounts
        .config
        .get(trident)
        .expect("config address not set");

    let config = trident
        .get_account_with_type::<Config>(&config_key, 8)
        .expect("config account must exist after initialization");

    // Invariant 1: no overselling
    assert!(
        config.total_tokens_sold <= config.total_tokens_for_sale,
        "Oversell detected: total_tokens_sold ({}) > total_tokens_for_sale ({})",
        config.total_tokens_sold,
        config.total_tokens_for_sale,
    );

    // Invariant 5: stage index in bounds
    assert!(
        config.current_stage <= 10,
        "current_stage ({}) is out of bounds",
        config.current_stage,
    );

    // Invariant 6: total_tokens_for_sale is immutable
    assert_eq!(
        config.total_tokens_for_sale,
        data::total_tokens_for_sale(),
        "total_tokens_for_sale was unexpectedly modified",
    );

    // Read all three buyer positions. A position may not exist on-chain yet
    // if that buyer never completed a successful purchase.
    let position_keys = [
        accounts.position_a.get(trident),
        accounts.position_b.get(trident),
        accounts.position_c.get(trident),
    ];

    let positions: Vec<Option<Position>> = position_keys
        .iter()
        .map(|opt_key| {
            opt_key
                .as_ref()
                .and_then(|key| trident.get_account_with_type::<Position>(key, 8))
        })
        .collect();

    // Invariant 2: config.total_tokens_sold == sum of positions
    let total_from_positions: u64 = positions
        .iter()
        .filter_map(|p| p.as_ref())
        .map(|p| p.total_tokens)
        .sum();

    assert_eq!(
        config.total_tokens_sold, total_from_positions,
        "Token accounting mismatch: config.total_tokens_sold ({}) != sum of positions ({})",
        config.total_tokens_sold, total_from_positions,
    );

    // Invariant 3 & 4: per-stage consistency and per-wallet cap
    for (stage_idx, stage) in config.stages.iter().enumerate() {
        // Sum tokens from all positions for this stage
        let stage_tokens_from_positions: u64 = positions
            .iter()
            .filter_map(|p| p.as_ref())
            .map(|p| p.stage_allocations[stage_idx].tokens)
            .sum();

        assert_eq!(
            stage.tokens_sold, stage_tokens_from_positions,
            "Stage {}: tokens_sold ({}) != sum of position allocations ({})",
            stage_idx, stage.tokens_sold, stage_tokens_from_positions,
        );

        // Maximum tokens any single buyer may hold for this stage
        let max_per_wallet = (stage.max_tokens as u128)
            .saturating_mul(stage.max_wallet_pct_bps as u128)
            .saturating_div(10_000) as u64;

        for (buyer_idx, position) in positions.iter().filter_map(|p| p.as_ref()).enumerate() {
            let allocation = position.stage_allocations[stage_idx].tokens;
            assert!(
                allocation <= max_per_wallet,
                "Buyer {buyer_idx} exceeded per-wallet cap in stage {stage_idx}: \
                 allocation ({allocation}) > max_per_wallet ({max_per_wallet})",
            );
        }

        // No stage can have more tokens sold than its maximum supply
        assert!(
            stage.tokens_sold <= stage.max_tokens,
            "Stage {stage_idx} oversold: tokens_sold ({}) > max_tokens ({})",
            stage.tokens_sold,
            stage.max_tokens,
        );
    }
}
