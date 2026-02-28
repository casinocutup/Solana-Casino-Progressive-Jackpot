use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CasinoError;

/// Initialize the casino jackpot system
/// Creates config, jackpot pool, and DeFi reward vault PDAs
pub fn initialize(
    ctx: Context<Initialize>,
    jackpot_percentage: u16,
    house_percentage: u16,
    defi_percentage: u16,
    min_bet: u64,
    max_bet: u64,
    win_probability_bps: u16,
    vrf_provider: u8,
    orao_network: Option<Pubkey>,
    switchboard_queue: Option<Pubkey>,
    reset_threshold: u64,
    milestone_bets: u64,
    apy_bps: u16,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let pool = &mut ctx.accounts.pool;
    let reward_vault = &mut ctx.accounts.reward_vault;
    
    // Validate percentages sum to reasonable amount (not more than 100%)
    let total_percentage = jackpot_percentage
        .checked_add(house_percentage)
        .and_then(|x| x.checked_add(defi_percentage))
        .ok_or(CasinoError::MathOverflow)?;
    
    require!(
        total_percentage <= 10000,
        CasinoError::InvalidConfig
    );
    
    require!(
        min_bet > 0 && max_bet >= min_bet,
        CasinoError::InvalidConfig
    );
    
    require!(
        win_probability_bps > 0 && win_probability_bps <= 10000,
        CasinoError::InvalidConfig
    );
    
    require!(
        vrf_provider <= 1,
        CasinoError::InvalidConfig
    );
    
    // Initialize config
    config.authority = ctx.accounts.authority.key();
    config.jackpot_percentage = jackpot_percentage;
    config.house_percentage = house_percentage;
    config.defi_percentage = defi_percentage;
    config.min_bet = min_bet;
    config.max_bet = max_bet;
    config.win_probability_bps = win_probability_bps;
    config.vrf_provider = vrf_provider;
    config.orao_network = orao_network;
    config.switchboard_queue = switchboard_queue;
    config.defi_vault_bump = ctx.bumps.reward_vault;
    config.total_bets = 0;
    config.total_wins = 0;
    config.bump = ctx.bumps.config;
    
    // Initialize pool
    pool.balance = 0;
    pool.last_winner = None;
    pool.last_win_timestamp = None;
    pool.reset_threshold = reset_threshold;
    pool.bets_since_win = 0;
    pool.milestone_bets = milestone_bets;
    pool.bump = ctx.bumps.pool;
    
    // Initialize reward vault
    reward_vault.staked_amount = 0;
    reward_vault.total_rewards_distributed = 0;
    reward_vault.last_distribution = Clock::get()?.unix_timestamp;
    reward_vault.distribution_period = 86400; // 1 day default
    reward_vault.apy_bps = apy_bps;
    reward_vault.bump = ctx.bumps.reward_vault;
    
    msg!("Casino initialized: jackpot={}%, house={}%, defi={}%", 
         jackpot_percentage, house_percentage, defi_percentage);
    
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<Config>(),
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<JackpotPool>(),
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, JackpotPool>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<RewardVault>(),
        seeds = [b"reward_vault"],
        bump
    )]
    pub reward_vault: Account<'info, RewardVault>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
