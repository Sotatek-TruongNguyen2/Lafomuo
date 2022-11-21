use anchor_lang::prelude::*;
use crate::state::fractional_token_escrow::FractionalTokenEscrow;

/// A group of [Escrow]s.
#[account]
#[derive(Copy, Debug, Default)]
pub struct AssetLocker {
    /// Base account used to generate signer seeds.
    pub base: Pubkey,
    /// Bump seed.
    pub bump: u8,
    /// Mint of the token that must be locked in the [Locker].
    pub token_mint: Pubkey,
    /// Total number of tokens locked in [Escrow]s.
    pub locked_supply: u64,
    /// Governor associated with the [Locker].
    pub governor: Pubkey
}


/// Accounts for [locked_voter::new_escrow].
#[derive(Accounts)]
pub struct NewEscrow<'info> {
    /// [Locker].
    pub locker: Account<'info, AssetLocker>,

    /// [Escrow].
    #[account(
        init,
        seeds = [
            b"Escrow".as_ref(),
            locker.key().to_bytes().as_ref(),
            escrow_owner.key().to_bytes().as_ref()
        ],
        bump,
        payer = payer,
        space = FractionalTokenEscrow::LEN
    )]
    pub escrow: Account<'info, FractionalTokenEscrow>,

    /// CHECK: Authority of the [Escrow] to be created.
    pub escrow_owner: UncheckedAccount<'info>,

    /// Payer of the initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}
