use anchor_lang::error_code;

#[error_code]
pub enum LandLordErrors {
    #[msg("Account has not initialized yet!")]
    AccountNotInitialized,
    #[msg("Not able to unpack account!")]
    NotAbleToUnpackAccount,
    #[msg("Invalid account owner!")]
    IncorrectOwner,
    NotOwnedBySPLProgram,
    TokenAccountMintMismatched,
    TokenAccountOwnerMismatched,
    TokenTransferFailed,
    PublicKeyMismatch,
    NotEnoughTokens,
    NotEnoughSOL,
    MintingAmountCantBeZero,
    DividendCheckpointCantBeZero,
    ImmutableGovernor,
    GovernorMismatch,
    NFTOwnerMismatch,
    NoFreezeAuthoritySet,
    InvalidAccountData,
    FractionalTokenZeroDecimals,
    FractionalTokenSupplyNotPure,
    NFTIsAlreadyFractionalized,
    TokenTreasuryPDANotFound,
    #[msg("Invalid Dividend Distribution Merkle proofs!!")]
    InvalidMerkleProof,
    #[msg("Dividend from this checkpoint is already claimed!!")]
    DividendAlreadyClaimed,
    ExceedsTotalDistributionAmount
}