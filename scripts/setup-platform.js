const IDL = require("../target/idl/solana_real_estate_tokenization.json");// directory of copy/paste types/your_program.ts file
const anchor = require('@project-serum/anchor');

const PROGRAM_ID = "2bUX9z3VgNm8yYzqxBMS1Fto3L5r7dkWTUp85ukciBcg";

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
    const owner = anchor.web3.Keypair.fromSecretKey(
        Uint8Array.from(
            [25, 147, 104, 88, 211, 14, 214, 228, 254, 100, 17, 104, 2, 198, 228, 175, 111, 1, 78, 146, 244, 248, 114, 237, 73, 126, 57, 170, 250, 253, 47, 27, 107, 5, 181, 9, 116, 75, 101, 180, 143, 52, 184, 189, 6, 110, 72, 55, 133, 119, 94, 111, 189, 248, 201, 81, 210, 25, 247, 224, 127, 14, 216, 8]
        )
    )
    const connection = new anchor.web3.Connection(anchor.web3.clusterApiUrl("devnet"), "confirmed");
    const wallet = new anchor.Wallet(owner);

    const program = getProgramInstance(connection, wallet);

    let fst_authority = anchor.web3.Keypair.generate();
    let sec_authority = anchor.web3.Keypair.generate();
    let governor = anchor.web3.Keypair.generate();
    let setting = anchor.web3.Keypair.generate();

    let sol_treasury = anchor.web3.Keypair.generate();
    
    console.log("Sol-treasury: ", sol_treasury.publicKey.toBase58());
    console.log("Governor: ", governor.publicKey.toBase58());
    console.log("setting: ", setting.publicKey.toBase58());

    const tx = await program.methods.setupPlatform(
        "LANDLORD", 
        new anchor.BN(anchor.web3.LAMPORTS_PER_SOL),
        10,
        4000
    )
        .accounts({
            bigGuardian: program.provider.publicKey,
            governor: governor.publicKey,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            systemProgram: anchor.web3.SystemProgram.programId,
            treasury: sol_treasury.publicKey,
            setting: setting.publicKey
        })
        .remainingAccounts([

        ])
        .signers([
            governor,
            setting
        ])
        .rpc();

    console.log(tx);
})();