use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

/// Global configuration for the casino jackpot system
#[account]
#[derive(Default)]
pub struct Config {
    /// Authority that can update config and withdraw house fees
    pub authority: Pubkey,
    
    /// Percentage of each bet that goes to jackpot (basis points, e.g., 500 = 5%)
    pub jackpot_percentage: u16,
    
    /// Percentage of each bet that goes to house (basis points, e.g., 200 = 2%)
    pub house_percentage: u16,
    
    /// Percentage of each bet that goes to DeFi rewards pool (basis points, e.g., 100 = 1%)
    pub defi_percentage: u16,
    
    /// Minimum bet amount in lamports
    pub min_bet: u64,
    
    /// Maximum bet amount in lamports
    pub max_bet: u64,
    
    /// Win probability per bet (basis points, e.g., 1 = 0.01% = 1/10000)
    pub win_probability_bps: u16,
    
    /// VRF provider: 0 = ORAO, 1 = Switchboard
    pub vrf_provider: u8,
    
    /// ORAO VRF network account (if using ORAO)
    pub orao_network: Option<Pubkey>,
    
    /// Switchboard VRF queue (if using Switchboard)
    pub switchboard_queue: Option<Pubkey>,
    
    /// DeFi staking vault PDA bump
    pub defi_vault_bump: u8,
    
    /// Total bets contributed
    pub total_bets: u64,
    
    /// Total jackpot wins
    pub total_wins: u64,
    
    /// Bump seed for config PDA
    pub bump: u8,
}

/// Progressive jackpot pool account
#[account]
#[derive(Default)]
pub struct JackpotPool {
    /// Current balance of the jackpot pool
    pub balance: u64,
    
    /// Last winner address (if any)
    pub last_winner: Option<Pubkey>,
    
    /// Timestamp of last win
    pub last_win_timestamp: Option<i64>,
    
    /// Reset threshold: if pool reaches this, auto-reset with partial payout
    pub reset_threshold: u64,
    
    /// Number of bets since last win
    pub bets_since_win: u64,
    
    /// Milestone trigger: win every N bets (0 = disabled)
    pub milestone_bets: u64,
    
    /// Bump seed for pool PDA
    pub bump: u8,
}

/// Individual bet record (optional, for large bets or tracking)
#[account]
#[derive(Default)]
pub struct Bet {
    /// Player who placed the bet
    pub player: Pubkey,
    
    /// Bet amount in lamports
    pub amount: u64,
    
    /// Timestamp when bet was placed
    pub timestamp: i64,
    
    /// VRF request ID (if VRF was triggered)
    pub vrf_request_id: Option<[u8; 32]>,
    
    /// Status: 0 = pending, 1 = won, 2 = lost, 3 = refunded
    pub status: u8,
    
    /// Win amount if won (0 if lost)
    pub win_amount: u64,
    
    /// Bump seed for bet PDA
    pub bump: u8,
}

/// DeFi reward vault for staking yields
#[account]
#[derive(Default)]
pub struct RewardVault {
    /// Total staked amount
    pub staked_amount: u64,
    
    /// Total rewards distributed
    pub total_rewards_distributed: u64,
    
    /// Last reward distribution timestamp
    pub last_distribution: i64,
    
    /// Reward distribution period (seconds)
    pub distribution_period: i64,
    
    /// Annual percentage yield (basis points, e.g., 500 = 5% APY)
    pub apy_bps: u16,
    
    /// Bump seed for vault PDA
    pub bump: u8,
}

/// User reward claim account
#[account]
#[derive(Default)]
pub struct RewardClaim {
    /// User who can claim rewards
    pub user: Pubkey,
    
    /// Total rewards earned
    pub total_earned: u64,
    
    /// Total rewards claimed
    pub total_claimed: u64,
    
    /// Last claim timestamp
    pub last_claim: i64,
    
    /// Bump seed for claim PDA
    pub bump: u8,
}

/// VRF request tracking account
#[account]
#[derive(Default)]
pub struct VrfRequest {
    /// Bet account associated with this request
    pub bet: Pubkey,
    
    /// Player who placed the bet
    pub player: Pubkey,
    
    /// VRF request timestamp
    pub timestamp: i64,
    
    /// VRF request ID/seed
    pub request_id: [u8; 32],
    
    /// Status: 0 = pending, 1 = fulfilled, 2 = timeout
    pub status: u8,
    
    /// VRF result (if fulfilled)
    pub result: Option<[u8; 32]>,
    
    /// Bump seed for request PDA
    pub bump: u8,
}
