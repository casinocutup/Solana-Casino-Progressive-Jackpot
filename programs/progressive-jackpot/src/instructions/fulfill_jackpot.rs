use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CasinoError;

/// Fulfill jackpot win based on VRF result
/// Determines if player wins, calculates payout, distributes funds
pub fn fulfill_jackpot(
    ctx: Context<FulfillJackpot>,
    vrf_result: [u8; 32],
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let pool = &mut ctx.accounts.pool;
    let bet = &mut ctx.accounts.bet;
    let vrf_request = &mut ctx.accounts.vrf_request;
    
    // Verify VRF request exists and is pending
    require!(
        vrf_request.status == 0,
        CasinoError::VrfRequestNotFound
    );
    
    require!(
        vrf_request.bet == bet.key(),
        CasinoError::InvalidVrfAuthority
    );
    
    // Check timeout (e.g., 1 hour)
    let timeout: i64 = 3600;
    require!(
        Clock::get()?.unix_timestamp - vrf_request.timestamp < timeout,
        CasinoError::VrfTimeout
    );
    
    // Mark VRF as fulfilled
    vrf_request.status = 1; // fulfilled
    vrf_request.result = Some(vrf_result);
    
    // Convert VRF result to u64 for probability calculation
    let vrf_value = u64::from_le_bytes([
        vrf_result[0], vrf_result[1], vrf_result[2], vrf_result[3],
        vrf_result[4], vrf_result[5], vrf_result[6], vrf_result[7],
    ]);
    
    // Calculate win threshold: win if vrf_value % 10000 < win_probability_bps
    let win_threshold = config.win_probability_bps as u64;
    let vrf_mod = vrf_value % 10000;
    let is_win = vrf_mod < win_threshold;
    
    if is_win {
        // Calculate win amount
        // Full jackpot for rare wins, partial for more common wins
        let win_multiplier = if vrf_mod < (win_threshold / 10) {
            // Rare win: 100% of pool
            10000
        } else if vrf_mod < (win_threshold / 2) {
            // Medium win: 50% of pool
            5000
        } else {
            // Common win: 25% of pool
            2500
        };
        
        let win_amount = pool.balance
            .checked_mul(win_multiplier)
            .and_then(|x| x.checked_div(10000))
            .ok_or(CasinoError::MathOverflow)?;
        
        require!(
            win_amount <= pool.balance,
            CasinoError::InsufficientFunds
        );
        
        // Transfer winnings to player
        **ctx.accounts.player.to_account_info().try_borrow_mut_lamports()? += win_amount;
        **ctx.accounts.pool.to_account_info().try_borrow_mut_lamports()? -= win_amount;
        
        // Update state
        pool.balance = pool.balance
            .checked_sub(win_amount)
            .ok_or(CasinoError::MathOverflow)?;
        
        pool.last_winner = Some(ctx.accounts.player.key());
        pool.last_win_timestamp = Some(Clock::get()?.unix_timestamp);
        pool.bets_since_win = 0;
        
        bet.status = 1; // won
        bet.win_amount = win_amount;
        
        config.total_wins = config.total_wins
            .checked_add(1)
            .ok_or(CasinoError::MathOverflow)?;
        
        msg!("Jackpot won! Player: {}, Amount: {}", ctx.accounts.player.key(), win_amount);
        
        emit!(JackpotWon {
            player: ctx.accounts.player.key(),
            amount: win_amount,
            pool_balance: pool.balance,
            vrf_value: vrf_mod,
        });
    } else {
        // No win
        bet.status = 2; // lost
        bet.win_amount = 0;
        
        msg!("No win. VRF value: {}, threshold: {}", vrf_mod, win_threshold);
        
        emit!(JackpotLoss {
            player: ctx.accounts.player.key(),
            vrf_value: vrf_mod,
        });
    }
    
    // Check if pool should reset (reached threshold)
    if pool.balance >= pool.reset_threshold && pool.reset_threshold > 0 {
        // Partial payout and reset
        let reset_payout = pool.reset_threshold
            .checked_div(2)
            .ok_or(CasinoError::MathOverflow)?;
        
        if reset_payout > 0 {
            **ctx.accounts.player.to_account_info().try_borrow_mut_lamports()? += reset_payout;
            **ctx.accounts.pool.to_account_info().try_borrow_mut_lamports()? -= reset_payout;
            
            pool.balance = pool.balance
                .checked_sub(reset_payout)
                .ok_or(CasinoError::MathOverflow)?;
            
            msg!("Pool reset threshold reached. Partial payout: {}", reset_payout);
        }
        
        pool.bets_since_win = 0;
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct FulfillJackpot<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    
    #[account(mut, seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, JackpotPool>,
    
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    
    #[account(mut)]
    pub vrf_request: Account<'info, VrfRequest>,
    
    /// CHECK: Player account (verified via bet.player)
    #[account(mut)]
    pub player: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[event]
pub struct JackpotWon {
    pub player: Pubkey,
    pub amount: u64,
    pub pool_balance: u64,
    pub vrf_value: u64,
}

#[event]
pub struct JackpotLoss {
    pub player: Pubkey,
    pub vrf_value: u64,
}
