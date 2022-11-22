use anchor_lang::prelude::*;
use crate::errors::LandLordErrors;
use crate::constants::*;
use anchor_spl::token::{Mint};

use super::auction::AuctionState;

#[account]
pub struct AssetBasket {
    pub asset_tokenize: AssetTokenization,
    pub basket_id: u64,
    pub asset_id: Pubkey,
    pub asset_metadata: Pubkey,
    pub owner: Pubkey,
    pub governor: Pubkey,
    pub auction_state: AuctionState,
    pub average_price: u64,
    pub total_distribution_checkpoint: u64,
    pub iat: i64,
    pub is_freezed: bool,
    pub bump: u8
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct AssetTokenization {
    pub token_mint: Pubkey,
    pub tokenized: bool,
    pub total_supply: u64,
    pub tokenized_at: i64
}

impl AssetBasket {
    pub const LEN: usize = 
        DISCRIMINATOR_LENGTH +
        // asset_tokenize 
        PUBLIC_KEY_LENGTH + // token_mint 
        BOOL_LENGTH + // tokenized 
        U128_LENGTH +  // total_supply + tokenized_at
        U128_LENGTH / 2 + // basket_id 
        PUBLIC_KEY_LENGTH * 4 + // asset_id, asset_metadata, owner, governor
        U128_LENGTH + // Auction State
        U128_LENGTH + // total_distribution_checkpoint, iat
        BOOL_LENGTH + // is_freezed
        U128_LENGTH / U128_LENGTH; // bump (1 byte)

    pub fn init(
        &mut self, 
        basket_id: u64,
        bump: u8,
        asset_metadata: Pubkey,
        governor: Pubkey, 
        owner: Pubkey,
        mint: Pubkey,
    ) -> Result<&mut AssetBasket> {
        let clock: Clock = Clock::get().unwrap();

        self.bump = bump;
        self.basket_id = basket_id;
        self.asset_metadata = asset_metadata;
        self.governor = governor;
        self.iat = clock.unix_timestamp;
        self.asset_id = mint;
        self.owner = owner;
        self.is_freezed = false;
        self.auction_state = AuctionState::INACTIVE;

        Ok(self)
    }

    pub fn fractionalize(
        &mut self,
        total_supply: u64,
        mint: &Account<Mint>
    ) -> Result<()> {
        if self.asset_tokenize.tokenized {
            return Err(LandLordErrors::NFTIsAlreadyFractionalized.into());
        }

        if mint.decimals == 0 {
            return Err(LandLordErrors::FractionalTokenZeroDecimals.into());
        }
    
        if mint.supply > 0 {
            return Err(LandLordErrors::FractionalTokenSupplyNotPure.into());
        }

        let clock: Clock = Clock::get().unwrap();

        self.asset_tokenize = AssetTokenization {
            token_mint: mint.key(),
            tokenized: true,
            tokenized_at:  clock.unix_timestamp,
            total_supply
        };

        Ok(())
    }
}
