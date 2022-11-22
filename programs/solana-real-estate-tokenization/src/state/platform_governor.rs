use anchor_lang::prelude::*;
use mpl_token_metadata::state::{MAX_SYMBOL_LENGTH};
use crate::errors::LandLordErrors;
use crate::constants::*;

#[account]
pub struct PlatformGovernor {
    /// The symbol for the asset
    pub symbol: String,
    pub is_mutable: bool,
    pub total_dividend_checkpoint: u64,
    pub total_assets_minted: u64,
    pub total_assets_burned: u64,
    pub minting_protocol_price: u64,
    pub minting_protocol_token: Option<Pubkey>,
    pub big_guardian: Pubkey,
    pub treasury: Pubkey,
    pub setting: Pubkey
}

impl PlatformGovernor {
   pub const LEN: usize = 
    DISCRIMINATOR_LENGTH + 
    BOOL_LENGTH + // is_mutable 
    U128_LENGTH + // total_dividend_checkpoint - total_assets_minted 
    U128_LENGTH + // total_assets_burned - minting_protocol_price
    PUBLIC_KEY_LENGTH + //  minting_protocol_token
    VEC_LENGTH_PREFIX +  // guardians length
    3 * PUBLIC_KEY_LENGTH + // (big_guardian - treasury - setting)(3)
    VEC_LENGTH_PREFIX + // string length
    MAX_TOPIC_LENGTH; // symbol length

    pub fn init(
        &mut self, 
        minting_protocol_price: u64, 
        minting_protocol_token: Option<Pubkey>,
        symbol: String,
        treasury: Pubkey,
        big_guardian: Pubkey,
        setting: Pubkey
    ) -> Result<&mut PlatformGovernor> {
        // Allow authorities to change protocol price and token 
        self.is_mutable = true;
        self.minting_protocol_token = minting_protocol_token;
        self.minting_protocol_price = minting_protocol_price;
        self.treasury = treasury;
        
        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_SYMBOL_LENGTH - symbol.len() {
            array_of_zeroes.push(0u8);
        }
        let new_symbol =
            symbol.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        
        self.symbol = new_symbol;
        self.big_guardian = big_guardian;
        self.setting = setting;
        self.total_assets_minted = 0;
        self.total_assets_burned = 0;
        self.total_dividend_checkpoint = 0;
        
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

    pub fn update_dividend_checkpoints(
        &mut self,
        more: u64
    ) -> Result<u64> {
        if more == 0 {
            return Err(LandLordErrors::DividendCheckpointCantBeZero.into());
        }

        self.total_dividend_checkpoint = self.total_dividend_checkpoint.checked_add(more).unwrap();
        Ok(self.total_dividend_checkpoint)
    }
}