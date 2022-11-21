import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolanaRealEstateTokenization } from "../target/types/solana_real_estate_tokenization";
import {
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createMintToInstruction,
} from "@solana/spl-token";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { Metadata } from "@metaplex-foundation/mpl-token-metadata";
import { expect } from "chai";
import { bufferToArray } from "./utils";
import { keccak_256 } from "js-sha3";
import { MerkleTree } from 'merkletreejs';
// import IDL from '../target/idl/solana_real_estate_tokenization.json';
// import tou8 from 'buffer-to-uint8array';

type Keypair = anchor.web3.Keypair;

function getProgramInstance(
  provider: anchor.Provider,
  programId: anchor.web3.PublicKey,
  IDL: any
): anchor.Program {
  // Read the generated IDL.
  const idl = IDL;
  // Address of the deployed program.
  // Generate the program client from IDL.
  const program = new anchor.Program(idl, programId, provider);
  return program;
}

describe("solana-real-estate-tokenization", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolanaRealEstateTokenization as Program<SolanaRealEstateTokenization>;

  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

  const getTokenTreasury = async (
    programID: anchor.web3.PublicKey,
  ): Promise<anchor.web3.PublicKey> => {
    const [treasuryPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("spl_token_treasury")],
      programID
    );

    return treasuryPubkey;
  };

  const getAssetLocker = async (programId: anchor.web3.PublicKey, governor: anchor.web3.PublicKey, basket_id: anchor.BN): Promise<anchor.web3.PublicKey> => {
    const [assetLockerPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("locker"), governor.toBuffer(), basket_id.toArrayLike(Buffer)],
      programId
    );

    return assetLockerPubkey;
  };

  const getDividendClaimedDetails = async (programId: anchor.web3.PublicKey, dividend_distributor: anchor.web3.PublicKey, claimer: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> => {
    const [metadataPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("claim_dividend"), dividend_distributor.toBuffer(), claimer.toBuffer()],
      programId
    );

    return metadataPubkey;
  };

  const getMetadata = async (mint: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> => {
    const [metadataPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      TOKEN_METADATA_PROGRAM_ID
    );

    return metadataPubkey;
  };


  const getMasterEdition = async (mint: anchor.web3.PublicKey): Promise<anchor.web3.PublicKey> => {
    const [masterEditionPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from("edition")],
      TOKEN_METADATA_PROGRAM_ID
    );

    return masterEditionPubkey;
  };

  const getAssetBasket = async (
    programID: anchor.web3.PublicKey,
    governor: anchor.web3.PublicKey,
    asset_owner: anchor.web3.PublicKey,
    mint: anchor.web3.PublicKey,
    basket_id: anchor.BN
  ): Promise<anchor.web3.PublicKey> => {
    const [assetBasketPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("basket"), mint.toBuffer(), asset_owner.toBuffer(), governor.toBuffer(), basket_id.toArrayLike(Buffer)],
      programID
    );

    return assetBasketPubkey;
  };

  let assetOwner: Keypair;
  let fst_authority: Keypair;
  let sec_authority: Keypair;
  let governor: Keypair;
  let sol_treasury: Keypair;
  let dividend_distributor: Keypair;
  let paymentToken: Keypair;
  let claimedDividend: Keypair;

  it("Is initialized!", async () => {
    claimedDividend = anchor.web3.Keypair.generate();
    paymentToken = anchor.web3.Keypair.generate();
    dividend_distributor = anchor.web3.Keypair.generate();
    sol_treasury = anchor.web3.Keypair.generate();
    governor = anchor.web3.Keypair.generate();
    assetOwner = anchor.web3.Keypair.generate();
    fst_authority = anchor.web3.Keypair.generate();
    sec_authority = anchor.web3.Keypair.generate();

    const mintKey: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    const nftTokenAccount = anchor.web3.Keypair.generate();

    const signature = await program.provider.connection.requestAirdrop(
      assetOwner.publicKey,
      LAMPORTS_PER_SOL * 10000
    );

    await program.provider.connection.requestAirdrop(
      fst_authority.publicKey,
      LAMPORTS_PER_SOL * 100
    );

    // await program.provider.connection.requestAirdrop(
    //   sol_treasury.publicKey,
    //   LAMPORTS_PER_SOL * 100
    // );

    await program.provider.connection.confirmTransaction(signature);

    console.log("====== AIRDROP SUCCESSFUL! ======");

    const lamports: number = await program.provider.connection.getMinimumBalanceForRentExemption(MINT_SIZE);
    console.log({
      fromPubkey: assetOwner.publicKey,
      newAccountPubkey: mintKey.publicKey,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM_ID,
      lamports,
    })
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

    console.log("===== Start initializing Mint and token account ====== ", program.programId.toBase58());

    await program.methods.setupPlatform("LANDLORD", new anchor.BN(anchor.web3.LAMPORTS_PER_SOL))
      .accounts({
        bigGuardian: program.provider.publicKey,
        governor: governor.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
        treasury: sol_treasury.publicKey
      })
      .remainingAccounts([

      ])
      .signers([
        governor
      ]).rpc({
        commitment: "confirmed"
      });


    // const governorInfo = await program.account.platformGovernor.fetch(governor.publicKey);

    const metadataAddress = await getMetadata(mintKey.publicKey);
    const masterEdition = await getMasterEdition(mintKey.publicKey);
    // programID: anchor.web3.PublicKey,
    // governor: anchor.web3.PublicKey,
    // asset_owner: anchor.web3.PublicKey,
    // mint: anchor.web3.PublicKey
    const assetBasketAddress = await getAssetBasket(
      program.programId,
      governor.publicKey,
      assetOwner.publicKey,
      mintKey.publicKey,
      new anchor.BN(1)
    );

    const ix = await program.methods.issueAsset("https://basc.s3.amazonaws.com/meta/3506.json", "Bored Apes").accounts(
      {
        bigGuardian: program.provider.publicKey,
        governor: governor.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
        treasury: sol_treasury.publicKey,
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
    ).signers([
      // nftTokenAccount,
      // assetOwner
    ]).instruction();

    mint_tx.add(ix);

    await program.provider.sendAndConfirm(mint_tx, [assetOwner, nftTokenAccount, mintKey]);

    const mintAccountInfo = await program.provider.connection.getParsedAccountInfo(mintKey.publicKey);

    const metadataInfo = await Metadata.fromAccountAddress(
      program.provider.connection,
      metadataAddress
    );

    const parsedMintAccount = (mintAccountInfo.value.data as any).parsed;

    expect(parsedMintAccount.info.mintAuthority).to.be.equals(masterEdition.toBase58());
    expect(parsedMintAccount.info.freezeAuthority).to.be.equals(masterEdition.toBase58());
    expect(metadataInfo.updateAuthority.toBase58()).to.be.equals(assetOwner.publicKey.toBase58());
    expect(metadataInfo.mint.toBase58()).to.be.equals(mintKey.publicKey.toBase58());

    console.log(await program.account.assetBasket.fetch(assetBasketAddress));
    console.log((await program.account.platformGovernor.fetch(governor.publicKey)));

    const tokenAccountInfo = await program.provider.connection.getParsedAccountInfo(nftTokenAccount.publicKey);
    const tokenAccountInfoData = tokenAccountInfo.value.data as any;

    expect(tokenAccountInfoData.parsed.info.mint).to.be.equals(mintKey.publicKey.toBase58());
    expect(tokenAccountInfoData.parsed.info.owner).to.be.equals(assetOwner.publicKey.toBase58());
    expect(tokenAccountInfoData.parsed.info.tokenAmount.amount).to.be.equals("1");

    // ================= Fractionalize NFT ====================

    const fractionalTokenMint = anchor.web3.Keypair.generate();
    const fractionalTokenAccount = await getAssociatedTokenAddress(fractionalTokenMint.publicKey, assetOwner.publicKey);

    const assetBasketAccount = await program.account.assetBasket.fetch(assetBasketAddress);
    const assetLocker = await getAssetLocker(program.programId, governor.publicKey, assetBasketAccount.basketId);
    const treasuryPDA = await getTokenTreasury(program.programId);
    const treasuryNFTTokenAccount = await getAssociatedTokenAddress(mintKey.publicKey, treasuryPDA, true);

    const fractional_nft_tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: assetOwner.publicKey,
        newAccountPubkey: fractionalTokenMint.publicKey,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
        lamports,
      }),
      createInitializeMintInstruction(fractionalTokenMint.publicKey, 8, assetOwner.publicKey, assetOwner.publicKey, TOKEN_PROGRAM_ID),
      createAssociatedTokenAccountInstruction(assetOwner.publicKey, fractionalTokenAccount, assetOwner.publicKey, fractionalTokenMint.publicKey),
      createAssociatedTokenAccountInstruction(assetOwner.publicKey, treasuryNFTTokenAccount, treasuryPDA, mintKey.publicKey),
    );

    const fractionalize_nft_ix = await program.methods.fractionalizeAsset(
      new anchor.BN(10000 * (10 ** 8))
    ).accounts({
      assetBasket: assetBasketAddress,
      bigGuardian: program.provider.publicKey,
      governor: governor.publicKey,
      mint: fractionalTokenMint.publicKey,
      owner: assetOwner.publicKey,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenAccount: fractionalTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
      mintNft: mintKey.publicKey,
      treasuryNftTokenAccount: treasuryNFTTokenAccount,
      ownerNftTokenAccount: nftTokenAccount.publicKey,
      assetLocker: assetLocker
    }).signers([
      // assetOwner
    ]).instruction();

    fractional_nft_tx.add(fractionalize_nft_ix);

    await program.provider.sendAndConfirm(fractional_nft_tx, [assetOwner, fractionalTokenMint]);

    console.log("Finish fractional !!!!");

    const DIVIDEND_AIRDROP = [
      {
        dividend_distributor: dividend_distributor.publicKey,
        claimer: fst_authority.publicKey,
        amount: new anchor.BN(10 * (10 ** 8))
      },
      {
        dividend_distributor: dividend_distributor.publicKey,
        claimer: sec_authority.publicKey,
        amount: new anchor.BN(100 * (10 ** 8))
      },
    ];

    let dividendHashes = [];

    for (let dividend of DIVIDEND_AIRDROP) {
      let bufferOfDividendClaimData = Buffer.concat([
        // bufferOfDividendClaimData,
        dividend.dividend_distributor.toBuffer(),
        dividend.claimer.toBuffer(),
        dividend.amount.toBuffer("le", 8),
      ])

      dividendHashes.push(Buffer.from(keccak_256.digest(bufferOfDividendClaimData)).toString("hex"));
    }

    const tree = new MerkleTree(dividendHashes, keccak_256, { sort: true });
    const root = bufferToArray(tree.getRoot());

    const assetOwnerPaymentAccount = await getAssociatedTokenAddress(paymentToken.publicKey, assetOwner.publicKey);
    const treasuryPaymentAccount = await getAssociatedTokenAddress(paymentToken.publicKey, treasuryPDA, true);
    const fstAuthorityPaymentAccount = await getAssociatedTokenAddress(paymentToken.publicKey, fst_authority.publicKey);

    const mint_payment_token_tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: assetOwner.publicKey,
        newAccountPubkey: paymentToken.publicKey,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
        lamports,
      }),
      createInitializeMintInstruction(paymentToken.publicKey, 8, assetOwner.publicKey, assetOwner.publicKey, TOKEN_PROGRAM_ID),
      createAssociatedTokenAccountInstruction(assetOwner.publicKey, treasuryPaymentAccount, treasuryPDA, paymentToken.publicKey),
      createAssociatedTokenAccountInstruction(assetOwner.publicKey, assetOwnerPaymentAccount, assetOwner.publicKey, paymentToken.publicKey),
      createAssociatedTokenAccountInstruction(assetOwner.publicKey, fstAuthorityPaymentAccount, fst_authority.publicKey, paymentToken.publicKey),
      createMintToInstruction(paymentToken.publicKey, assetOwnerPaymentAccount, assetOwner.publicKey, new anchor.BN(10000 * (10 ** 8)).toNumber()),
    );

    await program.provider.sendAndConfirm(mint_payment_token_tx, [assetOwner, paymentToken]);

    const tx = await program.methods.createDividendCheckpoint(
      root,
      new anchor.BN(1000 * (10 ** 8)),
    ).accounts({
      dividendDistributor: dividend_distributor.publicKey,
      governor: governor.publicKey,
      mint: paymentToken.publicKey,
      owner: assetOwner.publicKey,
      ownerTokenAccount: assetOwnerPaymentAccount,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      treasuryTokenAccount: treasuryPaymentAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      assetBasket: assetBasketAddress
    }).signers([
      assetOwner,
      dividend_distributor
    ]).rpc({
      commitment: "confirmed"
    });

    // const tx_details = await program.provider.connection.getTransaction(tx, {
    //   commitment: "confirmed",
    // });

    // console.log(tx_details);
    // const log = tx_details.meta.logMessages[9].split("Program data: ")[1];
    // const coder = new anchor.BorshEventCoder(IDL);

    // console.log(coder.decode(log));

    const dividendClaimDetails = await getDividendClaimedDetails(program.programId, dividend_distributor.publicKey, fst_authority.publicKey);

    const proofs = tree.getProof(dividendHashes[0]).map(proof => {
      return bufferToArray(proof.data);
    });

    // console.log(((await program.provider.connection.getParsedAccountInfo(treasuryPaymentAccount)).value.data as any).parsed);

    await program.methods.claimDividendByCheckpoint(
      new anchor.BN(10 * (10 ** 8)),
      proofs
    ).accounts({
      claimedDividend: dividendClaimDetails,
      claimer: fst_authority.publicKey,
      claimerTokenAccount: fstAuthorityPaymentAccount,
      dividendDistributor: dividend_distributor.publicKey,
      treasuryTokenAccountAuthority: treasuryPDA,
      treasuryTokenAccount: treasuryPaymentAccount,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([
      fst_authority
    ]).rpc();

  //   await program.methods.claimDividendByCheckpoint(
  //     new anchor.BN(10 * (10 ** 8)),
  //     proofs
  //   ).accounts({
  //     claimedDividend: dividendClaimDetails,
  //     claimer: fst_authority.publicKey,
  //     claimerTokenAccount: fstAuthorityPaymentAccount,
  //     dividendDistributor: dividend_distributor.publicKey,
  //     treasuryTokenAccountAuthority: treasuryPDA,
  //     treasuryTokenAccount: treasuryPaymentAccount,
  //     rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     systemProgram: anchor.web3.SystemProgram.programId,
  //     tokenProgram: TOKEN_PROGRAM_ID,
  //   }).signers([
  //     fst_authority
  //   ]).rpc();
  });
});

