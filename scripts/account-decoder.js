const IDL = require("../target/idl/solana_real_estate_tokenization.json");// directory of copy/paste types/your_program.ts file
const anchor = require('@project-serum/anchor');

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
    const coder = new anchor.BorshEventCoder(IDL);
    
    const connection = new anchor.web3.Connection(anchor.web3.clusterApiUrl("devnet"), "confirmed");

    const txs = [];

    let governor = new anchor.web3.PublicKey("5wBbT46X1G7kAL49wahJspTqruVo8KB5NdhvY925KgiC");

    let lastTransactions = await connection.getConfirmedSignaturesForAddress2(
        governor,
        {
            limit: 1
        },
        "finalized"
    );

    
    let before = lastTransactions[lastTransactions.length - 1].signature;
    txs.push(...lastTransactions);

    while (true) {
        lastTransactions = await connection.getConfirmedSignaturesForAddress2(
            governor,
            {
                limit: 5,
                before
            }
        );

        if (lastTransactions.length > 0) {
            before = lastTransactions[lastTransactions.length - 1].signature;
            txs.push(...lastTransactions);

            continue;
        }

        break;
    }

    console.log(txs);

    let signatureList = txs.map(tx => tx.signature);
    let transactionDetails = await connection.getParsedTransactions(signatureList, {
        maxSupportedTransactionVersion: 0,
    });

    let logs = transactionDetails.map((txDetail, n)=>{
        let logMessages = txDetail.meta.logMessages;
        console.log(logMessages);
        return coder.decode(logMessages[logMessages.findIndex(value => /^Program log: landlord-log/.test(value)) + 1].split("Program log: ")[1]);
    })

    console.log(logs);


    // const program = getProgramInstance(connection, wallet);

    // const accounts = await program.account.assetBasket.all();         
    // const p = await program.provider.connection.getAccountInfo(accounts[0].publicKey);
    // console.log(p);
    // const coder = new anchor.BorshAccountsCoder(IDL);
    // console.log("LOL :", await coder.decode("AssetBasket", p.data))

})();