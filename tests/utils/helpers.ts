import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { address } from "@solana/kit";

import {
  findConfigPda,
  findPositionPda,
  findTreasuryPda,
  getConfigDecoder,
} from "../../clients/js/src/generated";
import { FulboPresale } from "../../target/types/fulbo_presale";
import { stagesWithoutLimit } from "../data";
import { constants } from "./constants";

export async function getConfigAccount(connection: anchor.web3.Connection) {
  const [config] = await findConfigPda();
  const data = (await connection.getAccountInfo(new anchor.web3.PublicKey(config.toString())))!
    .data;
  return getConfigDecoder().decode(data);
}

export async function buyTokenForStage(
  program: Program<FulboPresale>,
  wallet: anchor.Wallet,
  stageIndex: number,
): Promise<string> {
  const maxTokens = stagesWithoutLimit[stageIndex].maxTokens;

  const [config] = await findConfigPda();
  const [treasury] = await findTreasuryPda();
  const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });

  return program.methods
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
}
