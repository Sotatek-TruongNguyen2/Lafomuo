use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct FractionalTokenEscrow {
    /// The [Locker] that this [Escrow] is part of.
    pub locker: Pubkey,
    pub owner: Pubkey,
    /// The token account holding the escrow tokens.
    pub hodl: Pubkey,
    pub locked_amount: u64,
    pub suggested_price: u64,
    pub bump: u8
}   

impl FractionalTokenEscrow {
    pub const LEN: usize = 
        DISCRIMINATOR_LENGTH +
        PUBLIC_KEY_LENGTH + // asset_id
        PUBLIC_KEY_LENGTH + // owner 
        PUBLIC_KEY_LENGTH +  // hodl
        U128_LENGTH + // locked_amount + suggested_price
        BOOL_LENGTH; // bump
}