pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use states::*;

declare_id!("preYE2fYthZLnc49qfRywsp1NdEonYmv4Dyxih6nyQj");

#[program]
pub mod fulbo_presale {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, stages: [Stage; 11]) -> Result<()> {
        initialize::process(ctx, stages)
    }
}
