use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{CONFIG_SEED, MONTHLY_UNLOCK_BPS, POSITION_SEED, SECONDS_PER_MONTH, TREASURY_SEED},
    error::ErrorCode,
    states::{Config, Position, Treasury},
};

#[derive(Accounts)]
pub struct ClaimToken<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        mut,
        seeds = [POSITION_SEED, claimer.key().as_ref()],
        bump = position.bump
    )]
    pub position: Account<'info, Position>,

    #[account(
        mut,
        address = config.mint
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = claimer,
        associated_token::mint = mint,
        associated_token::authority = claimer,
        associated_token::token_program = token_program
    )]
    pub claimer_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<ClaimToken>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let position = &mut ctx.accounts.position;

    // tge starts only after the presale is finalized (last token sold) or manually by admin
    require!(config.sale_finalized, ErrorCode::TgeNotStarted);

    let now = Clock::get()?.unix_timestamp;

    let seconds_since_tge = (now.saturating_sub(config.tge_timestamp)).max(0) as u64;
    let months_elapsed = seconds_since_tge / SECONDS_PER_MONTH;

    let mut total_claimable: u64 = 0;

    for alloc in position.stage_allocations.iter_mut() {
        if alloc.tokens == 0 || alloc.claimed {
            continue;
        }

        let locked_bps = alloc.locked_pct_bps as u64;
        let unlocked_at_tge_bps = 10_000u64.saturating_sub(locked_bps);

        let vested_bps = (months_elapsed.saturating_mul(MONTHLY_UNLOCK_BPS as u64)).min(locked_bps);

        let total_unlocked_bps = unlocked_at_tge_bps.saturating_add(vested_bps);

        let total_unlocked: u64 = (alloc.tokens as u128)
            .checked_mul(total_unlocked_bps as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(10_000)
            .ok_or(ErrorCode::MathOverflow)?
            .try_into()
            .map_err(|_| ErrorCode::MathOverflow)?;

        if total_unlocked > 0 {
            alloc.claimed = true;
            total_claimable = total_claimable
                .checked_add(total_unlocked)
                .ok_or(ErrorCode::MathOverflow)?;
        }
    }

    require!(total_claimable > 0, ErrorCode::NothingToClaim);

    // update position account
    position.tokens_claimed = position
        .tokens_claimed
        .checked_add(total_claimable)
        .ok_or(ErrorCode::MathOverflow)?;
    position.last_claim_ts = now;

    // update config account
    config.total_tokens_claimed = config
        .total_tokens_claimed
        .checked_add(total_claimable)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!("claim: {} tokens", total_claimable);

    // mint tokens to claimer
    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[config.bump]]];

    let cpi_accounts = token_interface::MintToChecked {
        authority: ctx.accounts.config.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.claimer_ata.to_account_info(),
    };

    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_accounts, signer_seeds);

    token_interface::mint_to_checked(cpi_ctx, total_claimable, ctx.accounts.mint.decimals)
}
