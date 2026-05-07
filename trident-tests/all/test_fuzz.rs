mod constants;
mod flows;
mod fuzz_accounts;
mod setup;
mod types;
mod utils;

use fuzz_accounts::*;
use trident_fuzz::fuzzing::*;

#[derive(FuzzTestMethods)]
struct FuzzTest {
    /// Trident client for interacting with the Solana program
    trident: Trident,
    /// Storage for all account addresses used in fuzz testing
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

    #[init]
    fn start(&mut self) {
        setup::run(&mut self.trident, &mut self.fuzz_accounts);
    }

    // ── Presale ─────────────────────────────────────────────────────────────
    #[flow]
    fn flow_buy_token(&mut self) {
        flows::presale::buy_token(&mut self.trident, &mut self.fuzz_accounts);
    }

    // ── Admin ────────────────────────────────────────────────────────────────
    #[flow]
    fn flow_pause(&mut self) {
        flows::admin::pause(&mut self.trident, &mut self.fuzz_accounts);
    }

    #[flow]
    fn flow_announce_tge(&mut self) {
        flows::admin::announce_tge(&mut self.trident, &mut self.fuzz_accounts);
    }

    #[flow]
    fn flow_finalize_unsold(&mut self) {
        flows::admin::finalize_unsold(&mut self.trident, &mut self.fuzz_accounts);
    }

    // ── Claims ───────────────────────────────────────────────────────────────
    #[flow]
    fn flow_claim_token(&mut self) {
        flows::claim::claim_token(&mut self.trident, &mut self.fuzz_accounts);
    }

    #[flow]
    fn flow_beneficiary_claim(&mut self) {
        flows::claim::beneficiary_claim(&mut self.trident, &mut self.fuzz_accounts);
    }

    // ── Withdraw ─────────────────────────────────────────────────────────────
    #[flow]
    fn flow_withdraw_treasury(&mut self) {
        flows::withdraw::withdraw_treasury(&mut self.trident, &mut self.fuzz_accounts);
    }

    #[end]
    fn end(&mut self) {}
}

fn main() {
    FuzzTest::fuzz(1000, 100);
}
