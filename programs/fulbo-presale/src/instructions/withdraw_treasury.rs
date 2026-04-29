use anchor_lang::{prelude::*, system_program};

use crate::{
    constants::{BENEFICIARY_TREASURY_SEED, CONFIG_SEED, TREASURY_SEED},
    error::ErrorCode,
    states::{Config, Treasury, TreasuryShare},
};

#[derive(Accounts)]
pub struct WithdrawTreasury<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(
        mut,
        seeds = [BENEFICIARY_TREASURY_SEED, beneficiary.key().as_ref()],
        bump = beneficiary_treasury.bump,
    )]
    pub beneficiary_treasury: Account<'info, TreasuryShare>,

    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury.bump,
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<WithdrawTreasury>) -> Result<()> {
    let beneficiary_treasury = &mut ctx.accounts.beneficiary_treasury;
    let now = Clock::get()?.unix_timestamp;

    let claim_timestamp = if beneficiary_treasury.is_liquidity {
        // full withdrawal, only after TGE
        let tge_timestamp = ctx.accounts.config.tge_timestamp;
        require!(
            ctx.accounts.config.sale_finalized || (tge_timestamp != 0 && now >= tge_timestamp),
            ErrorCode::TgeNotStarted
        );

        require!(
            beneficiary_treasury.last_sol_claim == 0,
            ErrorCode::NothingToClaim
        );

        now
    } else {
        require!(
            now >= beneficiary_treasury.presale_start,
            ErrorCode::NothingToClaim
        );

        let elapsed = now
            .checked_sub(beneficiary_treasury.presale_start)
            .ok_or(ErrorCode::MathOverflow)?;

        let intervals_elapsed = elapsed
            .checked_div(beneficiary_treasury.withdraw_interval)
            .ok_or(ErrorCode::MathOverflow)?;

        // at least one full interval must have passed since presale_start
        require!(intervals_elapsed >= 1, ErrorCode::NothingToClaim);

        // start of the interval we are currently inside
        let current_interval_start = beneficiary_treasury
            .presale_start
            .checked_add(
                intervals_elapsed
                    .checked_mul(beneficiary_treasury.withdraw_interval)
                    .ok_or(ErrorCode::MathOverflow)?,
            )
            .ok_or(ErrorCode::MathOverflow)?;

        msg!("Current interval start: {}", current_interval_start);
        msg!("Last sol claim: {}", beneficiary_treasury.last_sol_claim);

        // prevent a second withdrawal within the same interval window
        require!(
            current_interval_start > beneficiary_treasury.last_sol_claim,
            ErrorCode::NothingToClaim
        );

        current_interval_start
    };

    let treasury = &mut ctx.accounts.treasury;

    let total_entitlement: u64 = (beneficiary_treasury.sol_share_bps as u128)
        .checked_mul(treasury.total_sol as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10_000)
        .ok_or(ErrorCode::MathOverflow)?
        .try_into()
        .map_err(|_| ErrorCode::MathOverflow)?;

    let amount_to_withdraw = total_entitlement
        .checked_sub(beneficiary_treasury.sol_withdrawn)
        .ok_or(ErrorCode::MathOverflow)?;

    require!(amount_to_withdraw > 0, ErrorCode::NothingToClaim);

    let available_in_treasury = treasury
        .total_sol
        .checked_sub(treasury.withdrawn_sol)
        .ok_or(ErrorCode::MathOverflow)?;
    require!(
        amount_to_withdraw <= available_in_treasury,
        ErrorCode::NothingToClaim
    );

    // update beneficiary and treasury accounts
    beneficiary_treasury.sol_withdrawn = beneficiary_treasury
        .sol_withdrawn
        .checked_add(amount_to_withdraw)
        .ok_or(ErrorCode::MathOverflow)?;
    beneficiary_treasury.last_sol_claim = claim_timestamp;

    treasury.withdrawn_sol = treasury
        .withdrawn_sol
        .checked_add(amount_to_withdraw)
        .ok_or(ErrorCode::MathOverflow)?;

    // transfer amount
    let signer_seeds: &[&[&[u8]]] = &[&[TREASURY_SEED, &[treasury.bump]]];

    let cpi_accounts = system_program::Transfer {
        from: treasury.to_account_info(),
        to: ctx.accounts.beneficiary.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        cpi_accounts,
        signer_seeds,
    );

    system_program::transfer(cpi_ctx, amount_to_withdraw)
}
