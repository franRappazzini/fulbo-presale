use trident_fuzz::fuzzing::*;

use crate::constants;
use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;
use crate::utils::data;

/// Runs all program initialization at the start of each fuzz iteration:
/// mint → config/treasury → 4 beneficiaries (with TreasuryShare) → rewards pool.
pub fn run(trident: &mut Trident, fuzz_accounts: &mut AccountAddresses) {
    let authority = fuzz_accounts.authority.insert(trident, None);
    trident.airdrop(&authority, LAMPORTS_PER_SOL);

    // --- mint ---
    let mint_address = fuzz_accounts.mint.insert(trident, None);
    let mint_ixs = trident.initialize_mint(&authority, &mint_address, 6, &authority, None);

    let tx = trident.process_transaction(&mint_ixs, None);
    assert!(
        tx.is_success(),
        "Failed to initialize mint: {:?}",
        tx.logs()
    );

    // --- main PDAs ---
    let config = fuzz_accounts.config.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::CONFIG_SEED],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let treasury = fuzz_accounts.treasury.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::TREASURY_SEED],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let treasury_ata_address = fuzz_accounts.treasury_ata.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::TREASURY_SEED, mint_address.as_ref()],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let chainlink_feed_address = fuzz_accounts.chainlink_feed.insert(trident, None);

    // --- initialize ---
    let init_ix =
        fulbo_presale::InitializeInstruction::data(fulbo_presale::InitializeInstructionData {
            total_tokens_for_sale: data::total_tokens_for_sale(),
            stages: data::STAGES.clone(),
        })
        .accounts(fulbo_presale::InitializeInstructionAccounts {
            authority,
            config,
            treasury,
            mint: mint_address,
            treasury_ata: treasury_ata_address,
            chainlink_feed: chainlink_feed_address,
            token_program: constants::TOKEN_PROGRAM_ID,
        })
        .instruction();

    let tx = trident.process_transaction(&[init_ix], Some("Initialize"));
    assert!(
        tx.is_success(),
        "Failed to initialize config: {:?}",
        tx.logs()
    );

    // --- beneficiaries with TreasuryShare ---
    let beneficiary_names = ["Team", "Marketing", "Development", "Liquidity"];
    for (i, beneficiary_data) in data::BENEFICIARIES.iter().enumerate() {
        init_beneficiary(
            trident,
            &mut fuzz_accounts.team_beneficiary,
            &mut fuzz_accounts.team_allocation,
            &mut fuzz_accounts.team_treasury_share,
            beneficiary_data,
            authority,
            config,
            &beneficiary_names[i],
        );
    }

    // --- rewards pool (no TreasuryShare) ---
    let rewards_beneficiary = fuzz_accounts.rewards_beneficiary.insert(trident, None);
    let rewards_allocation = fuzz_accounts.rewards_allocation.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[
                constants::BENEFICIARY_ALLOCATION_SEED,
                rewards_beneficiary.as_ref(),
            ],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let ix = fulbo_presale::InitializeRewardsBeneficiaryInstruction::data(
        fulbo_presale::InitializeRewardsBeneficiaryInstructionData {
            total_tokens: 200_000_000_000_000, // 200M
        },
    )
    .accounts(
        fulbo_presale::InitializeRewardsBeneficiaryInstructionAccounts {
            authority,
            beneficiary: rewards_beneficiary,
            config,
            beneficiary_allocation: rewards_allocation,
        },
    )
    .instruction();

    let tx = trident.process_transaction(&[ix], Some("Initialize Rewards Beneficiary"));
    assert!(
        tx.is_success(),
        "Failed to initialize Rewards Beneficiary: {:?}",
        tx.logs()
    );

    // --- mint tokens to treasury ---
    let transfer_ix = trident.mint_to(
        &treasury_ata_address,
        &mint_address,
        &authority,
        300_000_000_000_000,
    ); // 300M to presale

    let tx = trident.process_transaction(&[transfer_ix], None);
    assert!(
        tx.is_success(),
        "Failed to mint tokens to treasury: {:?}",
        tx.logs()
    );
}

/// Initializes a vested beneficiary (with TreasuryShare) and stores the resulting
/// addresses into the provided `AddressStorage` fields so flows can reuse them.
fn init_beneficiary(
    trident: &mut Trident,
    beneficiary_storage: &mut AddressStorage,
    allocation_storage: &mut AddressStorage,
    share_storage: &mut AddressStorage,
    data: &fulbo_presale::InitializeBeneficiaryInstructionData,
    authority: Pubkey,
    config: Pubkey,
    label: &str,
) {
    let beneficiary = beneficiary_storage.insert(trident, None);

    let allocation = allocation_storage.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::BENEFICIARY_ALLOCATION_SEED, beneficiary.as_ref()],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let share = share_storage.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::BENEFICIARY_TREASURY_SEED, beneficiary.as_ref()],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let ix = fulbo_presale::InitializeBeneficiaryInstruction::data(
        fulbo_presale::InitializeBeneficiaryInstructionData {
            total_tokens: data.total_tokens,
            tge_unlock_bps: data.tge_unlock_bps,
            instant_unlock: data.instant_unlock,
            withdraw_interval: data.withdraw_interval,
            sol_share_bps: data.sol_share_bps,
        },
    )
    .accounts(fulbo_presale::InitializeBeneficiaryInstructionAccounts {
        authority,
        beneficiary,
        config,
        beneficiary_allocation: allocation,
        treasury_share: share,
    })
    .instruction();

    let tx = trident.process_transaction(&[ix], Some(&format!("Initialize {} beneficiary", label)));
    assert!(
        tx.is_success(),
        "Failed to initialize {} beneficiary: {:?}",
        label,
        tx.logs()
    );
}
