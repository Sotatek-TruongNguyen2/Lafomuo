use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct FractionalTokenEscrow {
    pub asset_id: Pubkey,
    /// Mint of the token that must be locked in the [Locker].
    pub token_mint: Pubkey,
    /// Governor
    pub governor: Pubkey,
    pub owner: Pubkey,
    pub suggested_price: u64,
    pub locked_amount: u64,
    pub bump: u8
}   

impl FractionalTokenEscrow {
    pub const LEN: usize = 
        DISCRIMINATOR_LENGTH +
        PUBLIC_KEY_LENGTH + // asset_id
        PUBLIC_KEY_LENGTH + // governor 
        PUBLIC_KEY_LENGTH +  // owner
        U128_LENGTH + // suggested_price + locked_amount
        BOOL_LENGTH; // bump
}