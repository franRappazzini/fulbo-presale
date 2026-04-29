use anchor_lang::prelude::{program_option::COption, *};
use anchor_spl::token_interface::Mint;

use crate::{
    constants::{CONFIG_SEED, TREASURY_SEED},
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

    #[account(
        constraint = mint.mint_authority == COption::Some(config.key()),
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: just saving the chainlink feed address
    pub chainlink_feed: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process(
    ctx: Context<Initialize>,
    total_tokens_for_sale: u64,
    stages: [Stage; 11],
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.mint = ctx.accounts.mint.key();
    config.chainlink_feed = ctx.accounts.chainlink_feed.key();
    config.presale_start_timestamp = Clock::get()?.unix_timestamp;
    config.total_tokens_for_sale = total_tokens_for_sale;
    config.stages = stages;
    config.bump = ctx.bumps.config;

    let treasury = &mut ctx.accounts.treasury;
    treasury.bump = ctx.bumps.treasury;

    Ok(())
}
