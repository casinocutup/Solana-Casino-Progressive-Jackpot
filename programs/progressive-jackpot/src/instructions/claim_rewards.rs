use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CasinoError;

/// Claim DeFi rewards from staked pool
/// Calculates rewards based on APY and time staked
pub fn claim_rewards(
    ctx: Context<ClaimRewards>,
) -> Result<()> {
    let reward_vault = &mut ctx.accounts.reward_vault;
    let reward_claim = &mut ctx.accounts.reward_claim;
    let config = &ctx.accounts.config;
    
    require!(
        reward_vault.staked_amount > 0,
        CasinoError::DefiNotInitialized
    );
    
    let current_time = Clock::get()?.unix_timestamp;
    
    // Initialize claim if first time
    if reward_claim.user == Pubkey::default() {
        reward_claim.user = ctx.accounts.user.key();
        reward_claim.total_earned = 0;
        reward_claim.total_claimed = 0;
        reward_claim.last_claim = current_time;
        reward_claim.bump = ctx.bumps.reward_claim;
    }
    
    // Calculate rewards based on APY
    // Formula: rewards = staked_amount * (APY / 100) * (time_elapsed / year_seconds)
    let year_seconds: i64 = 31536000; // 365 days
    let time_elapsed = current_time
        .checked_sub(reward_claim.last_claim)
        .unwrap_or(0);
    
    if time_elapsed <= 0 {
        return Err(CasinoError::ClaimPeriodNotStarted.into());
    }
    
    // Calculate user's share of rewards (simplified: equal share for all contributors)
    // In production, this would track individual contributions
    let apy_decimal = (config.defi_percentage as u64)
        .checked_mul(reward_vault.apy_bps as u64)
        .and_then(|x| x.checked_div(10000))
        .ok_or(CasinoError::MathOverflow)?;
    
    let rewards = reward_vault.staked_amount
        .checked_mul(apy_decimal)
        .and_then(|x| x.checked_mul(time_elapsed as u64))
        .and_then(|x| x.checked_div(10000))
        .and_then(|x| x.checked_div(year_seconds as u64))
        .ok_or(CasinoError::MathOverflow)?;
    
    require!(
        rewards > 0,
        CasinoError::NoRewardsAvailable
    );
    
    // Check if vault has enough funds
    let vault_balance = ctx.accounts.reward_vault.to_account_info().lamports();
    require!(
        vault_balance >= rewards,
        CasinoError::InsufficientFunds
    );
    
    // Transfer rewards to user
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += rewards;
    **ctx.accounts.reward_vault.to_account_info().try_borrow_mut_lamports()? -= rewards;
    
    // Update claim state
    reward_claim.total_earned = reward_claim.total_earned
        .checked_add(rewards)
        .ok_or(CasinoError::MathOverflow)?;
    
    reward_claim.total_claimed = reward_claim.total_claimed
        .checked_add(rewards)
        .ok_or(CasinoError::MathOverflow)?;
    
    reward_claim.last_claim = current_time;
    
    reward_vault.total_rewards_distributed = reward_vault.total_rewards_distributed
        .checked_add(rewards)
        .ok_or(CasinoError::MathOverflow)?;
    
    reward_vault.last_distribution = current_time;
    
    msg!("Rewards claimed: {} lamports by {}", rewards, ctx.accounts.user.key());
    
    emit!(RewardsClaimed {
        user: ctx.accounts.user.key(),
        amount: rewards,
        total_claimed: reward_claim.total_claimed,
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    
    #[account(mut, seeds = [b"reward_vault"], bump = reward_vault.bump)]
    pub reward_vault: Account<'info, RewardVault>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<RewardClaim>(),
        seeds = [b"reward_claim", user.key().as_ref()],
        bump
    )]
    pub reward_claim: Account<'info, RewardClaim>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[event]
pub struct RewardsClaimed {
    pub user: Pubkey,
    pub amount: u64,
    pub total_claimed: u64,
}
