use crate::constants::*;
use anchor_lang::prelude::*;

/// A group of [Escrow]s.
#[account]
#[derive(Copy, Debug, Default)]
pub struct FractionalizedTokenLocker {
    pub basket_id: u64,
    /// Base account used to generate signer seeds.
    pub dividend_distributor: Pubkey,
    /// Governor associated with the [Locker].
    pub governor: Pubkey,
    /// Mint of the token that must be locked in the [Locker].
    pub token_mint: Pubkey,
    /// Total number of tokens locked in [Escrow]s.
    pub locked_supply: u64,
    // locking duration of an escrow    
    pub lock_end_time: i64,
    /// Bump seed.
    pub bump: u8,
}

impl FractionalizedTokenLocker {
    pub const LEN: usize = DISCRIMINATOR_LENGTH +
        U128_LENGTH / 2 + // asset_id
        PUBLIC_KEY_LENGTH + // base
        PUBLIC_KEY_LENGTH + // token_mint 
        PUBLIC_KEY_LENGTH + // governor
        U128_LENGTH + // locked_supply - lock_end_time
        BOOL_LENGTH; // bump
}
