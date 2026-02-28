use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CasinoError;

/// Update configuration parameters (authority only)
pub fn update_config(
    ctx: Context<UpdateConfig>,
    jackpot_percentage: Option<u16>,
    house_percentage: Option<u16>,
    defi_percentage: Option<u16>,
    min_bet: Option<u64>,
    max_bet: Option<u64>,
    win_probability_bps: Option<u16>,
    reset_threshold: Option<u64>,
    milestone_bets: Option<u64>,
    apy_bps: Option<u16>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let pool = &mut ctx.accounts.pool;
    let reward_vault = &mut ctx.accounts.reward_vault;
    
    require!(
        ctx.accounts.authority.key() == config.authority,
        CasinoError::Unauthorized
    );
    
    // Update config fields if provided
    if let Some(jp) = jackpot_percentage {
        config.jackpot_percentage = jp;
    }
    
    if let Some(hp) = house_percentage {
        config.house_percentage = hp;
    }
    
    if let Some(dp) = defi_percentage {
        config.defi_percentage = dp;
    }
    
    if let Some(mb) = min_bet {
        require!(mb > 0, CasinoError::InvalidConfig);
        config.min_bet = mb;
    }
    
    if let Some(mxb) = max_bet {
        require!(mxb >= config.min_bet, CasinoError::InvalidConfig);
        config.max_bet = mxb;
    }
    
    if let Some(wp) = win_probability_bps {
        require!(wp > 0 && wp <= 10000, CasinoError::InvalidConfig);
        config.win_probability_bps = wp;
    }
    
    // Validate total percentage
    let total_percentage = config.jackpot_percentage
        .checked_add(config.house_percentage)
        .and_then(|x| x.checked_add(config.defi_percentage))
        .ok_or(CasinoError::MathOverflow)?;
    
    require!(
        total_percentage <= 10000,
        CasinoError::InvalidConfig
    );
    
    // Update pool
    if let Some(rt) = reset_threshold {
        pool.reset_threshold = rt;
    }
    
    if let Some(mb) = milestone_bets {
        pool.milestone_bets = mb;
    }
    
    // Update reward vault
    if let Some(apy) = apy_bps {
        reward_vault.apy_bps = apy;
    }
    
    msg!("Config updated by {}", ctx.accounts.authority.key());
    
    emit!(ConfigUpdated {
        authority: ctx.accounts.authority.key(),
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    
    #[account(mut, seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, JackpotPool>,
    
    #[account(mut, seeds = [b"reward_vault"], bump = reward_vault.bump)]
    pub reward_vault: Account<'info, RewardVault>,
    
    pub authority: Signer<'info>,
}

#[event]
pub struct ConfigUpdated {
    pub authority: Pubkey,
}
