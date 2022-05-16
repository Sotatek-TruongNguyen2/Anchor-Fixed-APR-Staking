pub mod account;
pub mod context;
pub mod error;
pub mod helpers;

use account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, TokenAccount};
use context::*;
use error::*;
use helpers::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_vesting {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializeStakingInfo>,
        minimum_staking_amount: u64,
        maximum_staking_amount: u64,
        lock_duration: i64,
        start_join_time: i64,
        end_join_time: i64,
        delay_duration: i64,
        apr: u32,
        total_slot: u8,
    ) -> Result<()> {
        let ruin_staking_admin: &Signer = &ctx.accounts.ruin_staking_admin;
        let ruin_staking_distributor: &Account<TokenAccount> =
            &ctx.accounts.ruin_staking_distributor;
        let ruin_staking_treasury: &Account<TokenAccount> = &ctx.accounts.ruin_staking_treasury;
        let ruin_staking_token: &Account<Mint> = &ctx.accounts.ruin_staking_token;

        let ruin_staking: &mut Account<RuinStaking> = &mut ctx.accounts.ruin_staking;
        let ruin_staking_term: &mut Account<RuinStakingTerm> = &mut ctx.accounts.ruin_staking_term;

        ruin_staking.staking_admin = ruin_staking_admin.key();
        ruin_staking.staking_token = ruin_staking_token.key();
        ruin_staking.distributor = ruin_staking_distributor.key();
        ruin_staking.treasury = ruin_staking_treasury.key();
        ruin_staking.paused = false;
        ruin_staking.total_staked = 0;

        ruin_staking_term.ruin_staking = ruin_staking.key();
        ruin_staking_term.maximum_staking_amount = maximum_staking_amount;
        ruin_staking_term.minimum_staking_amount = minimum_staking_amount;
        ruin_staking_term.start_join_time = start_join_time;
        ruin_staking_term.end_join_time = end_join_time;
        ruin_staking_term.lock_duration = lock_duration;
        ruin_staking_term.delay_duration = delay_duration;
        ruin_staking_term.apr = apr;
        ruin_staking_term.total_slot = total_slot;

        ruin_staking.distributor_bump = *ctx.bumps.get("ruin_staking_distributor").unwrap();
        ruin_staking.ruin_staking_bump = *ctx.bumps.get("ruin_staking").unwrap();
        ruin_staking_term.ruin_staking_term_bump = *ctx.bumps.get("ruin_staking_term").unwrap();

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let ruin_staking_term: &Account<RuinStakingTerm> = &ctx.accounts.ruin_staking_term;
        let clock: Clock = Clock::get().unwrap();

        let user_staked: &mut Account<UserStaked> = &mut ctx.accounts.user_staked;
        let user_pending_reward: &mut Account<PendingWithdrawl> =
            &mut ctx.accounts.user_pending_withdraw;

        if user_pending_reward.claimable_at == 0 {
            user_pending_reward.pending_withdrawl_bump =
                *ctx.bumps.get("user_pending_reward").unwrap();
        }

        if amount > user_staked.total_staked {
            return Err(ProgramErrorCode::ExceedsCurrentStakingAmount.into());
        }

        if user_staked
            .join_time
            .checked_add(ruin_staking_term.lock_duration)
            .unwrap()
            < clock.unix_timestamp
        {
            user_pending_reward.pending_tokens = user_pending_reward
                .pending_tokens
                .checked_add(amount)
                .unwrap();
            user_pending_reward.claimable_at = clock
                .unix_timestamp
                .checked_add(ruin_staking_term.lock_duration)
                .unwrap();
        }

        Ok(())
    }

    pub fn harvest(ctx: Context<Harvest>) -> Result<()> {
        let ruin_staking_term: &Account<RuinStakingTerm> = &ctx.accounts.ruin_staking_term;
        let clock: Clock = Clock::get().unwrap();

        let user_staked: &mut Account<UserStaked> = &mut ctx.accounts.user_staked;

        let pending_reward = calculate_reward(ruin_staking_term, &user_staked);

        let user_pending_withdrawl: &mut Account<PendingRewardWithdrawl> =
            &mut ctx.accounts.user_pending_withdrawl;

        if pending_reward > 0 {
            user_pending_withdrawl.claimable_at = clock
                .unix_timestamp
                .checked_add(ruin_staking_term.delay_duration)
                .unwrap();
            user_pending_withdrawl.pending_rewards = user_pending_withdrawl
                .pending_rewards
                .checked_add(pending_reward)
                .unwrap();
        }

        user_staked.updated_time = clock.unix_timestamp;

        Ok(())
    }

    pub fn claim_pending_reward(ctx: Context<ClaimPendingReward>) -> Result<()> {
        let clock: Clock = Clock::get().unwrap();

        let user_pending_withdrawl: &Account<PendingRewardWithdrawl> =
            &ctx.accounts.user_pending_withdrawl;

        let staking_admin = ctx.accounts.ruin_staking.staking_admin.key();
        let staking_token = ctx.accounts.ruin_staking.staking_token.key();

        if user_pending_withdrawl.claimable_at > clock.unix_timestamp
            && user_pending_withdrawl.pending_rewards > 0
        {
            let seeds = &[
                b"staking",
                staking_token.as_ref(),
                staking_admin.as_ref(),
                &[ctx.accounts.ruin_staking_term.lock_duration as u8],
                &[ctx.accounts.ruin_staking.ruin_staking_bump],
            ];

            let signer = &[&seeds[..]];

            transfer(
                ctx.accounts
                    .into_transfer_reward_to_investor()
                    .with_signer(signer),
                user_pending_withdrawl
                    .pending_rewards
                    .checked_div(DIV_PRECISION.try_into().unwrap())
                    .unwrap()
                    .try_into()
                    .unwrap(),
            )?;

            ctx.accounts.user_pending_withdrawl.pending_rewards = 0;
            msg!(
                "{:#?} claim reward: {:#?}",
                ctx.accounts.investor.key(),
                ctx.accounts.user_pending_withdrawl.pending_rewards
            )
        }

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let investor_token_account: &Account<TokenAccount> = &ctx.accounts.investor_token_account;
        let ruin_staking_term: &Account<RuinStakingTerm> = &ctx.accounts.ruin_staking_term;
        let clock: Clock = Clock::get().unwrap();

        let current_time = clock.unix_timestamp;

        let user_staked: &mut Account<UserStaked> = &mut ctx.accounts.user_staked;

        let user_pending_withdrawl: &mut Account<PendingRewardWithdrawl> =
            &mut ctx.accounts.user_pending_withdrawl;

        // Create user account for calculating reward and pending withdraw
        if user_staked.join_time == 0 {
            user_staked.user_staked_bump = *ctx.bumps.get("user_staked").unwrap();
            user_pending_withdrawl.pending_reward_withdrawl_bump =
                *ctx.bumps.get("user_pending_withdrawl").unwrap();
        }

        let pending_reward = calculate_reward(ruin_staking_term, &user_staked.clone());

        if pending_reward > 0 {
            user_pending_withdrawl.claimable_at = clock
                .unix_timestamp
                .checked_add(ruin_staking_term.delay_duration)
                .unwrap();
            user_pending_withdrawl.pending_rewards = user_pending_withdrawl
                .pending_rewards
                .checked_add(pending_reward)
                .unwrap();
        }

        user_staked.total_staked = user_staked.total_staked.checked_add(amount).unwrap();
        user_staked.join_time = current_time;

        if investor_token_account.amount < amount {
            return Err(ProgramErrorCode::ExceedsCurrentBalance.into());
        }

        if current_time < ruin_staking_term.start_join_time.try_into().unwrap()
            || current_time > ruin_staking_term.end_join_time.try_into().unwrap()
        {
            return Err(ProgramErrorCode::StakingNotStartedOrEnded.into());
        }

        if user_staked.total_staked < ruin_staking_term.minimum_staking_amount {
            return Err(ProgramErrorCode::MinimumAmountNotReached.into());
        }

        if user_staked.total_staked > ruin_staking_term.maximum_staking_amount {
            return Err(ProgramErrorCode::MaximumAmountExceeds.into());
        }

        user_staked.updated_time = current_time;

        transfer(ctx.accounts.into_transfer_token_to_treasury(), amount)?;

        Ok(())
    }
}
