use trident_fuzz::fuzzing::*;

use crate::constants;
use crate::fuzz_accounts::AccountAddresses;
use crate::types::fulbo_presale;
use crate::utils::data;

/// Runs all program initialization at the start of each fuzz iteration:
/// mint → config/treasury → 4 beneficiaries (with TreasuryShare) → rewards pool
/// → 3 buyers (A, B, C) with their position PDAs and SOL airdrops.
pub fn run(trident: &mut Trident, fuzz_accounts: &mut AccountAddresses) {
    // --- authority ---
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

    let treasury_ata = fuzz_accounts.treasury_ata.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::TREASURY_SEED, mint_address.as_ref()],
            program_id: fulbo_presale::program_id(),
        }),
    );

    let chainlink_feed = fuzz_accounts.chainlink_feed.insert(trident, None);

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
            treasury_ata,
            chainlink_feed,
            token_program: constants::TOKEN_PROGRAM_ID,
        })
        .instruction();

    let tx = trident.process_transaction(&[init_ix], Some("Initialize"));
    assert!(
        tx.is_success(),
        "Failed to initialize config: {:?}",
        tx.logs()
    );

    // --- vested beneficiaries (each with its own TreasuryShare) ---
    init_beneficiary(
        trident,
        &mut fuzz_accounts.team_beneficiary,
        &mut fuzz_accounts.team_allocation,
        &mut fuzz_accounts.team_treasury_share,
        &data::BENEFICIARIES[0],
        authority,
        config,
        "Team",
    );
    init_beneficiary(
        trident,
        &mut fuzz_accounts.marketing_beneficiary,
        &mut fuzz_accounts.marketing_allocation,
        &mut fuzz_accounts.marketing_treasury_share,
        &data::BENEFICIARIES[1],
        authority,
        config,
        "Marketing",
    );
    init_beneficiary(
        trident,
        &mut fuzz_accounts.development_beneficiary,
        &mut fuzz_accounts.development_allocation,
        &mut fuzz_accounts.development_treasury_share,
        &data::BENEFICIARIES[2],
        authority,
        config,
        "Development",
    );
    init_beneficiary(
        trident,
        &mut fuzz_accounts.liquidity_beneficiary,
        &mut fuzz_accounts.liquidity_allocation,
        &mut fuzz_accounts.liquidity_treasury_share,
        &data::BENEFICIARIES[3],
        authority,
        config,
        "Liquidity",
    );

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
        "Failed to initialize rewards beneficiary: {:?}",
        tx.logs()
    );

    // --- mint tokens to treasury (300M for presale) ---
    let transfer_ix = trident.mint_to(
        &treasury_ata,
        &mint_address,
        &authority,
        300_000_000_000_000,
    );
    let tx = trident.process_transaction(&[transfer_ix], None);
    assert!(
        tx.is_success(),
        "Failed to mint tokens to treasury: {:?}",
        tx.logs()
    );

    // --- buyers with their position PDAs ---
    // Each buyer is funded with 100 SOL so they can make many purchases.
    // Their positions are PDA-derived: [POSITION_SEED, buyer_pubkey].
    setup_buyer(
        trident,
        &mut fuzz_accounts.buyer_a,
        &mut fuzz_accounts.position_a,
    );
    setup_buyer(
        trident,
        &mut fuzz_accounts.buyer_b,
        &mut fuzz_accounts.position_b,
    );
    setup_buyer(
        trident,
        &mut fuzz_accounts.buyer_c,
        &mut fuzz_accounts.position_c,
    );
}

/// Initializes a vested beneficiary (with TreasuryShare) and stores the addresses.
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

/// Creates a buyer keypair, airdrops 100 SOL, and stores its position PDA.
fn setup_buyer(
    trident: &mut Trident,
    buyer_storage: &mut AddressStorage,
    position_storage: &mut AddressStorage,
) {
    let buyer = buyer_storage.insert(trident, None);
    trident.airdrop(&buyer, 100 * LAMPORTS_PER_SOL);

    position_storage.insert(
        trident,
        Some(PdaSeeds {
            seeds: &[constants::POSITION_SEED, buyer.as_ref()],
            program_id: fulbo_presale::program_id(),
        }),
    );
}
