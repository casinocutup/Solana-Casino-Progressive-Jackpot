# Solana Casino Progressive Jackpot – On-Chain System with DeFi Rewards

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Anchor](https://img.shields.io/badge/Anchor-0.30.0-blue.svg)](https://www.anchor-lang.com/)
[![Solana](https://img.shields.io/badge/Solana-1.18+-purple.svg)](https://solana.com/)

A production-ready, provably fair progressive jackpot system on Solana using Anchor framework. This system accumulates bets from players into a growing jackpot pool, triggers wins based on configurable conditions (VRF-based randomness or bet milestones), and distributes DeFi rewards through integrated staking mechanisms.

## 🎰 Features

### Core Functionality
- **Progressive Jackpot Pool**: Accumulates bets with configurable percentage allocation
- **Verifiable Randomness (VRF)**: Integrated with ORAO VRF (preferred) and Switchboard VRF for provably fair jackpot draws
- **DeFi Rewards Integration**: Staking yields and LP token rewards from pool liquidity
- **Configurable Win Conditions**: 
  - Random VRF-based wins (configurable probability)
  - Milestone-based wins (every N bets)
  - Reset threshold for automatic partial payouts
- **Multiple Payout Tiers**: Full jackpot, 50%, or 25% based on VRF result rarity
- **House Fee Management**: Automated fee collection and withdrawal system

### Security Features
- ✅ Reentrancy protection
- ✅ Checked math for all calculations
- ✅ Signer and bump seed validation
- ✅ Bet limits and validation
- ✅ VRF request timeout handling
- ✅ Comprehensive error handling
- ✅ Event emission for all major actions

## 🏗️ Architecture

### Account Structure

#### Config PDA
- House authority and configuration parameters
- Jackpot, house, and DeFi percentage allocations (basis points)
- Min/max bet limits
- VRF provider settings (ORAO/Switchboard)
- Win probability configuration

#### Jackpot Pool PDA
- Current pool balance
- Last winner and timestamp
- Reset threshold
- Bet counter since last win
- Milestone trigger settings

#### Bet Account (PDA)
- Player address
- Bet amount and timestamp
- VRF request ID (if applicable)
- Win status and amount

#### Reward Vault PDA
- Total staked amount
- APY configuration
- Reward distribution tracking
- Last distribution timestamp

#### VRF Request Account (PDA)
- Associated bet account
- Request ID and timestamp
- Fulfillment status
- VRF result storage

## 📦 Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor](https://www.anchor-lang.com/docs/installation) (v0.30.0+)
- [Node.js](https://nodejs.org/) (v18+)
- [Yarn](https://yarnpkg.com/) or npm

### Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd Solana-Casino-Progressive-Jackpot
   ```

2. **Install dependencies**
   ```bash
   # Install Rust dependencies (handled by Anchor)
   anchor build
   
   # Install Node.js dependencies
   yarn install
   # or
   npm install
   ```

3. **Build the program**
   ```bash
   anchor build
   ```

4. **Run tests**
   ```bash
   anchor test
   ```

## 🚀 Usage

### Initialization

Initialize the casino system with your desired parameters:

```typescript
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

// Initialize the casino
await program.methods
  .initialize(
    500,    // jackpot_percentage: 5% (500 basis points)
    200,    // house_percentage: 2% (200 basis points)
    100,    // defi_percentage: 1% (100 basis points)
    100000000,  // min_bet: 0.1 SOL (in lamports)
    10000000000, // max_bet: 10 SOL (in lamports)
    100,    // win_probability_bps: 1% chance (100 basis points)
    0,      // vrf_provider: 0 = ORAO, 1 = Switchboard
    null,   // orao_network (Pubkey or null)
    null,   // switchboard_queue (Pubkey or null)
    new BN(100000000000), // reset_threshold: 100 SOL
    new BN(1000),         // milestone_bets: win every 1000 bets
    500     // apy_bps: 5% APY for DeFi rewards
  )
  .accounts({
    config: configPda,
    pool: poolPda,
    rewardVault: rewardVaultPda,
    authority: authority.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Contributing Bets

Players contribute bets to the jackpot pool:

```typescript
const betAmount = new BN(1 * LAMPORTS_PER_SOL); // 1 SOL

// Derive bet PDA
const [betPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("bet"),
    player.publicKey.toBuffer(),
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
    player: player.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([player])
  .rpc();
```

### Fulfilling Jackpot Wins

After VRF is fulfilled (via ORAO or Switchboard), determine if player wins:

```typescript
// VRF result from oracle (32 bytes)
const vrfResult = Buffer.alloc(32);
// ... populate with actual VRF result

await program.methods
  .fulfillJackpot(Array.from(vrfResult))
  .accounts({
    config: configPda,
    pool: poolPda,
    bet: betPda,
    vrfRequest: vrfRequestPda,
    player: player.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Claiming DeFi Rewards

Users can claim accumulated DeFi rewards:

```typescript
const [rewardClaimPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("reward_claim"), user.publicKey.toBuffer()],
  program.programId
);

await program.methods
  .claimRewards()
  .accounts({
    config: configPda,
    rewardVault: rewardVaultPda,
    rewardClaim: rewardClaimPda,
    user: user.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([user])
  .rpc();
```

### House Operations

House authority can withdraw accumulated fees:

```typescript
await program.methods
  .withdrawHouse(amount)
  .accounts({
    config: configPda,
    houseVault: houseVault.publicKey,
    authority: authority.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Updating Configuration

Authority can update system parameters:

```typescript
await program.methods
  .updateConfig(
    newJackpotPercentage,  // Optional
    newHousePercentage,     // Optional
    newDefiPercentage,      // Optional
    newMinBet,              // Optional
    newMaxBet,              // Optional
    newWinProbabilityBps,   // Optional
    newResetThreshold,      // Optional
    newMilestoneBets,       // Optional
    newApyBps               // Optional
  )
  .accounts({
    config: configPda,
    pool: poolPda,
    rewardVault: rewardVaultPda,
    authority: authority.publicKey,
  })
  .rpc();
```

## 🎲 Fairness & VRF Verification

### VRF Integration

The system supports two VRF providers:

1. **ORAO VRF** (Preferred for 2026)
   - On-chain verifiable randomness
   - Lower latency
   - Cost-effective
   - Integration: `orao-solana-vrf = "0.4.0"`

2. **Switchboard VRF** (Alternative)
   - Battle-tested oracle network
   - Robust infrastructure
   - Integration: `switchboard-v2 = "0.4.0"`

### Win Probability Calculation

Wins are determined by:
```
vrf_value = VRF_result % 10000
is_win = vrf_value < win_probability_bps
```

Payout tiers based on VRF value:
- **Rare Win** (vrf_value < threshold/10): 100% of pool
- **Medium Win** (vrf_value < threshold/2): 50% of pool
- **Common Win** (vrf_value < threshold): 25% of pool

### Verifying VRF Results

All VRF requests and results are stored on-chain in `VrfRequest` accounts, allowing off-chain verification:
- Request ID and timestamp
- Fulfillment status
- VRF result bytes
- Associated bet account

## 🔒 Security

### Best Practices Implemented

1. **Reentrancy Protection**: All state updates before external calls
2. **Checked Math**: All arithmetic uses `checked_*` methods
3. **Signer Validation**: Authority checks on sensitive operations
4. **PDA Validation**: Bump seeds verified for all PDAs
5. **Input Validation**: Bet limits, percentage bounds, config validation
6. **Timeout Handling**: VRF requests can timeout and refund
7. **Event Emission**: All major actions emit events for monitoring

### Audit Considerations

⚠️ **Important**: This codebase is provided as-is for educational and development purposes. Before deploying to mainnet:

1. Conduct a professional security audit
2. Test thoroughly on devnet/testnet
3. Review all percentage calculations and edge cases
4. Verify VRF integration with actual oracle networks
5. Test DeFi staking integration with real protocols
6. Implement additional rate limiting if needed
7. Consider adding emergency pause functionality

## 📊 Testing

The test suite includes 10+ comprehensive test cases:

```bash
anchor test
```

Test coverage includes:
- ✅ System initialization
- ✅ Bet contributions (valid and invalid)
- ✅ Jackpot fulfillment (win/loss scenarios)
- ✅ DeFi reward claims
- ✅ House withdrawals
- ✅ Configuration updates
- ✅ Authorization checks
- ✅ Edge cases and error handling

## 🛠️ Tech Stack

- **Framework**: [Anchor](https://www.anchor-lang.com/) 0.30.0
- **Language**: Rust (latest stable)
- **Blockchain**: Solana
- **VRF**: ORAO VRF 0.4.0 / Switchboard V2 0.4.0
- **Testing**: TypeScript, Mocha, Chai
- **DeFi Integration**: SPL Token, Staking protocols

## 📁 Project Structure

```
.
├── programs/
│   └── progressive-jackpot/
│       └── src/
│           ├── lib.rs              # Program entry point
│           ├── state.rs            # Account structures
│           ├── error.rs            # Custom error codes
│           └── instructions/       # Instruction modules
│               ├── mod.rs
│               ├── initialize.rs
│               ├── contribute_bet.rs
│               ├── fulfill_jackpot.rs
│               ├── claim_rewards.rs
│               ├── withdraw_house.rs
│               └── update_config.rs
├── tests/
│   └── progressive-jackpot.ts      # Comprehensive test suite
├── Anchor.toml                      # Anchor configuration
├── Cargo.toml                       # Rust dependencies
├── package.json                     # Node.js dependencies
├── tsconfig.json                    # TypeScript configuration
└── README.md                        # This file
```

## 🔄 Integration with Casino Games

This jackpot system can be integrated with various casino games:

1. **Slot Machines**: Trigger VRF on spin completion
2. **Roulette**: Use jackpot for special bonus rounds
3. **Poker**: Progressive tournament prizes
4. **Blackjack**: Side bet jackpot contributions
5. **Custom Games**: Any game can contribute to the pool

Example integration pattern:
```typescript
// In your game contract
await jackpotProgram.methods
  .contributeBet(betAmount)
  .accounts({...})
  .rpc();

// After game logic completes
await jackpotProgram.methods
  .fulfillJackpot(vrfResult)
  .accounts({...})
  .rpc();
```

## 📈 DeFi Rewards Mechanism

The system automatically:
1. Allocates a percentage of each bet to the DeFi reward vault
2. Tracks staked amounts and APY
3. Calculates time-based rewards
4. Allows users to claim accumulated rewards

Reward calculation:
```
rewards = staked_amount × (APY / 100) × (time_elapsed / year_seconds)
```

## 📧 Support

- telegram: https://t.me/CasinoCutup
- twitter:  https://x.com/CasinoCutup
