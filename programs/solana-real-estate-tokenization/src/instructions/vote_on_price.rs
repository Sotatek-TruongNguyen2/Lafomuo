use anchor_lang::prelude::*;
use crate::state::asset_basket::AssetBasket;
use crate::state::platform_governor::PlatformGovernor;
use crate::state::setting::LafomuoSetting;
use crate::errors::LandLordErrors;
/// Accounts for [vote_on_price].
#[derive(Accounts)]
pub struct VoteOnPrice<'info> {
    #[account(
        mut,
        seeds = [   
            b"basket",
            asset_basket.asset_id.key().as_ref(),
            voter.key().as_ref()    ,
            governor.key().as_ref(),
            [asset_basket.basket_id as u8].as_ref()
        ],
        bump = asset_basket.bump,
    )]
    pub asset_basket: Box<Account<'info, AssetBasket>>,

    pub setting: Account<'info, LafomuoSetting>,
    
    #[account(
        has_one = setting @LandLordErrors::SettingAccountMismatched
    )]
    pub governor: Account<'info, PlatformGovernor>,

    // Asset owner
    #[account(mut)]
    pub voter: Signer<'info>,
}

pub fn process_vote_on_price(
    ctx: Context<VoteOnPrice>,
    price: u64
) -> Result<()> {
    Ok(())
}