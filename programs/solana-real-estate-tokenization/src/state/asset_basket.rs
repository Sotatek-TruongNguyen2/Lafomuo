use anchor_lang::prelude::*;
// use crate::errors::LandLordErrors;
use crate::constants::*;

#[account]
pub struct AssetBasket {
    pub asset_id: Pubkey,
    pub asset_metadata: Pubkey,
    pub owner: Pubkey,
    pub governor: Pubkey,
    pub iat: i64,
    pub is_freezed: bool,
    pub bump: u8
}

impl AssetBasket {
    pub const LEN: usize = DISCRIMINATOR_LENGTH + PUBLIC_KEY_LENGTH * 4 + U128_LENGTH / 2 + BOOL_LENGTH + U128_LENGTH / U128_LENGTH;


    pub fn init(
        &mut self, 
        bump: u8,
        asset_metadata: Pubkey,
        governor: Pubkey, 
        owner: Pubkey,
        mint: Pubkey,
    ) -> Result<&mut AssetBasket> {
        let clock: Clock = Clock::get().unwrap();

        self.bump = bump;
        self.asset_metadata = asset_metadata;
        self.governor = governor;
        self.iat = clock.unix_timestamp;
        self.asset_id = mint;
        self.owner = owner;
        self.is_freezed = false;

        Ok(self)
    }
}
