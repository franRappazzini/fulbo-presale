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
        constraint = !config.sale_finalized || config.tge_timestamp >= Clock::get()?.unix_timestamp @ ErrorCode::SaleAlreadyFinalized,
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
    if !ctx.accounts.position.is_initialized {
        ctx.accounts.position.is_initialized = true;
        ctx.accounts.position.bump = ctx.bumps.position;
    }

    let (sol_price, oracle_decimals) = get_sol_price(&ctx.accounts.chainlink_feed)?;

    msg!("Price: {}", sol_price);
    msg!("Decimals: {}", oracle_decimals);

    require!(sol_price > 0, ErrorCode::InvalidPrice);

    let current_stage_idx = ctx.accounts.config.current_stage as usize;
    let current_price_usd = ctx.accounts.config.stages[current_stage_idx].price_usd;
    let current_max_tokens = ctx.accounts.config.stages[current_stage_idx].max_tokens;
    let current_tokens_sold = ctx.accounts.config.stages[current_stage_idx].tokens_sold;
    let current_max_wallet_bps = ctx.accounts.config.stages[current_stage_idx].max_wallet_pct_bps;

    require!(current_price_usd > 0, ErrorCode::InvalidAmount);

    // calculate how many tokens are still available in the current stage, and how many overflow
    let available_in_current = current_max_tokens
        .checked_sub(current_tokens_sold)
        .ok_or(ErrorCode::MathOverflow)?;
    let tokens_in_current = amount.min(available_in_current);
    let tokens_in_overflow = amount.saturating_sub(available_in_current);

    let current_max_wallet = (current_max_tokens as u128)
        .checked_mul(current_max_wallet_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10_000)
        .ok_or(ErrorCode::MathOverflow)?;

    require!(
        (ctx.accounts.position.stage_allocations[current_stage_idx].tokens as u128)
            .checked_add(tokens_in_current as u128)
            .ok_or(ErrorCode::MathOverflow)?
            <= current_max_wallet,
        ErrorCode::ExceedsMaxPerWallet
    );

    // calculate lamports with correct stage pricing
    let (lamports, overflow_lamports) = if tokens_in_overflow > 0 {
        require!(current_stage_idx < 10, ErrorCode::InvalidAmount);

        let next_price_usd = ctx.accounts.config.stages[current_stage_idx + 1].price_usd;
        let next_max_tokens = ctx.accounts.config.stages[current_stage_idx + 1].max_tokens;
        let next_max_wallet_bps =
            ctx.accounts.config.stages[current_stage_idx + 1].max_wallet_pct_bps;

        require!(next_price_usd > 0, ErrorCode::InvalidAmount);

        let next_max_wallet = (next_max_tokens as u128)
            .checked_mul(next_max_wallet_bps as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(10_000)
            .ok_or(ErrorCode::MathOverflow)?;

        require!(
            (ctx.accounts.position.stage_allocations[current_stage_idx + 1].tokens as u128)
                .checked_add(tokens_in_overflow as u128)
                .ok_or(ErrorCode::MathOverflow)?
                <= next_max_wallet,
            ErrorCode::ExceedsMaxPerWallet
        );

        let lamps_current = if tokens_in_current > 0 {
            calculate_lamports(
                tokens_in_current,
                current_price_usd,
                sol_price,
                oracle_decimals,
            )?
        } else {
            0u64
        };
        let lamps_overflow = calculate_lamports(
            tokens_in_overflow,
            next_price_usd,
            sol_price,
            oracle_decimals,
        )?;
        let total = lamps_current
            .checked_add(lamps_overflow)
            .ok_or(ErrorCode::MathOverflow)?;
        (total, lamps_overflow)
    } else {
        let lamps = calculate_lamports(amount, current_price_usd, sol_price, oracle_decimals)?;
        (lamps, 0u64)
    };

    let cpi_accounts = system_program::Transfer {
        from: ctx.accounts.buyer.to_account_info(),
        to: ctx.accounts.treasury.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);

    system_program::transfer(cpi_ctx, lamports)?;

    // update config, treasury and position accounts
    let config = &mut ctx.accounts.config;
    let purchase_result = config.add_purchase(amount, lamports, overflow_lamports)?;

    let treasury = &mut ctx.accounts.treasury;
    treasury.total_sol = treasury
        .total_sol
        .checked_add(lamports)
        .ok_or(ErrorCode::MathOverflow)?;

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

    // oracle_decimals + SOL_decimals(9) - FULBO_decimals(6).
    let exponent_raw = (oracle_decimals as i32) + 9 - 6;
    require!(
        exponent_raw >= 0 && exponent_raw <= 12,
        ErrorCode::InvalidPrice
    );
    let exponent_adjustment = exponent_raw as u32;

    msg!("Exponent adjustment: {}", exponent_adjustment);

    let scale_factor: u128 = 10_i128
        .checked_pow(exponent_adjustment)
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
