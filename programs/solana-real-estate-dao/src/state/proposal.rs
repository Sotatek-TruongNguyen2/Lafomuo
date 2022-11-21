use anchor_lang::prelude::*;

/// A Proposal is a pending transaction that may or may not be executed by the DAO.
#[account]
pub struct Proposal {
    /// The instructions associated with the proposal.
    pub instructions: Vec<LafomuoInstruction>,

    /// The public key of the governor.
    pub governor: Pubkey,
    /// The unique ID of the proposal, auto-incremented.
    pub index: u64,
    /// Bump seed
    pub bump: u8,

    /// The public key of the proposer.
    pub proposer: Pubkey,

    /// The number of votes in support of a proposal required in order for a quorum to be reached and for a vote to succeed
    pub quorum_votes: u64,
    /// Current number of votes in favor of this proposal
    pub for_votes: u64,
    /// Current number of votes in opposition to this proposal
    pub against_votes: u64,
    /// Current number of votes for abstaining for this proposal
    pub abstain_votes: u64,

    /// The timestamp when the proposal was canceled.
    pub canceled_at: i64,
    /// The timestamp when the proposal was created.
    pub created_at: i64,
    /// The timestamp in which the proposal was activated.
    /// This is when voting begins.
    pub activated_at: i64,
    /// The timestamp when voting ends.
    /// This only applies to active proposals.
    pub voting_ends_at: i64,

    // /// The timestamp in which the proposal was queued, i.e.
    // /// approved for execution on the Smart Wallet.
    // pub queued_at: i64,
    // /// If the transaction was queued, this is the associated Goki Smart Wallet transaction.
    // pub queued_transaction: Pubkey,
}

impl Proposal {
    /// Space that the [Proposal] takes up.
    pub fn space(instructions: Vec<LafomuoInstruction>) -> usize {
        4  // Anchor discriminator.
        + 4 // Vec discriminator
            + std::mem::size_of::<Proposal>()
        + (instructions.iter().map(|ix| ix.space()).sum::<usize>())
    }
}

/// Instruction.
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct LafomuoInstruction {
    /// Pubkey of the instruction processor that executes this instruction
    pub program_id: Pubkey,
    /// Metadata for what accounts should be passed to the instruction processor
    pub keys: Vec<AccountMeta>,
    /// Opaque data passed to the instruction processor
    pub data: Vec<u8>,
}

impl LafomuoInstruction {
    /// Space that a [ProposalInstruction] takes up.
    pub fn space(&self) -> usize {
        std::mem::size_of::<Pubkey>()
            + (self.keys.len() as usize) * std::mem::size_of::<AccountMeta>()
            + (self.data.len() as usize)
    }
}

/// Account metadata used to define Instructions
#[derive(AnchorSerialize, AnchorDeserialize, Debug, PartialEq, Copy, Clone)]
pub struct AccountMeta {
    /// An account's public key
    pub pubkey: Pubkey,
    /// True if an Instruction requires a Transaction signature matching `pubkey`.
    pub is_signer: bool,
    /// True if the `pubkey` can be loaded as a read-write account.
    pub is_writable: bool,
}
