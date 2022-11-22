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

const getTokenTreasury = async (
    programID,
) => {
    const [treasuryPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("spl_token_treasury")],
        programID
    );

    return treasuryPubkey;
};

const getAssetLocker = async (programId, governor, basket_id) => {
    const [assetLockerPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("locker"), governor.toBuffer(), basket_id.toArrayLike(Buffer)],
        programId
    );

    return assetLockerPubkey;
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
    const program = getProgramInstance(connection, admin);

    const fractionalTokenMint = anchor.web3.Keypair.generate();
    const fractionalTokenAccount = await getAssociatedTokenAddress(fractionalTokenMint.publicKey, assetOwner.publicKey);

    // All 3 accounts should be queried from our BE
    // nft token account 
    const nftTokenAccount = new anchor.web3.PublicKey("7vhxHyQrQp5tLYozLiyjzyXMTLwHXwxvFtj9Na7LPzCU");
    const assetBasketAddress = new anchor.web3.PublicKey("4FyKqzT5Ep24pVCmu7d8Xqsg7XAc5tGX3ZzpK5JaSCg5");
    // nft-mint account ( nft address ) 
    const mintKey = new anchor.web3.PublicKey("EJb9G5HwV9FhFqJ7wd19gJzDbwRWVQtZb3tmFeWZN6Cz");

    const assetBasketAccount = await program.account.assetBasket.fetch(assetBasketAddress);
    const assetLocker = await getAssetLocker(program.programId, governor, assetBasketAccount.basketId);
    const treasuryPDA = await getTokenTreasury(program.programId);
    const treasuryNFTTokenAccount = await getAssociatedTokenAddress(mintKey, treasuryPDA, true);
    
    const lamports = await program.provider.connection.getMinimumBalanceForRentExemption(MINT_SIZE);

    const fractional_nft_tx = new anchor.web3.Transaction()
    .add(
        anchor.web3.SystemProgram.createAccount({
            fromPubkey: assetOwner.publicKey,
            newAccountPubkey: fractionalTokenMint.publicKey,
            space: MINT_SIZE,
            programId: TOKEN_PROGRAM_ID,
            lamports,
        }),
        createInitializeMintInstruction(fractionalTokenMint.publicKey, 8, assetOwner.publicKey, assetOwner.publicKey, TOKEN_PROGRAM_ID),
        createAssociatedTokenAccountInstruction(assetOwner.publicKey, fractionalTokenAccount, assetOwner.publicKey, fractionalTokenMint.publicKey),
        createAssociatedTokenAccountInstruction(assetOwner.publicKey, treasuryNFTTokenAccount, treasuryPDA, mintKey),
    );

    const fractionalize_nft_ix = await program.methods.fractionalizeAsset(
        new anchor.BN(10000 * (10 ** 8))
    ).accounts({
        assetBasket: assetBasketAddress,
        bigGuardian: program.provider.publicKey,
        governor,
        mint: fractionalTokenMint.publicKey,
        owner: assetOwner.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenAccount: fractionalTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        mintNft: mintKey,
        treasuryNftTokenAccount: treasuryNFTTokenAccount,
        ownerNftTokenAccount: nftTokenAccount,
        fractionalizeTokenLocker: assetLocker
    }).instruction();

    fractional_nft_tx.add(fractionalize_nft_ix);

    // co che confirm transaction tren solana voi ethereum
    const recentBlockhash = await program.provider.connection.getLatestBlockhash("confirmed");

    console.log("=========== Getting recent blockhash ===========");
    console.log("Recent blockhash: ", recentBlockhash);

    fractional_nft_tx.recentBlockhash = recentBlockhash.blockhash;
    fractional_nft_tx.feePayer = assetOwner.publicKey;
    
    fractional_nft_tx.partialSign(fractionalTokenMint);
    
    // This sign must be done by admin
    fractional_nft_tx.partialSign(admin.payer);

     // This line serves demo purpose only (FE don't need to do this)
     fractional_nft_tx.partialSign(assetOwner);


    const serialized_tx = fractional_nft_tx.serialize({
        requireAllSignatures: false
    });

    // console.log("Tx: ", serialized_tx.toString("base64"));

    // const finalTxHash = await program.provider.connection.sendRawTransaction(serialized_tx);
    // console.log("txHash :: ", finalTxHash)

    const tx_details = await program.provider.connection.getTransaction("4FkwPfcYiMFTLuGLTxc5zjitAUTDY5yEFN4AdVsoHAEtNuaW99FmBapmt7XpyV2vsAu4dQzuHB1jvQSyUMRzwZv4")
    console.log(tx_details);
    // const log = tx_details.meta.logMessages[tx_details.meta.logMessages.length - 3].split("Program data: ")[1];
    // const coder = new anchor.BorshEventCoder(IDL);

    // console.log(coder.decode(log));
})();