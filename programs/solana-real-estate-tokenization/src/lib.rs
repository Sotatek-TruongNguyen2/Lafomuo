pub mod state;
pub mod instructions;
pub mod assertions;
pub mod errors;
pub mod events;
pub mod constants;
pub mod utils;
pub mod merkle_proof;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("2bUX9z3VgNm8yYzqxBMS1Fto3L5r7dkWTUp85ukciBcg");

#[program]
pub mod solana_real_estate_tokenization {
    use super::*;

    pub fn setup_platform(
        ctx: Context<SetupPlatformGovernor>,
        symbol: String,
        minting_protocol_price: u64, 
        min_reserve_factor: u16,
        max_reserve_factor: u16
    ) -> Result<()> {
        setup_platform_governor(ctx, symbol, minting_protocol_price, min_reserve_factor, max_reserve_factor)?;
        Ok(())
    }

    pub fn issue_asset<'a>(
        ctx: Context<'_, '_, '_, 'a, IssueAsset<'a>>,
        uri: String, 
        title: String
    ) -> Result<()> {
        process_issue_asset(ctx, uri, title)?;
        Ok(())
    }

    pub fn fractionalize_asset(
        ctx: Context<FractionalizeNFT>,
        total_supply: u64,
    ) -> Result<()> {
        process_fractionalize_asset(ctx, total_supply)?;
        Ok(())
    }

    pub fn create_dividend_checkpoint(
        ctx: Context<CreateDividendCheckpoint>,
        root: [u8; 32],
        total_distribution_amount: u64
    ) -> Result<()> {
        process_create_dividend_distribution_checkpoint(ctx, root, total_distribution_amount)?;
        Ok(())
    }

    pub fn claim_dividend_by_checkpoint(
        ctx: Context<ClaimDividendCheckpoint>,
        amount: u64,
        proof: Vec<[u8; 32]>
    ) -> Result<()> {
        process_claim_dividend(ctx, amount, proof)?;
        Ok(())
    }

    pub fn new_escrow(
        ctx: Context<NewEscrow>,
    ) -> Result<()> {
        process_new_escrow(ctx)?;
        Ok(())
    }

    pub fn lock(
        ctx: Context<Lock>,
        amount: u64
    ) -> Result<()> {
        process_escrow_lock(ctx, amount)?;
        Ok(())
    }

}
