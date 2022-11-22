use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::LandLordErrors;

#[account]
pub struct LafomuoSetting {
    pub governor: Pubkey,
    pub min_reserve_factor: u16,
    pub max_reserve_factor: u16,
}   

impl LafomuoSetting {
    pub const LEN: usize = 
        DISCRIMINATOR_LENGTH + // Discriminator
        PUBLIC_KEY_LENGTH + // governor
        U128_LENGTH / 4; // min_reserve_factor + max_reserve_factor

    pub fn init(
        &mut self,
        governor: Pubkey,
        min_reserve_factor: u16,
        max_reserve_factor: u16
    ) -> Result<()> {

        if min_reserve_factor == 0 || max_reserve_factor == 0 {
            return Err(LandLordErrors::InvalidReserveFactorForSetting.into());
        }

        if min_reserve_factor > max_reserve_factor {
            return Err(LandLordErrors::MinReserveFactorGreaterThanMax.into());
        }

        if min_reserve_factor > BASIS_POINT || max_reserve_factor > BASIS_POINT {
            return Err(LandLordErrors::ReserveFactorMoreThanBasisPoint.into());
        }

        self.min_reserve_factor = min_reserve_factor;
        self.max_reserve_factor = max_reserve_factor;
        self.governor = governor;

        Ok(())
    }
}