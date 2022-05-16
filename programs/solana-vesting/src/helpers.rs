use crate::account::*;
use anchor_lang::prelude::*;

pub const ONE_YEAR_IN_SECONDS: u64 = 365 * 24 * 60 * 60;
pub const DIV_PRECISION: u64 = u64::pow(10, 12);

pub fn calculate_reward(
    term: &Account<RuinStakingTerm>,
    user_staked: &Account<UserStaked>,
) -> u128 {
    let clock: Clock = Clock::get().unwrap();

    let mut start_time = clock.unix_timestamp;
    let mut end_time = clock.unix_timestamp;

    if user_staked.updated_time > 0 {
        start_time = user_staked.updated_time;
    }

    if term.lock_duration > 0
        && user_staked
            .join_time
            .checked_add(term.lock_duration)
            .unwrap()
            < end_time
    {
        end_time = user_staked.join_time + term.delay_duration;
    }

    let mut stake_time_in_seconds: i64 = end_time.checked_sub(start_time).unwrap();

    if stake_time_in_seconds < 0 {
        stake_time_in_seconds = 0;
    }

    let user_total_staked: u128 = user_staked.total_staked.try_into().unwrap();

    let reward: u128 = user_total_staked
        .checked_mul(stake_time_in_seconds.try_into().unwrap())
        .unwrap()
        .checked_mul(term.apr.try_into().unwrap())
        .unwrap()
        .checked_div(10000)
        .unwrap()
        .checked_mul(DIV_PRECISION.try_into().unwrap())
        .unwrap()
        .checked_div(ONE_YEAR_IN_SECONDS.try_into().unwrap())
        .unwrap()
        .try_into()
        .unwrap();

    return reward;
}
