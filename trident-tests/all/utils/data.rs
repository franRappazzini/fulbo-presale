use crate::types::{fulbo_presale::InitializeBeneficiaryInstructionData, Stage};

pub const STAGES: &[Stage; 11] = &[
    Stage {
        price_usd: 500,
        max_tokens: 20_000_000_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 5000,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 700,
        max_tokens: 28_571_429_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 5000,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 900,
        max_tokens: 33_333_333_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 5000,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 1100,
        max_tokens: 36_363_636_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 5000,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 1400,
        max_tokens: 35_714_286_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 3500,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 1800,
        max_tokens: 33_333_333_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 3500,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 2300,
        max_tokens: 30_434_783_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 3500,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 2900,
        max_tokens: 27_586_207_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 3500,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 3700,
        max_tokens: 24_324_324_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 2000,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 5000,
        max_tokens: 20_000_000_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 2000,
        max_wallet_pct_bps: 500,
    },
    Stage {
        price_usd: 6500,
        max_tokens: 10_338_669_000_000,
        tokens_sold: 0,
        raised_sol: 0,
        locked_pct_bps: 2000,
        max_wallet_pct_bps: 500,
    },
];

pub fn total_tokens_for_sale() -> u64 {
    STAGES.iter().map(|stage| stage.max_tokens).sum()
}

pub const BENEFICIARIES: &[InitializeBeneficiaryInstructionData; 4] = &[
    InitializeBeneficiaryInstructionData {
        total_tokens: 150_000_000_000_000, // Team: 150M
        tge_unlock_bps: 500,               // 5% unlocked at TGE
        instant_unlock: false,
        withdraw_interval: 1, // 1s for tests
        sol_share_bps: 2000,  // 20%
    },
    InitializeBeneficiaryInstructionData {
        total_tokens: 200_000_000_000_000, // Marketing: 200M
        tge_unlock_bps: 2000,
        instant_unlock: false,
        withdraw_interval: 1,
        sol_share_bps: 2500, // 25%
    },
    InitializeBeneficiaryInstructionData {
        total_tokens: 50_000_000_000_000, // Development: 50M
        tge_unlock_bps: 500,
        instant_unlock: false,
        withdraw_interval: 1,
        sol_share_bps: 3500, // 35%
    },
    InitializeBeneficiaryInstructionData {
        total_tokens: 100_000_000_000_000, // Liquidity: 100M
        tge_unlock_bps: 0,
        instant_unlock: true,
        withdraw_interval: 0, // unused (instant_unlock=true)
        sol_share_bps: 2000,  // 20% — total bps: 2000+2500+3500+2000 = 10_000 (100%) ✓
    },
];
