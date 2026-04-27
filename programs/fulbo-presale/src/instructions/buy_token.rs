use anchor_lang::{prelude::*, system_program};
use chainlink_solana::v2::read_feed_v2;

use crate::{
    constants::{CONFIG_SEED, POSITION_SEED, TREASURY_SEED},
    error::ErrorCode,
    states::{Config, Position, Treasury},
};

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        constraint = !config.sale_finalized || config.tge_announced_timestamp >= Clock::get()?.unix_timestamp @ ErrorCode::SaleAlreadyFinalized,
        constraint = !config.paused @ ErrorCode::SalePaused,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        init_if_needed,
        payer = buyer,
        space = Position::SIZE,
        seeds = [POSITION_SEED, buyer.key().as_ref()],
        bump,
    )]
    pub position: Account<'info, Position>,

    /// CHECK: We're reading data from this chainlink feed account
    #[account(address = config.chainlink_feed)]
    pub chainlink_feed: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<BuyToken>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    // initialize position account if not already initialized
    let position = &mut ctx.accounts.position;
    if !position.is_initialized {
        position.is_initialized = true;
        position.bump = ctx.bumps.position;
    }

    // TODO: replace with actual price fetching from chainlink oracle
    let (sol_price, oracle_decimals) = get_sol_price(&ctx.accounts.chainlink_feed)?;
    // let sol_price = 9000000000;
    // let oracle_decimals = 8;

    msg!("Price: {}", sol_price);
    msg!("Decimals: {}", oracle_decimals);

    let usd_unit_amount =
        ctx.accounts.config.stages[ctx.accounts.config.current_stage as usize].price_usd;

    require!(usd_unit_amount > 0, ErrorCode::InvalidAmount);
    require!(sol_price > 0, ErrorCode::InvalidPrice);

    // checked inside
    let lamports = calculate_lamports(amount, usd_unit_amount, sol_price, oracle_decimals)?;

    // transfer lamports from buyer to treasury
    let cpi_accounts = system_program::Transfer {
        from: ctx.accounts.buyer.to_account_info(),
        to: ctx.accounts.treasury.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);

    system_program::transfer(cpi_ctx, lamports)?;

    // update config account
    let config = &mut ctx.accounts.config;
    let purchase_result = config.add_purchase(amount, lamports)?;

    // update treasury account
    let treasury = &mut ctx.accounts.treasury;
    treasury.total_sol = treasury
        .total_sol
        .checked_add(lamports)
        .ok_or(ErrorCode::MathOverflow)?;

    // update position account
    let position = &mut ctx.accounts.position;
    position.purchase(&purchase_result)
}

fn get_sol_price(feed: &UncheckedAccount) -> Result<(i128, u8)> {
    // Read the feed data directly from the account
    let result = read_feed_v2(feed.try_borrow_data()?, feed.owner.to_bytes())
        .map_err(|_| ErrorCode::ChainlinkReadError)?;

    // Get the latest round data
    let round = result
        .latest_round_data()
        .ok_or(ErrorCode::ChainlinkRoundDataMissing)?;

    Ok((round.answer, result.decimals()))
}

fn calculate_lamports(
    amount: u64,
    usd_unit_amount: u64,
    sol_price: i128,
    oracle_decimals: u8,
) -> Result<u64> {
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

    Ok(lamports_u64)
}
