use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{CONFIG_SEED, MONTHLY_UNLOCK_BPS, POSITION_SEED, SECONDS_PER_MONTH, TREASURY_SEED},
    error::ErrorCode,
    events::TokensClaimed,
    states::{Config, Position},
};

#[derive(Accounts)]
pub struct ClaimToken<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        constraint = config.sale_finalized
            || (config.tge_timestamp != 0 && config.tge_timestamp <= Clock::get()?.unix_timestamp)
            @ ErrorCode::TgeNotStarted,
        constraint = !config.paused @ ErrorCode::SalePaused,
    )]
    pub config: Account<'info, Config>,

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
        mut,
        seeds = [TREASURY_SEED, mint.key().as_ref()],
        bump = config.treasury_ata_bump,
        token::mint = mint,
        token::authority = config,
        token::token_program = token_program
    )]
    pub treasury_ata: Box<InterfaceAccount<'info, TokenAccount>>,

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

    let now = Clock::get()?.unix_timestamp;

    let seconds_since_tge = (now.saturating_sub(config.tge_timestamp)).max(0) as u64;
    let months_elapsed = seconds_since_tge / SECONDS_PER_MONTH as u64;

    let mut total_claimable: u64 = 0;

    for alloc in position.stage_allocations.iter_mut() {
        if alloc.tokens == 0 || alloc.claimed == alloc.tokens {
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

        msg!(
            "Stage {}: tokens: {}, locked_bps: {}, unlocked_at_tge_bps: {}, vested_bps: {}, total_unlocked_bps: {}, total_unlocked: {}",
            config.current_stage,
            alloc.tokens,
            locked_bps,
            unlocked_at_tge_bps,
            vested_bps,
            total_unlocked_bps,
            total_unlocked
        );

        let claimable = total_unlocked.saturating_sub(alloc.claimed);

        if claimable > 0 {
            alloc.claimed = alloc
                .claimed
                .checked_add(claimable)
                .ok_or(ErrorCode::MathOverflow)?;
            total_claimable = total_claimable
                .checked_add(claimable)
                .ok_or(ErrorCode::MathOverflow)?;
        }
    }

    require!(total_claimable > 0, ErrorCode::NothingToClaim);

    // update position account
    position.tokens_claimed = position
        .tokens_claimed
        .checked_add(total_claimable)
        .ok_or(ErrorCode::MathOverflow)?;

    // update config account
    config.total_tokens_claimed = config
        .total_tokens_claimed
        .checked_add(total_claimable)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!("claim: {} tokens", total_claimable);

    // transfer tokens to claimer
    let config_bump = config.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[config_bump]]];

    let cpi_accounts = token_interface::TransferChecked {
        authority: ctx.accounts.config.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.treasury_ata.to_account_info(),
        to: ctx.accounts.claimer_ata.to_account_info(),
    };

    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi_accounts, signer_seeds);

    token_interface::transfer_checked(cpi_ctx, total_claimable, ctx.accounts.mint.decimals)?;

    emit!(TokensClaimed {
        claimer: ctx.accounts.claimer.key(),
        total_claimable,
    });

    Ok(())
}
