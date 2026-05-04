use anchor_lang::prelude::*;

use crate::{
    constants::{
        BENEFICIARY_ALLOCATION_SEED, BENEFICIARY_TREASURY_SEED, CONFIG_SEED, MONTHLY_UNLOCK_BPS,
    },
    error::ErrorCode,
    events::BeneficiaryInitialized,
    states::{BeneficiaryAllocation, Config, TreasuryShare},
};

#[derive(Accounts)]
pub struct InitializeBeneficiary<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub beneficiary: SystemAccount<'info>,

    #[account(
        mut,
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

    #[account(
        init,
        payer = authority,
        space = TreasuryShare::SIZE,
        seeds = [BENEFICIARY_TREASURY_SEED, beneficiary.key().as_ref()],
        bump
    )]
    pub treasury_share: Account<'info, TreasuryShare>,

    pub system_program: Program<'info, System>,
}

pub fn process(
    ctx: Context<InitializeBeneficiary>,
    total_tokens: u64,
    tge_unlock_bps: u16,
    instant_unlock: bool,
    withdraw_interval: i64,
    sol_share_bps: u16,
) -> Result<()> {
    require!(total_tokens > 0, ErrorCode::InvalidAmount);
    require!(
        tge_unlock_bps > 0 && tge_unlock_bps <= 5_000,
        ErrorCode::InvalidAmount
    );
    require!(
        sol_share_bps > 0 && sol_share_bps <= 5_000,
        ErrorCode::InvalidAmount
    );
    // withdraw_interval is only meaningful for non-liquidity beneficiaries
    if !instant_unlock {
        require!(withdraw_interval > 0, ErrorCode::InvalidAmount);
    }

    // calculate first month unlock based based on monthly unlock bps
    let monthly_unlocked: u64 = (MONTHLY_UNLOCK_BPS as u128)
        .checked_mul(total_tokens as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10_000)
        .ok_or(ErrorCode::MathOverflow)?
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)?;

    // accumulate sol_share_bps and enforce 100 % cap
    let new_total_sol_shares = ctx
        .accounts
        .config
        .total_sol_shares_bps
        .checked_add(sol_share_bps)
        .ok_or(ErrorCode::MathOverflow)?;
    require!(new_total_sol_shares <= 10_000, ErrorCode::InvalidAmount);
    ctx.accounts.config.total_sol_shares_bps = new_total_sol_shares;

    msg!("monthly_unlocked: {}", monthly_unlocked);

    // set beneficiary allocation account
    ctx.accounts
        .beneficiary_allocation
        .set_inner(BeneficiaryAllocation {
            total_tokens,
            withdrawn_tokens: 0,
            monthly_unlocked,
            tge_unlock_bps,
            instant_unlock,
            bump: ctx.accounts.beneficiary_allocation.bump,
        });

    // set treasury share account
    ctx.accounts.treasury_share.set_inner(TreasuryShare {
        sol_withdrawn: 0,
        last_sol_claim: 0,
        presale_start: ctx.accounts.config.presale_start_timestamp,
        withdraw_interval,
        sol_share_bps,
        instant_unlock,
        bump: ctx.accounts.treasury_share.bump,
    });

    emit!(BeneficiaryInitialized {
        beneficiary: ctx.accounts.beneficiary.key(),
        total_tokens,
        tge_unlock_bps,
        sol_share_bps,
    });

    Ok(())
}
