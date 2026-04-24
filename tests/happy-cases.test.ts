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

import { FulboPresale } from "../target/types/fulbo_presale";
import { Program } from "@anchor-lang/core";
import { address } from "@solana/kit";
import { bn } from "./utils/functions";
import { constants } from "./utils/constants";
import { expect } from "chai";
import { stages } from "./data";

describe("fulbo pre-sale", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.fulbo_presale as Program<FulboPresale>;

  let mint: anchor.web3.PublicKey;

  before(async () => {
    const [config] = await findConfigPda();

    mint = await createMint(
      connection,
      wallet.payer as anchor.web3.Signer,
      new anchor.web3.PublicKey(config),
      null,
      6,
    );

    console.log("mint address:", mint);
  });

  it("initialize ix!", async () => {
    const [config] = await findConfigPda();
    const [treasury] = await findTreasuryPda();

    const totalTokensForSale = stages.reduce((acc, stage) => acc + stage.maxTokens.toNumber(), 0);

    const tx = await program.methods
      .initialize(bn(totalTokensForSale), stages)
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

  it("buy_token ix!", async () => {
    const amount = bn(1_000_000_000); // 1000 $FULBO

    const [config] = await findConfigPda();
    const [treasury] = await findTreasuryPda();
    const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });

    const tx = await program.methods
      .buyToken(amount)
      .accountsStrict({
        buyer: wallet.publicKey,
        config,
        treasury,
        position,
        chainlinkFeed: constants.CHAINLINK_SOL_USD_FEED_DEVNET,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("buy_token signature:", tx);

    // validate accounts
    const treasuryData = await connection.getAccountInfo(
      new anchor.web3.PublicKey(treasury.toString()),
    );

    const treasuryAccount = getTreasuryDecoder().decode(treasuryData!.data);
    expect(Number(treasuryAccount.totalSol)).greaterThan(0);

    const positionData = await connection.getAccountInfo(
      new anchor.web3.PublicKey(position.toString()),
    );

    const positionAccount = getPositionDecoder().decode(positionData!.data);
    expect(Number(positionAccount.totalTokens)).to.eq(amount.toNumber());
    expect(Number(positionAccount.totalSol)).to.eq(Number(treasuryAccount.totalSol));
    expect(Number(positionAccount.stageAllocations[0].tokens)).to.eq(amount.toNumber());
  });

  it("claim_token ix!", async () => {
    const [config] = await findConfigPda();
    const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });

    const claimerAta = getAssociatedTokenAddressSync(mint, wallet.publicKey);

    const tx = await program.methods
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
      .rpc();

    console.log("claim_token signature:", tx);
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
  });
});
