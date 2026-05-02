use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::BurnChecked,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{BENEFICIARY_ALLOCATION_SEED, CONFIG_SEED, TREASURY_SEED},
    error::ErrorCode,
    events::UnsoldTokensFinalized,
    states::{BeneficiaryAllocation, Config},
};

#[derive(Accounts)]
pub struct FinalizeUnsold<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub rewards_beneficiary: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority,
        constraint = config.sale_finalized
            || (config.tge_timestamp != 0 && config.tge_timestamp <= Clock::get()?.unix_timestamp)
            @ ErrorCode::TgeNotStarted,
        constraint = !config.unsold_finalized @ ErrorCode::SaleAlreadyFinalized,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [BENEFICIARY_ALLOCATION_SEED, rewards_beneficiary.key().as_ref()],
        bump = beneficiary_allocation.bump,
    )]
    pub beneficiary_allocation: Account<'info, BeneficiaryAllocation>,

    #[account(
        mut,
        address = config.mint,
    )]
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

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn process(ctx: Context<FinalizeUnsold>) -> Result<()> {
    let config = &mut ctx.accounts.config;

    let unsold = config
        .total_tokens_for_sale
        .checked_sub(config.total_tokens_sold)
        .ok_or(ErrorCode::MathOverflow)?;

    // if nothing is unsold, mark as finalized and exit early.
    if unsold == 0 {
        config.unsold_finalized = true;
        config.unsold_rewards_total = 0;
        emit!(UnsoldTokensFinalized {
            burned: 0,
            rewarded: 0,
        });
        return Ok(());
    }

    // 60% burned and 40% distributed to in-game rewards vault.
    let to_burn: u64 = (unsold as u128)
        .checked_mul(6_000)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10_000)
        .ok_or(ErrorCode::MathOverflow)?
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)?;

    let to_reward: u64 = unsold.checked_sub(to_burn).ok_or(ErrorCode::MathOverflow)?;

    let config_bump = config.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[config_bump]]];

    let cpi_accounts = BurnChecked {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.treasury_ata.to_account_info(),
        authority: ctx.accounts.config.to_account_info(),
    };

    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_accounts, signer_seeds);

    token_interface::burn_checked(cpi_ctx, to_burn, ctx.accounts.mint.decimals)?;

    // update config and beneficiary allocation accounts
    ctx.accounts.config.unsold_rewards_total = to_reward;
    ctx.accounts.config.unsold_finalized = true;

    ctx.accounts.beneficiary_allocation.total_tokens = ctx
        .accounts
        .beneficiary_allocation
        .total_tokens
        .checked_add(to_reward)
        .ok_or(ErrorCode::MathOverflow)?;

    emit!(UnsoldTokensFinalized {
        burned: to_burn,
        rewarded: to_reward,
    });

    Ok(())
}
