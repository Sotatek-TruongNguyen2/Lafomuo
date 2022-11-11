pub mod state;
pub mod instructions;
pub mod assertions;
pub mod errors;
pub mod events;
pub mod constants;
pub mod utils;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("2auz4bjuCFmQGDwX3NYJ8JyNEVWEcMuM1yt44szhrT2i");

#[program]
pub mod solana_real_estate_tokenization {
    use super::*;

    pub fn setup_platform(
        ctx: Context<SetupPlatformGovernor>,
        symbol: String,
        minting_protocol_price: u64, 
        guardians: [Option<Pubkey>; 3]
    ) -> Result<()> {
        setup_platform_governor(ctx, symbol, minting_protocol_price, guardians)?;
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
}
