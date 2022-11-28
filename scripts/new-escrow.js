const IDL = require("../target/idl/solana_real_estate_tokenization.json");// directory of copy/paste types/your_program.ts file
const anchor = require('@project-serum/anchor');
const bs58 = require('bs58');

const PROGRAM_ID = "7RLLimHKvGkFGZSiVipaBDYGZNKGCve9twDHfdsBDsN9";

const getCheckpointEscrow = async (programId, locker, escrow_owner) => {
    const [escrowPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("escrow"), locker.toBuffer(), escrow_owner.toBuffer()],
      programId
    );

    return escrowPubkey;
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
    // ========= this part shouldn't be done by FE + BE ( just for demo purpose ) ================
    let assetOwnerSecretKey = bs58.decode("56cbXYaekA6grbUrYM8qL6SMQuVzqQGZUjwruV4oEcuqdaoiJrkBm8P8Rz5d1fVYJDFhJpCE5Ri7Bai39m7rEiLi");
    const assetOwner = anchor.web3.Keypair.fromSecretKey(
        assetOwnerSecretKey
    )

    const connection = new anchor.web3.Connection(anchor.web3.clusterApiUrl("devnet"), "confirmed");
    const wallet = new anchor.Wallet(assetOwner);

    const program = getProgramInstance(connection, wallet);

    let assetLocker = new anchor.web3.PublicKey("2X4fduzrUq5J3oDKuafTHDTjVZzHmRecUu93Yi1Uim2a");
    let governor = new anchor.web3.PublicKey("5S6pBPEMspXFbdQ46vBhrd5MmUorccYPFSkJn4hoyL8Y");
    let escrow = await getCheckpointEscrow(program.programId, assetLocker, program.provider.publicKey);

     // Create an escrow for a user
    const tx = await program.methods.newEscrow().accounts({
        escrow,
        escrowOwner: program.provider.publicKey,
        governor: governor,
        locker: assetLocker,
        payer: program.provider.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).rpc({
        commitment: "confirmed"
      });

    console.log(tx);
    console.log("ESCROW: ", escrow.toBase58());
})();