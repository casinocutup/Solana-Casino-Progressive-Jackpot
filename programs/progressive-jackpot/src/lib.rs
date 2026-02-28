use anchor_lang::prelude::*;

pub mod error;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("JACKPOT1111111111111111111111111111111");

#[program]
pub mod progressive_jackpot {
    use super::*;

    /// Initialize the casino jackpot system
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
        instructions::initialize::initialize(
            ctx,
            jackpot_percentage,
            house_percentage,
            defi_percentage,
            min_bet,
            max_bet,
            win_probability_bps,
            vrf_provider,
            orao_network,
            switchboard_queue,
            reset_threshold,
            milestone_bets,
            apy_bps,
        )
    }

    /// Player contributes a bet to the jackpot pool
    pub fn contribute_bet(
        ctx: Context<ContributeBet>,
        amount: u64,
    ) -> Result<()> {
        instructions::contribute_bet::contribute_bet(ctx, amount)
    }

    /// Fulfill jackpot win based on VRF result
    pub fn fulfill_jackpot(
        ctx: Context<FulfillJackpot>,
        vrf_result: [u8; 32],
    ) -> Result<()> {
        instructions::fulfill_jackpot::fulfill_jackpot(ctx, vrf_result)
    }

    /// Claim DeFi rewards from staked pool
    pub fn claim_rewards(
        ctx: Context<ClaimRewards>,
    ) -> Result<()> {
        instructions::claim_rewards::claim_rewards(ctx)
    }

    /// House authority withdraws accumulated fees
    pub fn withdraw_house(
        ctx: Context<WithdrawHouse>,
        amount: u64,
    ) -> Result<()> {
        instructions::withdraw_house::withdraw_house(ctx, amount)
    }

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
        instructions::update_config::update_config(
            ctx,
            jackpot_percentage,
            house_percentage,
            defi_percentage,
            min_bet,
            max_bet,
            win_probability_bps,
            reset_threshold,
            milestone_bets,
            apy_bps,
        )
    }
}
