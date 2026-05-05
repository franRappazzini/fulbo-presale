use anchor_lang::prelude::*;

use crate::{
    constants::{BENEFICIARY_ALLOCATION_SEED, CONFIG_SEED},
    events::BeneficiaryInitialized,
    states::{BeneficiaryAllocation, Config},
};

#[derive(Accounts)]
pub struct InitializeRewardsBeneficiary<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub beneficiary: SystemAccount<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = authority,
        space = BeneficiaryAllocation::SIZE,
        seeds = [BENEFICIARY_ALLOCATION_SEED, beneficiary.key().as_ref()],
        bump
    )]
    pub beneficiary_allocation: Account<'info, BeneficiaryAllocation>,

    pub system_program: Program<'info, System>,
}

/// Initializes a rewards-only beneficiary.
///
/// Unlike `initialize_beneficiary`, this instruction:
/// - Does NOT create a `TreasuryShare` (no SOL distribution entitlement).
/// - Sets `instant_unlock = true` so the full allocation can be claimed in one
///   call after TGE — matching the behavior expected by `finalize_unsold`.
///
/// `total_tokens` may be 0 if the rewards wallet starts empty and is
/// populated entirely via `finalize_unsold`.
pub fn process(ctx: Context<InitializeRewardsBeneficiary>, total_tokens: u64) -> Result<()> {
    ctx.accounts
        .beneficiary_allocation
        .set_inner(BeneficiaryAllocation {
            total_tokens,
            withdrawn_tokens: 0,
            // not used when instant_unlock = true, set to 0 explicitly
            monthly_unlocked: 0,
            // not used when instant_unlock = true, set to 10_000 as a no-op convention
            tge_unlock_bps: 10_000,
            instant_unlock: true,
            bump: ctx.bumps.beneficiary_allocation,
        });

    emit!(BeneficiaryInitialized {
        beneficiary: ctx.accounts.beneficiary.key(),
        total_tokens,
        tge_unlock_bps: 10_000,
        sol_share_bps: 0,
    });

    Ok(())
}
