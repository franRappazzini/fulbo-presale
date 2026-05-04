use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::{CONFIG_SEED, TREASURY_SEED},
    error::ErrorCode,
    states::{Config, Stage, Treasury},
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Config::SIZE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = authority,
        space = Treasury::SIZE,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub treasury: Account<'info, Treasury>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = config,
        token::token_program = token_program,
        seeds = [TREASURY_SEED, mint.key().as_ref()],
        bump
    )]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: just saving the chainlink feed address
    pub chainlink_feed: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn process(
    ctx: Context<Initialize>,
    total_tokens_for_sale: u64,
    stages: [Stage; 11],
) -> Result<()> {
    // Validate that stage token allocations sum exactly to the total presale supply.
    let stage_tokens_total: u64 = stages
        .iter()
        .try_fold(0u64, |acc, s| acc.checked_add(s.max_tokens))
        .ok_or(ErrorCode::MathOverflow)?;
    require!(
        stage_tokens_total == total_tokens_for_sale,
        ErrorCode::InvalidAmount
    );

    // init accounts
    ctx.accounts.config.set_inner(Config {
        authority: ctx.accounts.authority.key(),
        mint: ctx.accounts.mint.key(),
        chainlink_feed: ctx.accounts.chainlink_feed.key(),
        stages,
        tge_timestamp: 0,
        presale_start_timestamp: Clock::get()?.unix_timestamp,
        total_sol_raised: 0,
        total_tokens_for_sale,
        total_tokens_sold: 0,
        total_tokens_claimed: 0,
        current_stage: 0,
        sale_finalized: false,
        paused: false,
        treasury_ata_bump: ctx.bumps.treasury_ata,
        bump: ctx.bumps.config,
        total_sol_shares_bps: 0,
        unsold_finalized: false,
    });

    let treasury = &mut ctx.accounts.treasury;
    treasury.bump = ctx.bumps.treasury;

    Ok(())
}
