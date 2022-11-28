use anchor_lang::prelude::*;
use anchor_spl::token::spl_token::state::{Mint, Account as TokenAccount};
use anchor_spl::token::{ID as TokenProgramID};
use crate::errors::LandLordErrors;
use crate::assertions::{assert_initialized, assert_owned_by, cmp_pubkeys};
use crate::state::setting::LafomuoSetting;
use crate::state::platform_governor::PlatformGovernor;

pub fn setup_platform_governor(
    ctx: Context<SetupPlatformGovernor>, 
    symbol: String,
    escrow_lock_duration: i64,
    pre_lock_before_distribution: i64,
    minting_protocol_price: u64,
    min_reserve_factor: u16,
    max_reserve_factor: u16
) -> Result<()> {
    let mut token_mint: Option<Pubkey> = None; 

    if !ctx.remaining_accounts.is_empty() {
        let token_mint_info = &ctx.remaining_accounts[0];
        let _token_mint: Mint = assert_initialized(token_mint_info)?;
        let token_account: TokenAccount = assert_initialized(&ctx.accounts.treasury)?;
    
        assert_owned_by(token_mint_info, &TokenProgramID)?;
        assert_owned_by(&ctx.accounts.treasury, &TokenProgramID)?;

        if !cmp_pubkeys(&token_account.mint, &token_mint_info.key()) {
            return Err(LandLordErrors::TokenAccountMintMismatched.into());
        }

        token_mint = Some(*token_mint_info.key);
    }

    let big_guardian: &Signer = &ctx.accounts.big_guardian;
    let setting = &mut ctx.accounts.setting;

    setting.init(ctx.accounts.governor.key(), escrow_lock_duration, pre_lock_before_distribution, min_reserve_factor, max_reserve_factor)?;

    ctx.accounts.governor.init(
        minting_protocol_price,
        token_mint,
        symbol,
        ctx.accounts.treasury.key(),
        big_guardian.key(),
        setting.key()
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct SetupPlatformGovernor<'info> {
    #[account(init, payer = big_guardian, space = PlatformGovernor::LEN)]
    pub governor: Account<'info, PlatformGovernor>,
    #[account(init, payer = big_guardian, space = LafomuoSetting::LEN)]
    pub setting: Account<'info, LafomuoSetting>,
    #[account(mut)]
    pub big_guardian: Signer<'info>,
    /// CHECK: account checked in CPI
    pub treasury: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}