import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ProgressiveJackpot } from "../target/types/progressive_jackpot";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";
import { BN } from "@coral-xyz/anchor";

describe("progressive-jackpot", () => {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ProgressiveJackpot as Program<ProgressiveJackpot>;
  
  // Test accounts
  const authority = provider.wallet;
  const player1 = Keypair.generate();
  const player2 = Keypair.generate();
  const houseVault = Keypair.generate();
  
  // PDAs
  let configPda: PublicKey;
  let poolPda: PublicKey;
  let rewardVaultPda: PublicKey;
  let configBump: number;
  let poolBump: number;
  let rewardVaultBump: number;

  // Test parameters
  const jackpotPercentage = 500; // 5%
  const housePercentage = 200; // 2%
  const defiPercentage = 100; // 1%
  const minBet = new BN(0.1 * LAMPORTS_PER_SOL);
  const maxBet = new BN(10 * LAMPORTS_PER_SOL);
  const winProbabilityBps = 100; // 1% = 1/100
  const resetThreshold = new BN(100 * LAMPORTS_PER_SOL);
  const milestoneBets = new BN(1000);
  const apyBps = 500; // 5% APY

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(
      player1.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      player2.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      houseVault.publicKey,
      1 * LAMPORTS_PER_SOL
    );

    // Wait for airdrops to confirm
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Derive PDAs
    [configPda, configBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
    [poolPda, poolBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      program.programId
    );
    [rewardVaultPda, rewardVaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_vault")],
      program.programId
    );
  });

  describe("Initialization", () => {
    it("Initializes the casino system", async () => {
      const tx = await program.methods
        .initialize(
          jackpotPercentage,
          housePercentage,
          defiPercentage,
          minBet,
          maxBet,
          winProbabilityBps,
          0, // ORAO VRF
          null, // orao_network
          null, // switchboard_queue
          resetThreshold,
          milestoneBets,
          apyBps
        )
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Verify config
      const config = await program.account.config.fetch(configPda);
      expect(config.authority.toString()).to.equal(authority.publicKey.toString());
      expect(config.jackpotPercentage).to.equal(jackpotPercentage);
      expect(config.housePercentage).to.equal(housePercentage);
      expect(config.defiPercentage).to.equal(defiPercentage);
      expect(config.minBet.toString()).to.equal(minBet.toString());
      expect(config.maxBet.toString()).to.equal(maxBet.toString());
      expect(config.winProbabilityBps).to.equal(winProbabilityBps);
      expect(config.totalBets.toString()).to.equal("0");
      expect(config.totalWins.toString()).to.equal("0");

      // Verify pool
      const pool = await program.account.jackpotPool.fetch(poolPda);
      expect(pool.balance.toString()).to.equal("0");
      expect(pool.resetThreshold.toString()).to.equal(resetThreshold.toString());
      expect(pool.milestoneBets.toString()).to.equal(milestoneBets.toString());
      expect(pool.betsSinceWin.toString()).to.equal("0");
    });

    it("Fails to initialize with invalid percentages", async () => {
      const invalidConfigPda = Keypair.generate();
      
      try {
        await program.methods
          .initialize(
            10000, // 100% jackpot
            1000,  // 10% house (exceeds 100% total)
            1000,  // 10% defi
            minBet,
            maxBet,
            winProbabilityBps,
            0,
            null,
            null,
            resetThreshold,
            milestoneBets,
            apyBps
          )
          .accounts({
            config: invalidConfigPda.publicKey,
            pool: poolPda,
            rewardVault: rewardVaultPda,
            authority: authority.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        
        expect.fail("Should have failed with invalid config");
      } catch (err) {
        expect(err.toString()).to.include("InvalidConfig");
      }
    });
  });

  describe("Bet Contributions", () => {
    it("Player contributes a valid bet", async () => {
      const betAmount = new BN(1 * LAMPORTS_PER_SOL);
      const timestamp = new BN(Date.now() / 1000);
      
      const [betPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player1.publicKey.toBuffer(),
          timestamp.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const [vrfRequestPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda.toBuffer()],
        program.programId
      );

      const tx = await program.methods
        .contributeBet(betAmount)
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          bet: betPda,
          vrfRequest: vrfRequestPda,
          houseVault: houseVault.publicKey,
          player: player1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1])
        .rpc();

      // Verify bet was recorded
      const bet = await program.account.bet.fetch(betPda);
      expect(bet.player.toString()).to.equal(player1.publicKey.toString());
      expect(bet.amount.toString()).to.equal(betAmount.toString());
      expect(bet.status).to.equal(0); // pending

      // Verify pool increased
      const pool = await program.account.jackpotPool.fetch(poolPda);
      const expectedJackpot = betAmount.muln(jackpotPercentage).divn(10000);
      expect(pool.balance.toString()).to.equal(expectedJackpot.toString());
      expect(pool.betsSinceWin.toString()).to.equal("1");

      // Verify config updated
      const config = await program.account.config.fetch(configPda);
      expect(config.totalBets.toString()).to.equal("1");
    });

    it("Fails with bet below minimum", async () => {
      const smallBet = new BN(0.01 * LAMPORTS_PER_SOL); // Below min
      
      const [betPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player1.publicKey.toBuffer(),
          smallBet.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const [vrfRequestPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .contributeBet(smallBet)
          .accounts({
            config: configPda,
            pool: poolPda,
            rewardVault: rewardVaultPda,
            bet: betPda,
            vrfRequest: vrfRequestPda,
            houseVault: houseVault.publicKey,
            player: player1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([player1])
          .rpc();
        
        expect.fail("Should have failed with bet too small");
      } catch (err) {
        expect(err.toString()).to.include("BetTooSmall");
      }
    });

    it("Fails with bet above maximum", async () => {
      const largeBet = new BN(20 * LAMPORTS_PER_SOL); // Above max
      
      const [betPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player1.publicKey.toBuffer(),
          largeBet.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const [vrfRequestPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .contributeBet(largeBet)
          .accounts({
            config: configPda,
            pool: poolPda,
            rewardVault: rewardVaultPda,
            bet: betPda,
            vrfRequest: vrfRequestPda,
            houseVault: houseVault.publicKey,
            player: player1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([player1])
          .rpc();
        
        expect.fail("Should have failed with bet too large");
      } catch (err) {
        expect(err.toString()).to.include("BetTooLarge");
      }
    });

    it("Multiple players contribute bets", async () => {
      const betAmount1 = new BN(2 * LAMPORTS_PER_SOL);
      const betAmount2 = new BN(1.5 * LAMPORTS_PER_SOL);
      
      // Player 1 bet
      const [betPda1] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player1.publicKey.toBuffer(),
          betAmount1.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );
      const [vrfRequestPda1] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda1.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeBet(betAmount1)
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          bet: betPda1,
          vrfRequest: vrfRequestPda1,
          houseVault: houseVault.publicKey,
          player: player1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1])
        .rpc();

      // Player 2 bet
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const [betPda2] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player2.publicKey.toBuffer(),
          betAmount2.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );
      const [vrfRequestPda2] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda2.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeBet(betAmount2)
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          bet: betPda2,
          vrfRequest: vrfRequestPda2,
          houseVault: houseVault.publicKey,
          player: player2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player2])
        .rpc();

      const pool = await program.account.jackpotPool.fetch(poolPda);
      const config = await program.account.config.fetch(configPda);
      
      // Pool should have contributions from both bets
      expect(parseInt(pool.balance.toString())).to.be.greaterThan(0);
      expect(config.totalBets.toString()).to.equal("3"); // 1 from previous test + 2 new
    });
  });

  describe("Jackpot Fulfillment", () => {
    let betPda: PublicKey;
    let vrfRequestPda: PublicKey;
    let betAmount: BN;

    beforeEach(async () => {
      // Create a bet for fulfillment tests
      betAmount = new BN(1 * LAMPORTS_PER_SOL);
      
      [betPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player1.publicKey.toBuffer(),
          betAmount.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      [vrfRequestPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeBet(betAmount)
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          bet: betPda,
          vrfRequest: vrfRequestPda,
          houseVault: houseVault.publicKey,
          player: player1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1])
        .rpc();
    });

    it("Fulfills jackpot with winning VRF result", async () => {
      // Create a winning VRF result (value < win_probability_bps)
      const winningVrf = Buffer.alloc(32);
      winningVrf.writeUInt32LE(50, 0); // 50 < 100 (win_probability_bps)
      
      const playerBalanceBefore = await provider.connection.getBalance(player1.publicKey);
      const poolBefore = await program.account.jackpotPool.fetch(poolPda);
      const poolBalanceBefore = poolBefore.balance;

      await program.methods
        .fulfillJackpot(Array.from(winningVrf))
        .accounts({
          config: configPda,
          pool: poolPda,
          bet: betPda,
          vrfRequest: vrfRequestPda,
          player: player1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Verify bet status
      const bet = await program.account.bet.fetch(betPda);
      expect(bet.status).to.equal(1); // won
      expect(parseInt(bet.winAmount.toString())).to.be.greaterThan(0);

      // Verify pool decreased
      const poolAfter = await program.account.jackpotPool.fetch(poolPda);
      expect(parseInt(poolAfter.balance.toString())).to.be.lessThan(parseInt(poolBalanceBefore.toString()));

      // Verify player received winnings
      const playerBalanceAfter = await provider.connection.getBalance(player1.publicKey);
      expect(playerBalanceAfter).to.be.greaterThan(playerBalanceBefore);

      // Verify config updated
      const config = await program.account.config.fetch(configPda);
      expect(parseInt(config.totalWins.toString())).to.be.greaterThan(0);
    });

    it("Fulfills jackpot with losing VRF result", async () => {
      // Create a losing VRF result (value >= win_probability_bps)
      const losingVrf = Buffer.alloc(32);
      losingVrf.writeUInt32LE(500, 0); // 500 >= 100 (win_probability_bps)
      
      const poolBefore = await program.account.jackpotPool.fetch(poolPda);
      const poolBalanceBefore = poolBefore.balance;

      await program.methods
        .fulfillJackpot(Array.from(losingVrf))
        .accounts({
          config: configPda,
          pool: poolPda,
          bet: betPda,
          vrfRequest: vrfRequestPda,
          player: player1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Verify bet status
      const bet = await program.account.bet.fetch(betPda);
      expect(bet.status).to.equal(2); // lost
      expect(bet.winAmount.toString()).to.equal("0");

      // Verify pool unchanged (no win)
      const poolAfter = await program.account.jackpotPool.fetch(poolPda);
      expect(poolAfter.balance.toString()).to.equal(poolBalanceBefore.toString());
    });

    it("Fails to fulfill with invalid VRF request", async () => {
      const fakeVrfRequest = Keypair.generate();
      const vrfResult = Buffer.alloc(32);

      try {
        await program.methods
          .fulfillJackpot(Array.from(vrfResult))
          .accounts({
            config: configPda,
            pool: poolPda,
            bet: betPda,
            vrfRequest: fakeVrfRequest.publicKey,
            player: player1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        
        expect.fail("Should have failed with invalid VRF request");
      } catch (err) {
        expect(err.toString()).to.include("VrfRequestNotFound");
      }
    });
  });

  describe("DeFi Rewards", () => {
    it("Claims rewards from staked pool", async () => {
      // First, contribute a bet to add to reward vault
      const betAmount = new BN(1 * LAMPORTS_PER_SOL);
      
      const [betPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player1.publicKey.toBuffer(),
          betAmount.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const [vrfRequestPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeBet(betAmount)
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          bet: betPda,
          vrfRequest: vrfRequestPda,
          houseVault: houseVault.publicKey,
          player: player1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1])
        .rpc();

      // Wait for time to pass (simulate in test)
      await new Promise((resolve) => setTimeout(resolve, 2000));

      // Derive reward claim PDA
      const [rewardClaimPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("reward_claim"), player1.publicKey.toBuffer()],
        program.programId
      );

      const userBalanceBefore = await provider.connection.getBalance(player1.publicKey);

      // Note: In a real scenario, rewards would accumulate over time
      // This test verifies the claim mechanism works
      try {
        await program.methods
          .claimRewards()
          .accounts({
            config: configPda,
            rewardVault: rewardVaultPda,
            rewardClaim: rewardClaimPda,
            user: player1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([player1])
          .rpc();

        // If rewards are available, verify claim
        const rewardClaim = await program.account.rewardClaim.fetch(rewardClaimPda);
        expect(rewardClaim.user.toString()).to.equal(player1.publicKey.toString());
      } catch (err) {
        // Expected if no rewards available yet (time-based)
        expect(err.toString()).to.include("NoRewardsAvailable");
      }
    });
  });

  describe("House Operations", () => {
    it("House authority withdraws fees", async () => {
      // First ensure house vault has funds (from bet contributions)
      const betAmount = new BN(1 * LAMPORTS_PER_SOL);
      
      const [betPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player1.publicKey.toBuffer(),
          betAmount.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const [vrfRequestPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda.toBuffer()],
        program.programId
      );

      await program.methods
        .contributeBet(betAmount)
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          bet: betPda,
          vrfRequest: vrfRequestPda,
          houseVault: houseVault.publicKey,
          player: player1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1])
        .rpc();

      const houseBalanceBefore = await provider.connection.getBalance(houseVault.publicKey);
      const authorityBalanceBefore = await provider.connection.getBalance(authority.publicKey);

      const withdrawAmount = new BN(0.01 * LAMPORTS_PER_SOL);

      await program.methods
        .withdrawHouse(withdrawAmount)
        .accounts({
          config: configPda,
          houseVault: houseVault.publicKey,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const houseBalanceAfter = await provider.connection.getBalance(houseVault.publicKey);
      const authorityBalanceAfter = await provider.connection.getBalance(authority.publicKey);

      expect(houseBalanceAfter).to.be.lessThan(houseBalanceBefore);
      expect(authorityBalanceAfter).to.be.greaterThan(authorityBalanceBefore);
    });

    it("Fails to withdraw with unauthorized account", async () => {
      const unauthorized = Keypair.generate();
      await provider.connection.requestAirdrop(unauthorized.publicKey, 1 * LAMPORTS_PER_SOL);
      await new Promise((resolve) => setTimeout(resolve, 1000));

      try {
        await program.methods
          .withdrawHouse(new BN(0.01 * LAMPORTS_PER_SOL))
          .accounts({
            config: configPda,
            houseVault: houseVault.publicKey,
            authority: unauthorized.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([unauthorized])
          .rpc();
        
        expect.fail("Should have failed with unauthorized");
      } catch (err) {
        expect(err.toString()).to.include("Unauthorized");
      }
    });
  });

  describe("Configuration Updates", () => {
    it("Authority updates configuration", async () => {
      const newJackpotPercentage = 600; // 6%
      const newMinBet = new BN(0.2 * LAMPORTS_PER_SOL);

      await program.methods
        .updateConfig(
          newJackpotPercentage,
          null,
          null,
          newMinBet,
          null,
          null,
          null,
          null,
          null
        )
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          authority: authority.publicKey,
        })
        .rpc();

      const config = await program.account.config.fetch(configPda);
      expect(config.jackpotPercentage).to.equal(newJackpotPercentage);
      expect(config.minBet.toString()).to.equal(newMinBet.toString());
    });

    it("Fails to update config with unauthorized account", async () => {
      const unauthorized = Keypair.generate();
      await provider.connection.requestAirdrop(unauthorized.publicKey, 1 * LAMPORTS_PER_SOL);
      await new Promise((resolve) => setTimeout(resolve, 1000));

      try {
        await program.methods
          .updateConfig(
            600,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null
          )
          .accounts({
            config: configPda,
            pool: poolPda,
            rewardVault: rewardVaultPda,
            authority: unauthorized.publicKey,
          })
          .signers([unauthorized])
          .rpc();
        
        expect.fail("Should have failed with unauthorized");
      } catch (err) {
        expect(err.toString()).to.include("Unauthorized");
      }
    });

    it("Fails to update config with invalid parameters", async () => {
      try {
        await program.methods
          .updateConfig(
            null,
            null,
            null,
            new BN(100 * LAMPORTS_PER_SOL), // min > max
            null,
            null,
            null,
            null,
            null
          )
          .accounts({
            config: configPda,
            pool: poolPda,
            rewardVault: rewardVaultPda,
            authority: authority.publicKey,
          })
          .rpc();
        
        expect.fail("Should have failed with invalid config");
      } catch (err) {
        expect(err.toString()).to.include("InvalidConfig");
      }
    });
  });

  describe("Edge Cases", () => {
    it("Handles empty pool gracefully", async () => {
      // Try to fulfill with empty pool (should still work but no payout)
      const betAmount = new BN(0.1 * LAMPORTS_PER_SOL);
      const [betPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          player2.publicKey.toBuffer(),
          betAmount.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const [vrfRequestPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vrf_request"), betPda.toBuffer()],
        program.programId
      );

      // Create a small bet
      await program.methods
        .contributeBet(betAmount)
        .accounts({
          config: configPda,
          pool: poolPda,
          rewardVault: rewardVaultPda,
          bet: betPda,
          vrfRequest: vrfRequestPda,
          houseVault: houseVault.publicKey,
          player: player2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player2])
        .rpc();

      // Fulfill with winning result
      const winningVrf = Buffer.alloc(32);
      winningVrf.writeUInt32LE(50, 0);

      await program.methods
        .fulfillJackpot(Array.from(winningVrf))
        .accounts({
          config: configPda,
          pool: poolPda,
          bet: betPda,
          vrfRequest: vrfRequestPda,
          player: player2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Should succeed even with small pool
      const bet = await program.account.bet.fetch(betPda);
      expect(bet.status).to.equal(1); // won
    });
  });
});
