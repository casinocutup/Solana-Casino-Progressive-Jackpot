use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::CasinoError;

/// House authority withdraws accumulated fees
pub fn withdraw_house(
    ctx: Context<WithdrawHouse>,
    amount: u64,
) -> Result<()> {
    let config = &ctx.accounts.config;
    
    require!(
        ctx.accounts.authority.key() == config.authority,
        CasinoError::Unauthorized
    );
    
    let vault_balance = ctx.accounts.house_vault.to_account_info().lamports();
    require!(
        vault_balance >= amount,
        CasinoError::InsufficientFunds
    );
    
    // Transfer to authority
    **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += amount;
    **ctx.accounts.house_vault.to_account_info().try_borrow_mut_lamports()? -= amount;
    
    msg!("House withdrew {} lamports", amount);
    
    emit!(HouseWithdrawal {
        authority: ctx.accounts.authority.key(),
        amount,
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawHouse<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    
    /// CHECK: House vault for fees
    #[account(mut)]
    pub house_vault: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[event]
pub struct HouseWithdrawal {
    pub authority: Pubkey,
    pub amount: u64,
}
