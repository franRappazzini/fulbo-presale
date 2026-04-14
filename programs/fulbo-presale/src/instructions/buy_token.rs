use anchor_lang::prelude::*;
use chainlink_solana::v2::read_feed_v2;

use crate::{
    constants::{CONFIG_SEED, TREASURY_SEED},
    error::ErrorCode,
    states::{Config, Treasury},
};

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    /// CHECK: We're reading data from this chainlink feed account
    #[account(address = config.chainlink_feed)]
    pub chainlink_feed: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<BuyToken>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let feed = &ctx.accounts.chainlink_feed;

    // Read the feed data directly from the account
    let result = read_feed_v2(feed.try_borrow_data()?, feed.owner.to_bytes())
        .map_err(|_| ErrorCode::ChainlinkReadError)?;

    // Get the latest round data
    let round = result
        .latest_round_data()
        .ok_or(ErrorCode::ChainlinkRoundDataMissing)?;

    let sol_price = round.answer;
    let oracle_decimals = result.decimals();

    msg!("Price: {}", sol_price);
    msg!("Decimals: {}", oracle_decimals);

    let usd_unit_amount =
        ctx.accounts.config.stages[ctx.accounts.config.current_stage as usize].price_usd;

    require!(usd_unit_amount > 0, ErrorCode::InvalidAmount);
    require!(sol_price > 0, ErrorCode::InvalidPrice);

    let total_usd_amount = (amount as u128)
        .checked_mul(usd_unit_amount as u128)
        .and_then(|v| v.checked_div(1_000_000)) // 10^6 decimals
        .ok_or(ErrorCode::MathOverflow)?;

    let exponent_adjustment = (oracle_decimals)
        .checked_add(9) // SOL decimals
        .and_then(|v| v.checked_sub(6)) // FULBO decimals
        .ok_or(ErrorCode::MathOverflow)?;

    require!(exponent_adjustment <= 12, ErrorCode::InvalidAmount); // sanity check
    msg!("Exponent adjustment: {}", exponent_adjustment);

    let scale_factor: u128 = 10_i128
        .checked_pow(exponent_adjustment as u32)
        .ok_or(ErrorCode::MathOverflow)?
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)?;

    let numerator = (total_usd_amount)
        .checked_mul(scale_factor)
        .ok_or(ErrorCode::MathOverflow)?;

    let lamports = numerator
        .checked_div(sol_price as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    require!(lamports > 0, ErrorCode::InvalidAmount);

    let lamports_u64: u64 = lamports.try_into().map_err(|_| ErrorCode::MathOverflow)?;

    msg!(
        "FULBO amount: {} | total USD: {} | total lamports: {} | SOL price: {} (10^-{}).",
        amount,
        total_usd_amount,
        lamports_u64,
        sol_price,
        oracle_decimals
    );

    Ok(())
}
