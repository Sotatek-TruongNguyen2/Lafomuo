const IDL = require("../target/idl/solana_real_estate_tokenization.json");// directory of copy/paste types/your_program.ts file
const anchor = require('@project-serum/anchor');
const bs58 = require('bs58');
const { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createAssociatedTokenAccountInstruction } = require('@solana/spl-token');

const PROGRAM_ID = "7RLLimHKvGkFGZSiVipaBDYGZNKGCve9twDHfdsBDsN9";

function getProgramInstance(connection, wallet) {
    if (!wallet.publicKey) return;
    const provider = new anchor.AnchorProvider(
        connection,
        wallet,
        anchor.AnchorProvider.defaultOptions()
    );
    // Read the generated IDL.
    const idl = IDL;
    // Address of the deployed program.
    const programId = PROGRAM_ID;
    // Generate the program client from IDL.
    const program = new anchor.Program(idl, programId, provider);
    return program;
}

const getCheckpointEscrow = async (programId, locker, escrow_owner) => {
    const [escrowPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("escrow"), locker.toBuffer(), escrow_owner.toBuffer()],
        programId
    );

    return escrowPubkey;
};


(async () => {
    // ========= this part shouldn't be done by FE + BE ( just for demo purpose ) ================
    let assetOwnerSecretKey = bs58.decode("56cbXYaekA6grbUrYM8qL6SMQuVzqQGZUjwruV4oEcuqdaoiJrkBm8P8Rz5d1fVYJDFhJpCE5Ri7Bai39m7rEiLi");
    const assetOwner = anchor.web3.Keypair.fromSecretKey(
        assetOwnerSecretKey
    )

    const connection = new anchor.web3.Connection(anchor.web3.clusterApiUrl("devnet"), "confirmed");
    const wallet = new anchor.Wallet(assetOwner);

    const program = getProgramInstance(connection, wallet);

    let fractionalTokenMint = new anchor.web3.PublicKey("7dvR24tD94yD5Y6PmJUbNCZfk9VU5K6msCkPoiDhbhag")
    let assetLocker = new anchor.web3.PublicKey("2X4fduzrUq5J3oDKuafTHDTjVZzHmRecUu93Yi1Uim2a");
    let escrow = new anchor.web3.PublicKey("9i24BzwLESCuHzjSksK9h3LUr6rUVYj6QLbxSirTujx2");
    let assetOwnerTokenAccount = await getAssociatedTokenAddress(fractionalTokenMint, program.provider.publicKey);
    let escrowHodl = await getAssociatedTokenAddress(
        fractionalTokenMint,
        escrow,
        true
    );

    const escrow_account_hodl = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(program.provider.publicKey, escrowHodl, escrow, fractionalTokenMint),
    );

    const lock_ix = await program.methods.lock(new anchor.BN(10 * (10 ** 8))).accounts(
        {
            escrow,
            escrowOwner: program.provider.publicKey,
            locker: assetLocker,
            escrowTokenHodl: escrowHodl,
            sourceTokens: assetOwnerTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
        }
    ).instruction();

    escrow_account_hodl.add(lock_ix);

    await program.provider.sendAndConfirm(escrow_account_hodl, []);

    console.log("Finish Escrow");
})();