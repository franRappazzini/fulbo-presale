use trident_fuzz::fuzzing::{pubkey, Pubkey};

pub const CONFIG_SEED: &[u8] = b"config";

pub const TREASURY_SEED: &[u8] = b"treasury";

pub const POSITION_SEED: &[u8] = b"position";

pub const BENEFICIARY_ALLOCATION_SEED: &[u8] = b"beneficiary_allocation";

pub const BENEFICIARY_TREASURY_SEED: &[u8] = b"beneficiary_treasury";

pub const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
