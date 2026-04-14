use anchor_lang::constant;

pub const DISCRIMINATOR: usize = 8;

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const TREASURY_SEED: &[u8] = b"treasury";
