use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct DividendDistributor {
    pub id: u64,
    pub root: [u8; 32],
    pub owner: Pubkey,
    pub token_mint: Pubkey,
    pub total_distribute_amount: u64,
    pub total_claimed: u64,
    pub freezed: bool
}   

impl DividendDistributor {
    pub const LEN: usize = 
        DISCRIMINATOR_LENGTH + // Discriminator
        U128_LENGTH / 2 + // ID 
        U128_LENGTH * 2 + // Root
        PUBLIC_KEY_LENGTH +  // Owner
        PUBLIC_KEY_LENGTH + // Token mint
        U128_LENGTH +   // Total_distribute_amount - Total_claimed
        BOOL_LENGTH ; // freezed

    pub fn init(
        &mut self,
        id: u64,
        root: [u8; 32],
        owner: Pubkey,
        mint: Pubkey,
        total_distribute_amount: u64,
    ) -> Result<()> {
        self.id = id;
        self.root = root;
        self.owner = owner;
        self.token_mint = mint;
        self.total_claimed = 0;
        self.total_distribute_amount = total_distribute_amount;
        self.freezed = false;

        Ok(())
    }

}