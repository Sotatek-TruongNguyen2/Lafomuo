use crate::constants::*;
use anchor_lang::prelude::*;

/// A group of [Escrow]s.
#[account]
#[derive(Copy, Debug, Default)]
pub struct FractionalizedTokenLocker {
    pub asset_id: Pubkey,
    /// Base account used to generate signer seeds.
    pub base: Pubkey,
    /// Governor associated with the [Locker].
    pub governor: Pubkey,
    /// Mint of the token that must be locked in the [Locker].
    pub token_mint: Pubkey,
    /// Total number of tokens locked in [Escrow]s.
    pub locked_supply: u64,
    /// Bump seed.
    pub bump: u8,
}

impl FractionalizedTokenLocker {
    pub const LEN: usize = DISCRIMINATOR_LENGTH +
        PUBLIC_KEY_LENGTH + // asset_id
        PUBLIC_KEY_LENGTH + // base
        PUBLIC_KEY_LENGTH + // token_mint 
        PUBLIC_KEY_LENGTH + // governor
        U128_LENGTH / 2 + // locked_supply
        BOOL_LENGTH; // bump
}
