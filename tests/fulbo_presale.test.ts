import * as anchor from "@anchor-lang/core";

import { findConfigPda, findTreasuryPda } from "../clients/js/src/generated";

import { FulboPresale } from "../target/types/fulbo_presale";
import { Program } from "@anchor-lang/core";
import { bn } from "./utils/functions";
import { constants } from "./utils/constants";
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
  });

  it("buy_token ix!", async () => {
    const amount = bn(1_000_000_000); // 1000 $FULBO

    const [config] = await findConfigPda();
    const [treasury] = await findTreasuryPda();

    const tx = await program.methods
      .buyToken(amount)
      .accountsStrict({
        buyer: wallet.publicKey,
        config,
        treasury,
        chainlinkFeed: constants.CHAINLINK_SOL_USD_FEED_DEVNET,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("buy_token signature:", tx);
  });
});
