use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{BENEFICIARY_ALLOCATION_SEED, CONFIG_SEED, SECONDS_PER_MONTH, TREASURY_SEED},
    error::ErrorCode,
    states::{BeneficiaryAllocation, Config},
};

#[derive(Accounts)]
pub struct BeneficiaryClaim<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        constraint = config.sale_finalized
            || (config.tge_timestamp > 0 && config.tge_timestamp <= Clock::get()?.unix_timestamp)
            @ ErrorCode::TgeNotStarted,
        constraint = !config.paused @ ErrorCode::SalePaused,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds = [BENEFICIARY_ALLOCATION_SEED, beneficiary.key().as_ref()],
        bump = beneficiary_allocation.bump,
    )]
    pub beneficiary_allocation: Account<'info, BeneficiaryAllocation>,

    #[account(address = config.mint)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = config,
        token::token_program = token_program,
        seeds = [TREASURY_SEED, mint.key().as_ref()],
        bump = config.treasury_ata_bump,

    )]
    pub treasury_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = beneficiary,
        associated_token::mint = mint,
        associated_token::authority = beneficiary,
        associated_token::token_program = token_program,
    )]
    pub beneficiary_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<BeneficiaryClaim>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    let beneficiary_allocation = &mut ctx.accounts.beneficiary_allocation;

    let claimable_tokens: u64 = if beneficiary_allocation.is_liquidity {
        // full claim, once only.
        require!(
            !beneficiary_allocation.tge_claimed,
            ErrorCode::NothingToClaim
        );
        beneficiary_allocation.tge_claimed = true;
        beneficiary_allocation.total_tokens
    } else {
        let tge_timestamp = ctx.accounts.config.tge_timestamp;

        let seconds_since_tge: u64 = now
            .checked_sub(tge_timestamp)
            .ok_or(ErrorCode::MathOverflow)?
            .try_into()
            .map_err(|_| ErrorCode::MathOverflow)?;
        let months_since_tge = seconds_since_tge / SECONDS_PER_MONTH as u64;

        let tge_unlock_amount: u64 = (beneficiary_allocation.total_tokens as u128)
            .checked_mul(beneficiary_allocation.tge_unlock_bps as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(10_000)
            .ok_or(ErrorCode::MathOverflow)?
            .try_into()
            .map_err(|_| ErrorCode::MathOverflow)?;

        // vested monthly amount
        let vested_amount: u64 = (beneficiary_allocation.monthly_unlocked as u128)
            .checked_mul(months_since_tge as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .try_into()
            .map_err(|_| ErrorCode::MathOverflow)?;

        let cumulative_unlocked = tge_unlock_amount
            .checked_add(vested_amount)
            .ok_or(ErrorCode::MathOverflow)?
            .min(beneficiary_allocation.total_tokens); // just to prevent overclaiming in case of several months passing without claiming

        let claimable = cumulative_unlocked
            .checked_sub(beneficiary_allocation.withdrawn_tokens)
            .ok_or(ErrorCode::MathOverflow)?;

        require!(claimable > 0, ErrorCode::NothingToClaim);

        claimable
    };

    // update beneficiary account
    beneficiary_allocation.withdrawn_tokens = beneficiary_allocation
        .withdrawn_tokens
        .checked_add(claimable_tokens)
        .ok_or(ErrorCode::MathOverflow)?;
    beneficiary_allocation.last_vesting_claim = now;

    // transfer claimable tokens
    let signer_seeds: &[&[&[u8]]] = &[&[
        TREASURY_SEED,
        &ctx.accounts.mint.key().to_bytes(),
        &[ctx.accounts.config.treasury_ata_bump],
    ]];

    let cpi_accounts = token_interface::TransferChecked {
        authority: ctx.accounts.config.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.treasury_ata.to_account_info(),
        to: ctx.accounts.beneficiary_ata.to_account_info(),
    };

    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_accounts, signer_seeds);

    token_interface::transfer_checked(cpi_ctx, claimable_tokens, ctx.accounts.mint.decimals)
}
