import * as anchor from "@anchor-lang/core";

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  findConfigPda,
  findPositionPda,
  findTreasuryPda,
  getConfigDecoder,
  getPositionDecoder,
  getTreasuryDecoder,
} from "../clients/js/src/generated";
import { stages, stagesWithoutLimit } from "./data";

import { FulboPresale } from "../target/types/fulbo_presale";
import { Program } from "@anchor-lang/core";
import { address } from "@solana/kit";
import { bn } from "./utils/functions";
import { constants } from "./utils/constants";
import { expect } from "chai";

// ─── Helpers ────────────────────────────────────────────────────────────────

async function buyTokenForStage(
  program: Program<FulboPresale>,
  wallet: anchor.Wallet,
  stageIndex: number,
): Promise<string> {
  const maxTokens = stagesWithoutLimit[stageIndex].maxTokens;

  const [config] = await findConfigPda();
  const [treasury] = await findTreasuryPda();
  const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });

  const txSignature = await program.methods
    .buyToken(maxTokens)
    .accountsStrict({
      buyer: wallet.publicKey,
      config,
      treasury,
      position,
      chainlinkFeed: constants.CHAINLINK_SOL_USD_FEED_DEVNET,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc({ commitment: "confirmed" });

  // get config account and log current stage info
  const configAccount = getConfigDecoder().decode(
    (await program.provider.connection.getAccountInfo(
      new anchor.web3.PublicKey(config.toString()),
    ))!.data,
  );
  const currentStageInfo = configAccount.stages[configAccount.currentStage];
  console.log(`Current stage info:`, {
    ...currentStageInfo,
    maxTokens: currentStageInfo.maxTokens.toString(),
  });

  return txSignature;
}

async function claimTokens(
  program: Program<FulboPresale>,
  wallet: anchor.Wallet,
  mint: anchor.web3.PublicKey,
): Promise<string> {
  const [config] = await findConfigPda();
  const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });
  const claimerAta = getAssociatedTokenAddressSync(mint, wallet.publicKey);

  return program.methods
    .claimToken()
    .accountsStrict({
      claimer: wallet.publicKey,
      config,
      position,
      mint,
      claimerAta,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc({ commitment: "confirmed" });
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe("fulbo pre-sale", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.fulbo_presale as Program<FulboPresale>;

  let mint: anchor.web3.PublicKey = new anchor.web3.PublicKey(
    "6mpuQU4XkaGLrrggJ8swKWj18MH1bBcoEKHVthBoTJG6",
  );

  before(async () => {
    const [config] = await findConfigPda();

    mint = await createMint(
      connection,
      wallet.payer as anchor.web3.Signer,
      new anchor.web3.PublicKey(config), // config PDA as mint authority
      null,
      6,
    );

    console.log("mint address:", mint.toString());
  });

  it("initialize ix!", async () => {
    const [config] = await findConfigPda();
    const [treasury] = await findTreasuryPda();

    const totalTokensForSale = stagesWithoutLimit.reduce(
      (acc, stage) => acc + stage.maxTokens.toNumber(),
      0,
    );

    const tx = await program.methods
      // FIXME: replace with stages (with limits)
      .initialize(bn(totalTokensForSale), stagesWithoutLimit)
      .accountsStrict({
        authority: wallet.publicKey,
        mint,
        config,
        treasury,
        chainlinkFeed: constants.CHAINLINK_SOL_USD_FEED_DEVNET,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("initialize signature:", tx);

    const configData = await connection.getAccountInfo(
      new anchor.web3.PublicKey(config.toString()),
    );

    const configAccount = getConfigDecoder().decode(configData!.data);
    expect(configAccount.authority.toString()).eq(wallet.publicKey.toString());
    expect(configAccount.chainlinkFeed.toString()).eq(
      constants.CHAINLINK_SOL_USD_FEED_DEVNET.toString(),
    );
  });

  it.skip("pause ix!", async () => {
    const [config] = await findConfigPda();

    const tx = await program.methods
      .pause()
      .accountsStrict({
        authority: wallet.publicKey,
        config,
      })
      .rpc();
    console.log("pause signature:", tx);

    // Print config state after pause
    const configAccount = getConfigDecoder().decode(
      (await connection.getAccountInfo(new anchor.web3.PublicKey(config.toString())))!.data,
    );
    console.log(
      "config after pause:",
      JSON.stringify(configAccount, (_, v) => (typeof v === "bigint" ? v.toString() : v), 2),
    );
  });

  it("buy_token ix! (all stages)", async () => {
    for (let i = 0; i < stagesWithoutLimit.length; i++) {
      // if (i > 1) break; // just buy first 2 stages for testing
      const maxTokens = stagesWithoutLimit[i].maxTokens;
      console.log(`\n--- Stage ${i} | maxTokens: ${maxTokens.toString()} ---`);

      const tx = await buyTokenForStage(program, wallet as anchor.Wallet, i);
      console.log(`buy_token stage ${i} signature:`, tx);
      setTimeout(() => {}, 500);
    }

    // Print final on-chain state after all stages are bought
    const [config] = await findConfigPda();
    const [treasury] = await findTreasuryPda();
    const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });

    const configAccount = getConfigDecoder().decode(
      (await connection.getAccountInfo(new anchor.web3.PublicKey(config.toString())))!.data,
    );
    const treasuryAccount = getTreasuryDecoder().decode(
      (await connection.getAccountInfo(new anchor.web3.PublicKey(treasury.toString())))!.data,
    );
    const positionAccount = getPositionDecoder().decode(
      (await connection.getAccountInfo(new anchor.web3.PublicKey(position.toString())))!.data,
    );

    console.log(
      "\nconfig:",
      JSON.stringify(configAccount, (_, v) => (typeof v === "bigint" ? v.toString() : v), 2),
    );
    console.log("---------------------------");
    console.log("treasury:", { treasuryAccount });
    console.log("---------------------------");
    console.log(
      "position:",
      JSON.stringify(positionAccount, (_, v) => (typeof v === "bigint" ? v.toString() : v), 2),
    );
  });

  it("announce_tge ix!", async () => {
    const [config] = await findConfigPda();

    const tx = await program.methods
      .announceTge()
      .accountsStrict({
        authority: wallet.publicKey,
        config,
      })
      .rpc();
    console.log("announce_tge signature:", tx);

    // Print final on-chain state after announce_tge
    const configAccount = getConfigDecoder().decode(
      (await connection.getAccountInfo(new anchor.web3.PublicKey(config.toString())))!.data,
    );
    console.log(
      "config after announce_tge:",
      JSON.stringify(configAccount, (_, v) => (typeof v === "bigint" ? v.toString() : v), 2),
    );
  });

  // 61,81972975 %
  it("claim_tokens ix!", async () => {
    const tx = await claimTokens(program, wallet as anchor.Wallet, mint);
    console.log("claim_token signature:", tx);

    // Print final on-chain state after claim
    const [config] = await findConfigPda();
    const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });

    const configAccount = getConfigDecoder().decode(
      (await connection.getAccountInfo(new anchor.web3.PublicKey(config.toString())))!.data,
    );
    const positionAccount = getPositionDecoder().decode(
      (await connection.getAccountInfo(new anchor.web3.PublicKey(position.toString())))!.data,
    );

    console.log({ configAccount });
    // TODO: ver como checkear las stage_allocations post claim (tiene que quedar el % locked y revisar todos los demas campos)
    console.log(
      JSON.stringify(positionAccount, (_, v) => (typeof v === "bigint" ? v.toString() : v), 2),
    );

    // total claimed and total claimable tokens check
    const totalClaimed = positionAccount.stageAllocations.reduce(
      (acc, alloc) => acc + Number(alloc.claimed),
      0,
    );
    const totalBuyed = positionAccount.stageAllocations.reduce(
      (acc, alloc) => acc + Number(alloc.tokens),
      0,
    );

    console.log("total claimable tokens:", totalBuyed);
    console.log("total claimed tokens:", totalClaimed);
  });
});
