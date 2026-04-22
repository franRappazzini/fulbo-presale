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

describe("fulbo pre-sale", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.fulbo_presale as Program<FulboPresale>;

  let mint: anchor.web3.PublicKey = new anchor.web3.PublicKey(
    "JCeMvfJ8pTg7gUqEMJWKYbzBWTfVDnMv9WdRY4qQBAN6",
  );

  before(async () => {
    let [config] = await findConfigPda();
    // return;

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

  it("buy_token ix!", async () => {
    // FIXME
    // const amount = bn(1_000_000_000); // 1000 $FULBO
    const amount = bn(
      stagesWithoutLimit.reduce((acc, stage) => acc + stage.maxTokens.toNumber(), 0),
    ); // buy all tokens available in the sale

    console.log("amount:", amount.toNumber());

    const [config] = await findConfigPda();
    const [treasury] = await findTreasuryPda();
    const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });

    const treasuryDataPre = await connection.getAccountInfo(
      new anchor.web3.PublicKey(treasury.toString()),
    );

    const positionDataPre = await connection.getAccountInfo(
      new anchor.web3.PublicKey(position.toString()),
    );

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

    const treasuryAccountPre = treasuryDataPre?.data
      ? getTreasuryDecoder().decode(treasuryDataPre!.data)
      : null;
    const treasuryAccount = getTreasuryDecoder().decode(treasuryData!.data);

    const positionData = await connection.getAccountInfo(
      new anchor.web3.PublicKey(position.toString()),
    );

    const positionAccountPre = getPositionDecoder().decode(positionDataPre!.data);
    const positionAccount = getPositionDecoder().decode(positionData!.data);

    console.log({ treasuryAccount });
    console.log({ positionAccount });

    expect(Number(treasuryAccount.totalSol)).greaterThan(Number(treasuryAccountPre?.totalSol || 0));

    expect(Number(positionAccount.totalTokens)).to.eq(
      Number(positionAccountPre.totalTokens) + amount.toNumber(),
    );
    expect(Number(positionAccount.totalSol)).to.eq(Number(treasuryAccount.totalSol));
    // expect(Number(positionAccount.stageAllocations[0].tokens)).to.eq(amount.toNumber());
  });

  it("claim_tokens ix!", async () => {
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

    // validate accounts
    const configData = await connection.getAccountInfo(
      new anchor.web3.PublicKey(config.toString()),
    );

    const configAccount = getConfigDecoder().decode(configData!.data);
    console.log({ configAccount });

    const positionData = await connection.getAccountInfo(
      new anchor.web3.PublicKey(position.toString()),
    );

    const positionAccount = getPositionDecoder().decode(positionData!.data);
    // TODO: ver como checkear las stage_allocations post claim (tiene que quedar el % locked y revisar todos los demas campos)
    console.log(
      JSON.stringify(
        positionAccount,
        (key, value) => (typeof value === "bigint" ? value.toString() : value),
        2,
      ),
    );
  });
});
