const IDL = require("../target/idl/solana_real_estate_tokenization.json");// directory of copy/paste types/your_program.ts file
const anchor = require('@project-serum/anchor');
const {
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    getAssociatedTokenAddress,
    TOKEN_PROGRAM_ID,
    MINT_SIZE,
} = require("@solana/spl-token");
const PROGRAM_ID = "2bUX9z3VgNm8yYzqxBMS1Fto3L5r7dkWTUp85ukciBcg";
const bs58 = require('bs58');
const { sha256 } = require("js-sha256");

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
     let sol_treasury = new anchor.web3.PublicKey("8zhub8g59f2hyn2VKyho1mvUbY1vXkTZFHkFWibLEz8q");
     let governor = new anchor.web3.PublicKey("5MvPJqs6MVymt6XjUUHPdwVnD7yCMntTHzTHbUrYj5VL");

    // Must be by FE
    const program = getProgramInstance(connection, admin);

    const fractionalTokenMint = anchor.web3.Keypair.generate();
    const fractionalTokenAccount = await getAssociatedTokenAddress(fractionalTokenMint.publicKey, assetOwner.publicKey);

    // All 3 accounts should be queried from our BE
    // nft token account 
    const nftTokenAccount = new anchor.web3.PublicKey("E4Vs2bj4FHdi5RbXZsufEFHi2bz1dwuyoW4rvpKVtJj8");
    const assetBasketAddress = new anchor.web3.PublicKey("E7JFbq8o3auKMSRU4yydxMwNajHYAetxvj8CAvQGEnhZ");
    // nft-mint account ( nft address ) 
    const mintKey = new anchor.web3.PublicKey("ApoRqaZB5SLB9wTZHofn7815p2uX2kqBn1mGWeeBQAF3");

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

    console.log("Tx: ", serialized_tx.toString("base64"));

    const finalTxHash = await program.provider.connection.sendRawTransaction(serialized_tx);
    console.log("txHash :: ", finalTxHash)

    // console.log(
    //     treasuryNFTTokenAccount.toBase58(),
    //     (
    //         await program.provider.connection.getTokenAccountsByOwner(assetOwner.publicKey, {
    //             mint: new anchor.web3.PublicKey("9byF2kYGCRpwQjWF7hME1gVpQ5wZmNixijs6oYWRoXPD"),
    //             programId: TOKEN_PROGRAM_ID
    //         })
    //     ).value[0].pubkey.toBase58()
    // );

    // console.log(await program.provider.connection.getTokenAccountBalance(new anchor.web3.PublicKey("FomJvbqxeQyL1ZJVccZ33JTmFhGLiCDLFrX8XBr2dnAf")));

    // const tx_details = await program.provider.connection.getTransaction("4FkwPfcYiMFTLuGLTxc5zjitAUTDY5yEFN4AdVsoHAEtNuaW99FmBapmt7XpyV2vsAu4dQzuHB1jvQSyUMRzwZv4")
    // console.log(tx_details);
    // // const log = tx_details.meta.logMessages[tx_details.meta.logMessages.length - 3].split("Program data: ")[1];
    // // const coder = new anchor.BorshEventCoder(IDL);

    // // console.log(coder.decode(log));
    // const fractionalizeTokenLockerDiscriminator = Buffer.from(sha256.digest('account:FractionalizedTokenLocker')).slice(0, 8);
    // const assetBasketDiscriminator = Buffer.from(sha256.digest('account:AssetBasket')).slice(0, 8);
    // console.log(
    //     await program.provider.connection.getProgramAccounts(new anchor.web3.PublicKey(PROGRAM_ID))
    // );
    // const assetBasketAccounts = await connection.getProgramAccounts(new anchor.web3.PublicKey(PROGRAM_ID), {
    //     filters: [
    //         { memcmp: { offset: 0, bytes: bs58.encode(assetBasketDiscriminator) } }, // Ensure it's a CandyMachine account.
    //         { memcmp: { offset: 8 + 32 + 1 + 16 + 8 + 32 * 3, bytes: governor.toBase58() } }, // Ensure it's a CandyMachine account.
    //     ],
    // })

    // const lockerAccount = await connection.getProgramAccounts(new anchor.web3.PublicKey(PROGRAM_ID), {
    //     filters: [
    //         { memcmp: { offset: 0, bytes: bs58.encode(fractionalizeTokenLockerDiscriminator) } }, // Ensure it's a CandyMachine account.
    //         { memcmp: { offset: 8 + 32 * 2, bytes: bs58.encode(governor.toBuffer()) } }, // Ensure it's a CandyMachine account.
    //     ],
    // })

    // console.log("AssetBasket length: ", assetBasketAccounts.length);

    // // parse AssetBasket account info
    // for (let account of assetBasketAccounts) {
    //     console.log(`==== Account: ${account.pubkey.toBase58()} ====`);
    //     const coder = new anchor.BorshAccountsCoder(IDL);
    //     console.log(coder.decode("AssetBasket", account.account.data))
    // }

    // // parse FractionalizeTokenLocker account info
    // for (let account of lockerAccount) {
    //     console.log(`==== Account: ${account.pubkey.toBase58()} ====`);
    //     const coder = new anchor.BorshAccountsCoder(IDL);
    //     const a = coder.decode("FractionalizedTokenLocker", account.account.data);
    //     console.log(a.assetId.toBase58());
    // }
})();