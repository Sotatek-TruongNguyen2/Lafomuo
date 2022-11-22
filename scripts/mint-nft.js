const IDL = require("../target/idl/solana_real_estate_tokenization.json");// directory of copy/paste types/your_program.ts file
const anchor = require('@project-serum/anchor');
const {
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    getAssociatedTokenAddress,
    TOKEN_PROGRAM_ID,
    MINT_SIZE,
} = require("@solana/spl-token");
const PROGRAM_ID = "FZtmv1R8AgFU4K7TnD5pyANFVbz2dVvb4UkW9E14n5hm";
const bs58 = require('bs58');

const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

const getAssetBasket = async (
    programID,
    governor,
    asset_owner,
    mint,
    basket_id
) => {
    const [assetBasketPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("basket"), mint.toBuffer(), asset_owner.toBuffer(), governor.toBuffer(), basket_id.toArrayLike(Buffer)],
        programID
    );

    return assetBasketPubkey;
};

const getMetadata = async (mint) => {
    const [metadataPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        TOKEN_METADATA_PROGRAM_ID
    );

    return metadataPubkey;
};


const getMasterEdition = async (mint) => {
    const [masterEditionPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from("edition")],
        TOKEN_METADATA_PROGRAM_ID
    );

    return masterEditionPubkey;
};

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

(async () => {
    const adminKeyPair = anchor.web3.Keypair.fromSecretKey(
        Uint8Array.from(
            [25, 147, 104, 88, 211, 14, 214, 228, 254, 100, 17, 104, 2, 198, 228, 175, 111, 1, 78, 146, 244, 248, 114, 237, 73, 126, 57, 170, 250, 253, 47, 27, 107, 5, 181, 9, 116, 75, 101, 180, 143, 52, 184, 189, 6, 110, 72, 55, 133, 119, 94, 111, 189, 248, 201, 81, 210, 25, 247, 224, 127, 14, 216, 8]
        )
    )
    const connection = new anchor.web3.Connection(anchor.web3.clusterApiUrl("devnet"), "confirmed");
    const admin = new anchor.Wallet(adminKeyPair);

    console.log(bs58.encode(adminKeyPair.publicKey.toBuffer()));

    // ========= this part shouldn't be done by FE + BE ( just for demo purpose ) ================
    let assetOwnerSecretKey = bs58.decode("56cbXYaekA6grbUrYM8qL6SMQuVzqQGZUjwruV4oEcuqdaoiJrkBm8P8Rz5d1fVYJDFhJpCE5Ri7Bai39m7rEiLi");
    const assetOwner = anchor.web3.Keypair.fromSecretKey(
        assetOwnerSecretKey
    )

    // sol_treasury, governor: Provided by admin
    let sol_treasury = new anchor.web3.PublicKey("6tzHJCsUgHg5AaSsJ2bk829ZU6KkxkLQAvpZBytBo6kM");
    let governor = new anchor.web3.PublicKey("2Auyf1oknPZZvJyabr2heEsecCJyDahdDAPn6L3FqhRm");

    // Must be by FE
    const mintKey = anchor.web3.Keypair.generate();
    const nftTokenAccount = anchor.web3.Keypair.generate();

    const program = getProgramInstance(connection, admin);
    const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(MINT_SIZE);

    console.log("==== Setup minting tx ====");

    const mint_tx = new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.createAccount({
            fromPubkey: assetOwner.publicKey,
            newAccountPubkey: mintKey.publicKey,
            space: MINT_SIZE,
            programId: TOKEN_PROGRAM_ID,
            lamports,
        }),
        createInitializeMintInstruction(mintKey.publicKey, 0, assetOwner.publicKey, assetOwner.publicKey, TOKEN_PROGRAM_ID),
    );

    // Mint NFT - Fractionalize - Create Dividend Checkpoint - Claim dividend (Finish)
    // DAO - Buyout 
    // Crawlers (Aleph-im) - 3 services (API - Cron job - Aleph)

    // createAssociatedTokenAccountInstruction(assetOwner.publicKey, nftTokenAccount, assetOwner.publicKey, mintKey.publicKey)

    console.log("===== Start initializing Mint and token account ====== ");

    // await program.provider.sendAndConfirm(mint_tx, [assetOwner, mintKey]);

    console.log("====== Finish minting =====");
    console.log("Mint token: ", mintKey.publicKey.toBase58());

    console.log("==== Start Issuing Asset ====");

    // ===== This part must be done by BE

    const metadataAddress = await getMetadata(mintKey.publicKey);
    const masterEdition = await getMasterEdition(mintKey.publicKey);
    const governorAccount = await program.provider.connection.getAccountInfo(governor);
    const governorDetails = program.coder.accounts.decode("PlatformGovernor", governorAccount.data);

    const assetBasketAddress = await getAssetBasket(
        program.programId,
        governor,
        assetOwner.publicKey,
        mintKey.publicKey,
        governorDetails.totalAssetsMinted.add(new anchor.BN(1))
    );

    // first data will be signed by big guardian
    const ix = await program.methods.issueAsset("https://basc.s3.amazonaws.com/meta/3506.json", "House A").accounts(
        {
            bigGuardian: program.provider.publicKey,
            governor: governor,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            systemProgram: anchor.web3.SystemProgram.programId,
            treasury: sol_treasury,
            masterEdition,
            metadata: metadataAddress,
            tokenProgram: TOKEN_PROGRAM_ID,
            mint: mintKey.publicKey,
            mintAuthority: assetOwner.publicKey,
            updateAuthority: assetOwner.publicKey,
            tokenAccount: nftTokenAccount.publicKey,
            owner: assetOwner.publicKey,
            tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
            assetBasket: assetBasketAddress
        }
    ).instruction();

    // const tx = new anchor.web3.Transaction(
    //     {
    //         recentBlockhash: recentBlockhash.blockhash,
    //         feePayer: assetOwner.publicKey,
    //     },
    // );

    mint_tx.add(ix);


    // co che confirm transaction tren solana voi ethereum
    const recentBlockhash = await program.provider.connection.getLatestBlockhash("confirmed");

    console.log("=========== Getting recent blockhash ===========");
    console.log("Recent blockhash: ", recentBlockhash);

    mint_tx.recentBlockhash = recentBlockhash.blockhash;
    mint_tx.feePayer = assetOwner.publicKey;

    mint_tx.partialSign(mintKey);
    mint_tx.partialSign(nftTokenAccount);
    mint_tx.partialSign(assetOwner);

    // console.log(tx);
    // console.log(mint_tx.serialize({ requireAllSignatures: false }).toString("base64"))
    // mint_tx.partialSign(admin.payer);

    const tx = anchor.web3.Transaction.from(Buffer.from(await mint_tx.serialize({
        requireAllSignatures: false
    }), "base64"));


    tx.partialSign(admin.payer);
    // console.log(tx.serialize({ requireAllSignatures: false }).toString("base64"))

    const serialized_tx = tx.serialize({
        requireAllSignatures: false
    });

    // console.log("Tx: ", serialized_tx.toString("base64"));

    const finalTxHash = await program.provider.connection.sendRawTransaction(serialized_tx);
    console.log("txHash :: ", finalTxHash)
})();