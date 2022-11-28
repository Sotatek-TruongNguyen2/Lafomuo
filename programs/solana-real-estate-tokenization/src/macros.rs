/// Generates the signer seeds for an [crate::Escrow].
#[macro_export]
macro_rules! escrow_seeds {
    ($escrow: expr) => {
        &[
            b"escrow" as &[u8],
            &$escrow.locker.to_bytes(),
            &$escrow.owner.to_bytes(),
            &[$escrow.bump],
        ]
    };
}