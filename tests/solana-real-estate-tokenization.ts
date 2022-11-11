import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolanaRealEstateTokenization } from "../target/types/solana_real_estate_tokenization";
import {
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
  MINT_SIZE,

} from "@solana/spl-token";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { Metadata, MasterEditionV2 } from "@metaplex-foundation/mpl-token-metadata";
import { expect } from "chai";

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
    mint: anchor.web3.PublicKey
  ): Promise<anchor.web3.PublicKey> => {
    const [assetBasketPubkey, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("basket"), mint.toBuffer(), asset_owner.toBuffer(), governor.toBuffer()],
      programID
    );

    return assetBasketPubkey;
  };

  let assetOwner: Keypair;
  let fst_authority: Keypair;
  let sec_authority: Keypair;
  let governor: Keypair;
  let sol_treasury: Keypair;

  it("Is initialized!", async () => {
    sol_treasury = anchor.web3.Keypair.generate();
    governor = anchor.web3.Keypair.generate();
    assetOwner = anchor.web3.Keypair.generate();
    fst_authority = anchor.web3.Keypair.generate();
    sec_authority = anchor.web3.Keypair.generate();

    const mintKey: anchor.web3.Keypair = anchor.web3.Keypair.generate();
    const nftTokenAccount = await getAssociatedTokenAddress(mintKey.publicKey, assetOwner.publicKey);

    const signature = await program.provider.connection.requestAirdrop(
      assetOwner.publicKey,
      LAMPORTS_PER_SOL * 100
    );

    // await program.provider.connection.requestAirdrop(
    //   sol_treasury.publicKey,
    //   LAMPORTS_PER_SOL * 100
    // );

    await program.provider.connection.confirmTransaction(signature);

    console.log("====== AIRDROP SUCCESSFUL! ======");

    const lamports: number = await program.provider.connection.getMinimumBalanceForRentExemption(MINT_SIZE);

    const mint_tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: assetOwner.publicKey,
        newAccountPubkey: mintKey.publicKey,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
        lamports,
      }),
      createInitializeMintInstruction(mintKey.publicKey, 0, assetOwner.publicKey, assetOwner.publicKey, TOKEN_PROGRAM_ID),
      createAssociatedTokenAccountInstruction(assetOwner.publicKey, nftTokenAccount, assetOwner.publicKey, mintKey.publicKey)
    );

    console.log("===== Start initializing Mint and token account ====== ");

    await program.provider.sendAndConfirm(mint_tx, [assetOwner, mintKey]);

    await program.methods.setupPlatform("LANDLORD", new anchor.BN(anchor.web3.LAMPORTS_PER_SOL), [
      fst_authority.publicKey,
      sec_authority.publicKey,
      program.provider.publicKey
    ])
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
      ])
      .rpc({
        preflightCommitment: "processed",
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
      mintKey.publicKey
    );

    await program.methods.issueAsset("https://basc.s3.amazonaws.com/meta/3506.json", "Bored Apes").accounts(
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
        tokenAccount: nftTokenAccount,
        owner: assetOwner.publicKey,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        assetBasket: assetBasketAddress
      }
    ).signers([
      assetOwner
    ]).rpc({
      commitment: "confirmed"
    });

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
  });
});
