use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct DividendClaimedDetails {
    pub checkpoint_id: u64,
    pub claimer: Pubkey,
    pub token_mint: Pubkey,
    pub total_claimed: u64,
    pub last_claim_epoch: i64
}   

impl DividendClaimedDetails {
    pub const LEN: usize = 
        DISCRIMINATOR_LENGTH + // Discriminator
        U128_LENGTH / 2 + // checkpoint_id 
        PUBLIC_KEY_LENGTH + // claimer
        PUBLIC_KEY_LENGTH +  // token_mint
        U128_LENGTH;  // total_claimed - last_claim_epoch
}