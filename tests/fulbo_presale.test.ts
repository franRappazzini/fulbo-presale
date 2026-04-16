import * as anchor from "@anchor-lang/core";

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
import { publicKey } from "@anchor-lang/core/dist/cjs/utils";
import { stages } from "./data";

describe("fulbo pre-sale", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.fulbo_presale as Program<FulboPresale>;

  it.skip("initialize ix!", async () => {
    const [config] = await findConfigPda();
    const [treasury] = await findTreasuryPda();

    const tx = await program.methods
      .initialize(stages)
      .accountsStrict({
        authority: wallet.publicKey,
        mint: anchor.web3.Keypair.generate().publicKey, // FIXME: crear mint real
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

  it.skip("buy_token ix!", async () => {
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
    expect(treasuryAccount.totalSol).to.eq(amount.toNumber());

    const positionData = await connection.getAccountInfo(
      new anchor.web3.PublicKey(position.toString()),
    );

    const positionAccount = getPositionDecoder().decode(positionData!.data);
    expect(positionAccount.totalTokens).to.eq(amount.toNumber());
  });
});
