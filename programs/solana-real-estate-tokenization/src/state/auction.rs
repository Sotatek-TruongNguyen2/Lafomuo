use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub enum AuctionState {
    REDEEM,
    INACTIVE,
    LIVE,
    END
}