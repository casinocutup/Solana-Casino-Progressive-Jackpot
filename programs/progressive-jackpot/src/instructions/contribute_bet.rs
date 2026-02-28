use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CasinoError;

/// Player contributes a bet to the jackpot pool
/// Automatically distributes funds: jackpot, house, DeFi
/// Triggers VRF request if win condition might be met
pub fn contribute_bet(
    ctx: Context<ContributeBet>,
    amount: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let pool = &mut ctx.accounts.pool;
    let reward_vault = &mut ctx.accounts.reward_vault;
    
    // Validate bet amount
    require!(
        amount >= config.min_bet,
        CasinoError::BetTooSmall
    );
    
    require!(
        amount <= config.max_bet,
        CasinoError::BetTooLarge
    );
    
    // Calculate distribution
    let jackpot_contribution = amount
        .checked_mul(config.jackpot_percentage as u64)
        .and_then(|x| x.checked_div(10000))
        .ok_or(CasinoError::MathOverflow)?;
    
    let house_fee = amount
        .checked_mul(config.house_percentage as u64)
        .and_then(|x| x.checked_div(10000))
        .ok_or(CasinoError::MathOverflow)?;
    
    let defi_contribution = amount
        .checked_mul(config.defi_percentage as u64)
        .and_then(|x| x.checked_div(10000))
        .ok_or(CasinoError::MathOverflow)?;
    
    // Transfer SOL to program
    **ctx.accounts.pool.to_account_info().try_borrow_mut_lamports()? += jackpot_contribution;
    **ctx.accounts.player.to_account_info().try_borrow_mut_lamports()? -= jackpot_contribution;
    
    **ctx.accounts.house_vault.to_account_info().try_borrow_mut_lamports()? += house_fee;
    **ctx.accounts.player.to_account_info().try_borrow_mut_lamports()? -= house_fee;
    
    **ctx.accounts.reward_vault.to_account_info().try_borrow_mut_lamports()? += defi_contribution;
    **ctx.accounts.player.to_account_info().try_borrow_mut_lamports()? -= defi_contribution;
    
    // Update state
    pool.balance = pool.balance
        .checked_add(jackpot_contribution)
        .ok_or(CasinoError::MathOverflow)?;
    
    pool.bets_since_win = pool.bets_since_win
        .checked_add(1)
        .ok_or(CasinoError::MathOverflow)?;
    
    config.total_bets = config.total_bets
        .checked_add(1)
        .ok_or(CasinoError::MathOverflow)?;
    
    reward_vault.staked_amount = reward_vault.staked_amount
        .checked_add(defi_contribution)
        .ok_or(CasinoError::MathOverflow)?;
    
    // Check if we should trigger VRF (milestone or random chance)
    let should_trigger_vrf = if pool.milestone_bets > 0 {
        pool.bets_since_win >= pool.milestone_bets
    } else {
        // Random chance: in production, this would be determined off-chain
        // For now, we'll always create a VRF request for tracking
        true
    };
    
    if should_trigger_vrf {
        // Create VRF request account
        let vrf_request = &mut ctx.accounts.vrf_request;
        let request_id = Clock::get()?.unix_timestamp.to_le_bytes();
        let mut request_id_bytes = [0u8; 32];
        request_id_bytes[..8].copy_from_slice(&request_id);
        
        vrf_request.bet = ctx.accounts.bet.key();
        vrf_request.player = ctx.accounts.player.key();
        vrf_request.timestamp = Clock::get()?.unix_timestamp;
        vrf_request.request_id = request_id_bytes;
        vrf_request.status = 0; // pending
        vrf_request.result = None;
        vrf_request.bump = ctx.bumps.vrf_request;
        
        // In production, here you would:
        // - For ORAO: Call orao_solana_vrf::request()
        // - For Switchboard: Call switchboard_v2::request()
        // For now, we'll simulate with a placeholder
        msg!("VRF request created: {:?}", request_id_bytes);
    }
    
    // Create bet record
    let bet = &mut ctx.accounts.bet;
    bet.player = ctx.accounts.player.key();
    bet.amount = amount;
    bet.timestamp = Clock::get()?.unix_timestamp;
    bet.vrf_request_id = if should_trigger_vrf {
        Some(ctx.accounts.vrf_request.request_id)
    } else {
        None
    };
    bet.status = 0; // pending
    bet.win_amount = 0;
    bet.bump = ctx.bumps.bet;
    
    msg!(
        "Bet contributed: {} lamports, jackpot={}, house={}, defi={}",
        amount, jackpot_contribution, house_fee, defi_contribution
    );
    
    emit!(BetContributed {
        player: ctx.accounts.player.key(),
        amount,
        jackpot_contribution,
        pool_balance: pool.balance,
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct ContributeBet<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    
    #[account(mut, seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, JackpotPool>,
    
    #[account(mut, seeds = [b"reward_vault"], bump = reward_vault.bump)]
    pub reward_vault: Account<'info, RewardVault>,
    
    #[account(
        init,
        payer = player,
        space = 8 + std::mem::size_of::<Bet>(),
        seeds = [b"bet", player.key().as_ref(), amount.to_le_bytes().as_ref()],
        bump
    )]
    pub bet: Account<'info, Bet>,
    
    #[account(
        init,
        payer = player,
        space = 8 + std::mem::size_of::<VrfRequest>(),
        seeds = [b"vrf_request", bet.key().as_ref()],
        bump
    )]
    pub vrf_request: Account<'info, VrfRequest>,
    
    /// CHECK: House vault for fees (can be any account)
    #[account(mut)]
    pub house_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[event]
pub struct BetContributed {
    pub player: Pubkey,
    pub amount: u64,
    pub jackpot_contribution: u64,
    pub pool_balance: u64,
}
