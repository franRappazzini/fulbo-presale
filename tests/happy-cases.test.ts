import * as anchor from "@anchor-lang/core";

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
  mintTo,
} from "@solana/spl-token";
import { bn, logJson, sleep } from "./utils/functions";
import { buyTokenForStage, getConfigAccount } from "./utils/helpers";
import {
  findBeneficiaryAllocationPda,
  findConfigPda,
  findPositionPda,
  findTreasuryAtaPda,
  findTreasuryPda,
  findTreasurySharePda,
  getBeneficiaryAllocationDecoder,
  getPositionDecoder,
  getTreasuryShareDecoder,
} from "../clients/js/src/generated";

import { FulboPresale } from "../target/types/fulbo_presale";
import { Program } from "@anchor-lang/core";
import { address } from "@solana/kit";
import { constants } from "./utils/constants";
import { expect } from "chai";
import { stagesWithoutLimit } from "./data";

// ─── Tests ───────────────────────────────────────────────────────────────────

describe("fulbo pre-sale", () => {
  const provider = anchor.AnchorProvider.env();
  const { connection, wallet } = provider;

  anchor.setProvider(provider);

  const program = anchor.workspace.fulbo_presale as Program<FulboPresale>;

  let mint: anchor.web3.PublicKey;

  // 5 beneficiaries
  const teamKeypair = anchor.web3.Keypair.generate();
  const marketingKeypair = anchor.web3.Keypair.generate();
  const developmentKeypair = anchor.web3.Keypair.generate();
  const liquidityKeypair = anchor.web3.Keypair.generate();
  const rewardsKeypair = anchor.web3.Keypair.generate();

  // ─── Setup global ──────────────────────────────────────────────────────────
  before(async () => {
    // 1 SOL airdrop for each beneficiary
    const keypairs = [
      teamKeypair,
      marketingKeypair,
      developmentKeypair,
      liquidityKeypair,
      rewardsKeypair,
    ];
    await Promise.all(
      keypairs.map((kp) => connection.requestAirdrop(kp.publicKey, anchor.web3.LAMPORTS_PER_SOL)),
    );
    await sleep(2000);

    // TODO: hacer token burnable y metadata
    mint = await createMint(
      connection,
      wallet.payer as anchor.web3.Signer,
      wallet.publicKey,
      null,
      6,
    );
    console.log("mint:", mint.toString());
  });

  // ─── 1. Initialize ──────────────────────────────────────────────────────────
  describe("1. initialize", () => {
    it("should initialize config, treasury and treasury_ata", async () => {
      const [config] = await findConfigPda();
      const [treasury] = await findTreasuryPda();
      const [treasuryAta] = await findTreasuryAtaPda({ mint: address(mint.toBase58()) });

      const totalTokensForSale = stagesWithoutLimit.reduce(
        (acc, stage) => acc + BigInt(stage.maxTokens.toString()),
        0n,
      );

      const tx = await program.methods
        .initialize(bn(totalTokensForSale), stagesWithoutLimit)
        .accountsStrict({
          authority: wallet.publicKey,
          mint,
          config,
          treasury,
          treasuryAta,
          chainlinkFeed: constants.CHAINLINK_SOL_USD_FEED_DEVNET,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc({ commitment: "confirmed" });

      console.log("initialize:", tx);

      // Depositar todos los tokens al treasury_ata:
      // presale (300M) + beneficiarios (700M) = 1B tokens = 1_000_000_000 * 10^6 units
      await mintTo(
        connection,
        wallet.payer as anchor.web3.Signer,
        mint,
        new anchor.web3.PublicKey(treasuryAta.toString()),
        wallet.payer as anchor.web3.Signer,
        1_000_000_000_000_000n,
      );
      console.log("mintTo treasury_ata: 1B tokens");

      const configAccount = await getConfigAccount(connection);
      expect(configAccount.authority.toString()).eq(wallet.publicKey.toString());
      expect(configAccount.mint.toString()).eq(mint.toString());
      expect(configAccount.tgeTimestamp).eq(0n);
      expect(configAccount.saleFinalized).eq(false);
      expect(configAccount.paused).eq(false);
    });
  });

  // ─── 2. Initialize Beneficiary ─────────────────────────────────────────────
  describe("2. initialize_beneficiary", () => {
    const beneficiaries = [
      {
        label: "Team",
        keypair: null as anchor.web3.Keypair | null,
        totalTokens: bn(150_000_000_000_000), // 150M
        tgeUnlockBps: 500,
        instantUnlock: false,
        withdrawInterval: bn(1), // 1s para tests
        solShareBps: 2000, // 20%
      },
      {
        label: "Marketing",
        keypair: null,
        totalTokens: bn(200_000_000_000_000), // 200M
        tgeUnlockBps: 2000,
        instantUnlock: false,
        withdrawInterval: bn(1),
        solShareBps: 2500, // 25%
      },
      {
        label: "Development",
        keypair: null,
        totalTokens: bn(50_000_000_000_000), // 50M
        tgeUnlockBps: 500,
        instantUnlock: false,
        withdrawInterval: bn(1),
        solShareBps: 3500, // 35%
      },
      {
        label: "Liquidity",
        keypair: null,
        totalTokens: bn(100_000_000_000_000), // 100M
        tgeUnlockBps: 0,
        instantUnlock: true,
        withdrawInterval: bn(0), // no usado (instant_unlock=true)
        solShareBps: 2000, // 20%
        // total bps: 2000+2500+3500+2000 = 10_000 (100%) ✓
      },
    ];

    before(() => {
      beneficiaries[0].keypair = teamKeypair;
      beneficiaries[1].keypair = marketingKeypair;
      beneficiaries[2].keypair = developmentKeypair;
      beneficiaries[3].keypair = liquidityKeypair;
    });

    for (let i = 0; i < 4; i++) {
      it(`should initialize ${
        ["Team", "Marketing", "Development", "Liquidity"][i]
      } beneficiary`, async () => {
        const b = beneficiaries[i];
        const kp = b.keypair!;

        const [config] = await findConfigPda();
        const [beneficiaryAllocation] = await findBeneficiaryAllocationPda({
          beneficiary: address(kp.publicKey.toBase58()),
        });
        const [treasuryShare] = await findTreasurySharePda({
          beneficiary: address(kp.publicKey.toBase58()),
        });

        const tx = await program.methods
          .initializeBeneficiary(
            b.totalTokens,
            b.tgeUnlockBps,
            b.instantUnlock,
            b.withdrawInterval,
            b.solShareBps,
          )
          .accountsStrict({
            authority: wallet.publicKey,
            beneficiary: kp.publicKey,
            config,
            beneficiaryAllocation,
            treasuryShare,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc({ commitment: "confirmed" });

        console.log(`initialize_beneficiary (${b.label}):`, tx);

        const alloc = getBeneficiaryAllocationDecoder().decode(
          (await connection.getAccountInfo(
            new anchor.web3.PublicKey(beneficiaryAllocation.toString()),
          ))!.data,
        );

        expect(alloc.totalTokens.toString()).eq(b.totalTokens.toString());
        expect(alloc.instantUnlock).eq(b.instantUnlock);
      });
    }
  });

  // ─── 3. Initialize Rewards Beneficiary ────────────────────────────────────
  describe("3. initialize_rewards_beneficiary", () => {
    it("should initialize Rewards beneficiary (no SOL share, instant_unlock)", async () => {
      const [config] = await findConfigPda();
      const [beneficiaryAllocation] = await findBeneficiaryAllocationPda({
        beneficiary: address(rewardsKeypair.publicKey.toBase58()),
      });

      const totalTokens = bn(200_000_000_000_000); // 200M

      const tx = await program.methods
        .initializeRewardsBeneficiary(totalTokens)
        .accountsStrict({
          authority: wallet.publicKey,
          beneficiary: rewardsKeypair.publicKey,
          config,
          beneficiaryAllocation,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc({ commitment: "confirmed" });

      console.log("initialize_rewards_beneficiary:", tx);

      const alloc = getBeneficiaryAllocationDecoder().decode(
        (await connection.getAccountInfo(
          new anchor.web3.PublicKey(beneficiaryAllocation.toString()),
        ))!.data,
      );

      expect(alloc.totalTokens.toString()).eq(totalTokens.toString());
      expect(alloc.instantUnlock).eq(true);
    });
  });

  // ─── 4. Pause (toggle) ─────────────────────────────────────────────────────
  describe("4. pause", () => {
    it("should pause the sale", async () => {
      const [config] = await findConfigPda();

      const tx = await program.methods
        .pause()
        .accountsStrict({ authority: wallet.publicKey, config })
        .rpc({ commitment: "confirmed" });

      console.log("pause:", tx);

      const configAccount = await getConfigAccount(connection);
      expect(configAccount.paused).eq(true);
    });

    it("should unpause the sale (toggle)", async () => {
      const [config] = await findConfigPda();

      const tx = await program.methods
        .pause()
        .accountsStrict({ authority: wallet.publicKey, config })
        .rpc({ commitment: "confirmed" });

      console.log("unpause:", tx);

      const configAccount = await getConfigAccount(connection);
      expect(configAccount.paused).eq(false);
    });
  });

  // ─── 5. Buy Token ──────────────────────────────────────────────────────────
  describe("5. buy_token", () => {
    // Compra parcial: 1M tokens de stage 0.
    // Mantiene sale_finalized=false para poder testear announce_tge manualmente.
    it("should buy tokens (partial — 1M from stage 0)", async () => {
      const [config] = await findConfigPda();
      const [treasury] = await findTreasuryPda();
      const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });

      // 1_000_000 tokens * 10^6 decimales = 1_000_000_000_000 units
      const amount = bn(1_000_000_000_000);

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
        .rpc({ commitment: "confirmed" });

      console.log("buy_token (partial):", tx);

      const posAccount = getPositionDecoder().decode(
        (await connection.getAccountInfo(new anchor.web3.PublicKey(position.toString())))!.data,
      );

      expect(posAccount.stageAllocations[0].tokens.toString()).eq(amount.toString());
      expect(posAccount.totalTokens.toString()).eq(amount.toString());

      const configAccount = await getConfigAccount(connection);
      expect(configAccount.saleFinalized).eq(false);
      expect(configAccount.currentStage).eq(0);
    });

    // Compra total: agota todos los stages, dispara sale_finalized=true automáticamente.
    // SKIP por defecto — desactivar para probar el flujo alternativo sin announce_tge.
    it.skip("should buy tokens (all stages — exhausts presale)", async () => {
      for (let i = 0; i < stagesWithoutLimit.length; i++) {
        console.log(
          `\n--- Stage ${i} | maxTokens: ${stagesWithoutLimit[i].maxTokens.toString()} ---`,
        );
        const tx = await buyTokenForStage(program, wallet as anchor.Wallet, i);
        console.log(`buy_token stage ${i}:`, tx);
      }

      const configAccount = await getConfigAccount(connection);
      expect(configAccount.saleFinalized).eq(true);
    });
  });

  // ─── 6. Announce TGE ───────────────────────────────────────────────────────
  describe("6. announce_tge", () => {
    it("should announce TGE and wait for it to pass", async () => {
      const [config] = await findConfigPda();

      const configBefore = await getConfigAccount(connection);
      expect(configBefore.saleFinalized).eq(false);
      expect(configBefore.tgeTimestamp).eq(0n);

      const tx = await program.methods
        .announceTge()
        .accountsStrict({ authority: wallet.publicKey, config })
        .rpc({ commitment: "confirmed" });

      console.log("announce_tge:", tx);

      const configAccount = await getConfigAccount(connection);
      expect(configAccount.tgeTimestamp > 0n).eq(true);

      // SECONDS_PER_MONTH = 2s en tests — esperar 3s para que el TGE ya haya pasado
      console.log("  Esperando que pase el TGE (3s)...");
      await sleep(3000);
    });
  });

  // ─── 7. Claim Token ────────────────────────────────────────────────────────
  describe("7. claim_token", () => {
    it("should claim unlocked tokens (TGE unlock — no months elapsed)", async () => {
      const [config] = await findConfigPda();
      const [position] = await findPositionPda({ buyer: address(wallet.publicKey.toBase58()) });
      const [treasuryAta] = await findTreasuryAtaPda({ mint: address(mint.toBase58()) });
      const claimerAta = getAssociatedTokenAddressSync(mint, wallet.publicKey);

      const tx = await program.methods
        .claimToken()
        .accountsStrict({
          claimer: wallet.publicKey,
          config,
          position,
          mint,
          treasuryAta,
          claimerAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc({ commitment: "confirmed" });

      console.log("claim_token:", tx);

      const posAccount = getPositionDecoder().decode(
        (await connection.getAccountInfo(new anchor.web3.PublicKey(position.toString())))!.data,
      );

      // Stage 0: lockedPctBps=5000 → unlock at TGE = 50% de 1M = 500_000 tokens
      const expectedClaimed = (1_000_000_000_000n * 5000n) / 10_000n;
      expect(posAccount.tokensClaimed.toString()).eq(expectedClaimed.toString());

      const ataBalance = await connection.getTokenAccountBalance(claimerAta);
      expect(Number(ataBalance.value.amount)).greaterThan(0);
    });
  });

  // ─── 8. Beneficiary Claim ─────────────────────────────────────────────────
  describe("8. beneficiary_claim", () => {
    const claims = [
      {
        label: "Liquidity",
        keypair: null as anchor.web3.Keypair | null,
        expectedMin: 100_000_000_000_000n, // 100M — instant_unlock total
      },
      {
        label: "Rewards",
        keypair: null,
        expectedMin: 200_000_000_000_000n, // 200M — instant_unlock total
      },
      {
        label: "Team",
        keypair: null,
        expectedMin: 7_500_000_000_000n, // 150M * 5% TGE unlock
      },
      {
        label: "Marketing",
        keypair: null,
        expectedMin: 40_000_000_000_000n, // 200M * 20% TGE unlock
      },
      {
        label: "Development",
        keypair: null,
        expectedMin: 2_500_000_000_000n, // 50M * 5% TGE unlock
      },
    ];

    before(() => {
      claims[0].keypair = liquidityKeypair;
      claims[1].keypair = rewardsKeypair;
      claims[2].keypair = teamKeypair;
      claims[3].keypair = marketingKeypair;
      claims[4].keypair = developmentKeypair;
    });

    for (let i = 0; i < claims.length; i++) {
      it(`should claim tokens for ${
        ["Liquidity", "Rewards", "Team", "Marketing", "Development"][i]
      }`, async () => {
        const c = claims[i];
        const kp = c.keypair!;

        const [config] = await findConfigPda();
        const [beneficiaryAllocation] = await findBeneficiaryAllocationPda({
          beneficiary: address(kp.publicKey.toBase58()),
        });
        const [treasuryAta] = await findTreasuryAtaPda({ mint: address(mint.toBase58()) });
        const beneficiaryAta = getAssociatedTokenAddressSync(mint, kp.publicKey);

        const tx = await program.methods
          .beneficiaryClaim()
          .accountsStrict({
            beneficiary: kp.publicKey,
            config,
            beneficiaryAllocation,
            mint,
            treasuryAta,
            beneficiaryAta,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([kp])
          .rpc({ commitment: "confirmed" });

        console.log(`beneficiary_claim (${c.label}):`, tx);

        const alloc = getBeneficiaryAllocationDecoder().decode(
          (await connection.getAccountInfo(
            new anchor.web3.PublicKey(beneficiaryAllocation.toString()),
          ))!.data,
        );

        expect(alloc.withdrawnTokens >= c.expectedMin).eq(true);

        const ataBalance = await connection.getTokenAccountBalance(beneficiaryAta);
        expect(Number(ataBalance.value.amount)).greaterThan(0);
      });
    }
  });

  // ─── 9. Withdraw Treasury ──────────────────────────────────────────────────
  describe("9. withdraw_treasury", () => {
    // Rewards no tiene TreasuryShare (initialize_rewards_beneficiary no lo crea)
    const withdrawals = [
      { label: "Liquidity", keypair: null as anchor.web3.Keypair | null },
      { label: "Team", keypair: null },
      { label: "Marketing", keypair: null },
      { label: "Development", keypair: null },
    ];

    before(() => {
      withdrawals[0].keypair = liquidityKeypair;
      withdrawals[1].keypair = teamKeypair;
      withdrawals[2].keypair = marketingKeypair;
      withdrawals[3].keypair = developmentKeypair;
    });

    for (let i = 0; i < withdrawals.length; i++) {
      it(`should withdraw SOL for ${
        ["Liquidity", "Team", "Marketing", "Development"][i]
      }`, async () => {
        const w = withdrawals[i];
        const kp = w.keypair!;

        const [config] = await findConfigPda();
        const [treasury] = await findTreasuryPda();
        const [beneficiaryTreasury] = await findTreasurySharePda({
          beneficiary: address(kp.publicKey.toBase58()),
        });

        const balanceBefore = await connection.getBalance(kp.publicKey);

        const tx = await program.methods
          .withdrawTreasury()
          .accountsStrict({
            beneficiary: kp.publicKey,
            beneficiaryTreasury,
            treasury,
            config,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([kp])
          .rpc({ commitment: "confirmed" });

        console.log(`withdraw_treasury (${w.label}):`, tx);

        const share = getTreasuryShareDecoder().decode(
          (await connection.getAccountInfo(
            new anchor.web3.PublicKey(beneficiaryTreasury.toString()),
          ))!.data,
        );

        expect(share.solWithdrawn > 0n).eq(true);
      });
    }
  });

  // ─── 10. Finalize Unsold ───────────────────────────────────────────────────
  describe("10. finalize_unsold", () => {
    it("should burn 60% and route 40% to rewards beneficiary", async () => {
      const [config] = await findConfigPda();
      const [treasuryAta] = await findTreasuryAtaPda({ mint: address(mint.toBase58()) });
      const [rewardsBeneficiaryAllocation] = await findBeneficiaryAllocationPda({
        beneficiary: address(rewardsKeypair.publicKey.toBase58()),
      });

      const rewardsAllocBefore = getBeneficiaryAllocationDecoder().decode(
        (await connection.getAccountInfo(
          new anchor.web3.PublicKey(rewardsBeneficiaryAllocation.toString()),
        ))!.data,
      );
      console.log("  rewardsTotalTokensBefore:", rewardsAllocBefore.totalTokens.toString());

      const tx = await program.methods
        .finalizeUnsold()
        .accountsStrict({
          authority: wallet.publicKey,
          rewardsBeneficiary: rewardsKeypair.publicKey,
          config,
          beneficiaryAllocation: rewardsBeneficiaryAllocation,
          mint,
          treasuryAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc({ commitment: "confirmed" });

      console.log("finalize_unsold:", tx);

      const configAccount = await getConfigAccount(connection);
      expect(configAccount.unsoldFinalized).eq(true);

      const rewardsAllocAfter = getBeneficiaryAllocationDecoder().decode(
        (await connection.getAccountInfo(
          new anchor.web3.PublicKey(rewardsBeneficiaryAllocation.toString()),
        ))!.data,
      );

      // 40% del unsold debe haberse sumado al total de rewards
      expect(rewardsAllocAfter.totalTokens > rewardsAllocBefore.totalTokens).eq(true);
    });
  });
});
