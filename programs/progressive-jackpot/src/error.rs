use anchor_lang::prelude::*;

#[error_code]
pub enum CasinoError {
    #[msg("Invalid bet amount: below minimum")]
    BetTooSmall,
    
    #[msg("Invalid bet amount: above maximum")]
    BetTooLarge,
    
    #[msg("Jackpot pool is empty")]
    EmptyPool,
    
    #[msg("VRF request not found or expired")]
    VrfRequestNotFound,
    
    #[msg("VRF request not fulfilled yet")]
    VrfNotFulfilled,
    
    #[msg("VRF request already fulfilled")]
    VrfAlreadyFulfilled,
    
    #[msg("Invalid VRF authority")]
    InvalidVrfAuthority,
    
    #[msg("No win condition met")]
    NoWin,
    
    #[msg("Insufficient funds for payout")]
    InsufficientFunds,
    
    #[msg("Unauthorized: not house authority")]
    Unauthorized,
    
    #[msg("Invalid configuration parameters")]
    InvalidConfig,
    
    #[msg("DeFi staking not initialized")]
    DefiNotInitialized,
    
    #[msg("No rewards available to claim")]
    NoRewardsAvailable,
    
    #[msg("Reward claim period not started")]
    ClaimPeriodNotStarted,
    
    #[msg("Math overflow in calculation")]
    MathOverflow,
    
    #[msg("VRF request timeout: refunding bet")]
    VrfTimeout,
    
    #[msg("Invalid win probability threshold")]
    InvalidWinThreshold,
    
    #[msg("Jackpot reset threshold not met")]
    ResetThresholdNotMet,
}
