use anchor_lang::prelude::*;
use mpl_token_metadata::state::{MAX_SYMBOL_LENGTH};
use crate::errors::LandLordErrors;
use crate::constants::*;

#[account]
pub struct PlatformGovernor {
    /// The symbol for the asset
    pub symbol: String,
    pub is_mutable: bool,
    pub total_assets_minted: u64,
    pub total_assets_burned: u64,
    pub minting_protocol_price: u64,
    pub minting_protocol_token: Option<Pubkey>,
    pub guardians: [Option<Pubkey>; 3],
    pub big_guardian: Pubkey,
    pub treasury: Pubkey
}

impl PlatformGovernor {
   pub const LEN: usize = 
    DISCRIMINATOR_LENGTH + 
    BOOL_LENGTH + // is_mutable 
    U128_LENGTH + // total_assets_minted + total_assets_burned
    U128_LENGTH / 2 + //  minting_protocol_price
    PUBLIC_KEY_LENGTH + //  minting_protocol_token
    VEC_LENGTH_PREFIX +  // guardians length
    5 * PUBLIC_KEY_LENGTH + // guardians (3) + (big_guardian - treasury)(2)
    VEC_LENGTH_PREFIX + // string length
    MAX_TOPIC_LENGTH; // symbol length

    pub fn init(
        &mut self, 
        minting_protocol_price: u64, 
        minting_protocol_token: Option<Pubkey>,
        symbol: String,
        treasury: Pubkey,
        big_guardian: Pubkey,
        guardians: [Option<Pubkey>; 3]
    ) -> Result<&mut PlatformGovernor> {
        // Allow authorities to change protocol price and token 
        self.is_mutable = true;
        self.minting_protocol_token = minting_protocol_token;
        self.minting_protocol_price = minting_protocol_price;
        self.treasury = treasury;
        
        let mut total_authorities: u8 = 0;
        
        for authority in guardians.iter() {
           if authority.is_some() {
            total_authorities = total_authorities.checked_add(1).unwrap(); 
           }
        }
        
        if total_authorities == 0 {
            return Err(LandLordErrors::PlatformHasNoAuthorities.into());
        }

        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_SYMBOL_LENGTH - symbol.len() {
            array_of_zeroes.push(0u8);
        }
        let new_symbol =
            symbol.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        
        self.symbol = new_symbol;
        self.guardians = guardians;
        self.big_guardian = big_guardian;
        self.total_assets_minted = 0;
        self.total_assets_burned = 0;
        
        Ok(self)
    }

    pub fn increase_total_minted(
        &mut self,
        more: u64
    ) -> Result<()> {
        if more == 0 {
            return Err(LandLordErrors::MintingAmountCantBeZero.into());
        }

        self.total_assets_minted = self.total_assets_minted.checked_add(more).unwrap();
        Ok(())
    }
}