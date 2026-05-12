mod constants;
mod flows;
mod fuzz_accounts;
mod types;
mod utils;

use fuzz_accounts::*;
use trident_fuzz::fuzzing::*;

#[derive(FuzzTestMethods)]
struct FuzzTest {
    trident: Trident,
    fuzz_accounts: AccountAddresses,
}

#[flow_executor]
impl FuzzTest {
    fn new() -> Self {
        Self {
            trident: Trident::default(),
            fuzz_accounts: AccountAddresses::default(),
        }
    }

    /// Initializes the full program state: mint, config, treasury, all beneficiaries,
    /// rewards pool, and the three buyers (A, B, C) with their position PDAs.
    #[init]
    fn start(&mut self) {
        flows::setup::run(&mut self.trident, &mut self.fuzz_accounts);
    }

    /// Buyer A purchases a random amount of tokens from the current presale stage.
    /// Transactions may fail legitimately (per-wallet cap, stage exhausted, etc.).
    #[flow]
    fn flow_buy_a(&mut self) {
        flows::buy::execute(
            &mut self.trident,
            &self.fuzz_accounts,
            |a| &a.buyer_a,
            |a| &a.position_a,
        );
    }

    /// Buyer B purchases a random amount of tokens from the current presale stage.
    #[flow]
    fn flow_buy_b(&mut self) {
        flows::buy::execute(
            &mut self.trident,
            &self.fuzz_accounts,
            |a| &a.buyer_b,
            |a| &a.position_b,
        );
    }

    /// Buyer C purchases a random amount of tokens from the current presale stage.
    #[flow]
    fn flow_buy_c(&mut self) {
        flows::buy::execute(
            &mut self.trident,
            &self.fuzz_accounts,
            |a| &a.buyer_c,
            |a| &a.position_c,
        );
    }

    /// Verifies all invariants: token accounting, per-wallet caps, stage consistency,
    /// and that `total_tokens_for_sale` was never modified.
    #[end]
    fn end(&mut self) {
        utils::invariants::check(&mut self.trident, &self.fuzz_accounts);
    }
}

fn main() {
    FuzzTest::fuzz(1000, 100);
}
