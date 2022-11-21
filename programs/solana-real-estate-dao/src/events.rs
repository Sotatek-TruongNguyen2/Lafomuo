use anchor_lang::prelude::*;
use crate::state::{GovernanceParameters, LafomuoInstruction};

/// Log to Program Log with a prologue so transaction scraper knows following line is valid mango log
///
/// Warning: This will allocate large buffers on the heap which will never be released as Solana
/// uses a simple bump allocator where free() is a noop. Since the max heap size is limited
// (32k currently), calling this multiple times can lead to memory allocation failures.
#[macro_export]
macro_rules! Lafomuo_emit {
    ($e:expr) => {
        msg!("Lafomuo-log");
        emit!($e);
    };
}

/// Event called in [govern::create_governor].
#[event]
pub struct GovernorCreateEvent {
    /// The governor being created.
    #[index]
    pub governor: Pubkey,
    /// The electorate of the created [Governor].
    pub electorate: Pubkey,
    /// The [SmartWallet].
    pub proposer: Pubkey,
    /// Governance parameters.
    pub parameters: GovernanceParameters,
}
/// Event called in [govern::create_proposal].
#[event]
pub struct ProposalCreateEvent {
    /// The governor.
    #[index]
    pub governor: Pubkey,
    /// The proposal being created.
    #[index]
    pub proposal: Pubkey,
    /// The index of the [Proposal].
    pub index: u64,
    /// Instructions in the proposal.
    pub instructions: Vec<LafomuoInstruction>,
}