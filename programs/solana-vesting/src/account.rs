use anchor_lang::prelude::*;

#[account]
pub struct RuinStaking {
    pub distributor_bump: u8,
    pub ruin_staking_bump: u8,
    pub staking_admin: Pubkey,
    pub staking_token: Pubkey,
    pub distributor: Pubkey,
    pub treasury: Pubkey,
    pub total_staked: u128,
    pub paused: bool,
}

#[account]
pub struct RuinStakingTerm {
    pub ruin_staking: Pubkey,
    pub ruin_staking_term_bump: u8,
    pub minimum_staking_amount: u64,
    pub maximum_staking_amount: u64,
    pub start_join_time: i64,
    pub end_join_time: i64,
    pub lock_duration: i64,
    pub delay_duration: i64,
    pub apr: u32,
    pub total_slot: u8,
}

#[account]
pub struct UserStaked {
    pub user_staked_bump: u8,
    pub total_staked: u64,
    pub join_time: i64,
    pub updated_time: i64,
}

#[account]
pub struct PendingRewardWithdrawl {
    pub pending_reward_withdrawl_bump: u8,
    pub pending_rewards: u128,
    pub claimable_at: i64,
}

#[account]
pub struct PendingWithdrawl {
    pub pending_withdrawl_bump: u8,
    pub pending_tokens: u64,
    pub claimable_at: i64,
}

// 2. Add some useful constants for sizing propeties.
const BOOL_LENGTH: usize = 8;
const BUMP_LENGTH: usize = 8;
const PUBLIC_KEY_LENGTH: usize = 32;
const DISCRIMINATOR_LENGTH: usize = 8;
const STAKING_AMOUNT_LENGTH: usize = 128;
const TIME_EPOCH_LENGTH: usize = 64;
const APR_KEY_LENGTH: usize = 32;
const TOTAL_SLOT_LENGTH: usize = 8;

impl RuinStaking {
    pub const LEN: usize = DISCRIMINATOR_LENGTH
        + PUBLIC_KEY_LENGTH * 4
        + STAKING_AMOUNT_LENGTH
        + BOOL_LENGTH
        + BUMP_LENGTH;
}

impl RuinStakingTerm {
    pub const LEN: usize = DISCRIMINATOR_LENGTH
        + PUBLIC_KEY_LENGTH
        + TIME_EPOCH_LENGTH * 6
        + APR_KEY_LENGTH
        + TOTAL_SLOT_LENGTH
        + BUMP_LENGTH;
}

impl UserStaked {
    pub const LEN: usize = DISCRIMINATOR_LENGTH + BUMP_LENGTH + TIME_EPOCH_LENGTH * 3;
}

impl PendingWithdrawl {
    pub const LEN: usize = DISCRIMINATOR_LENGTH + BUMP_LENGTH + TIME_EPOCH_LENGTH * 2;
}

impl PendingRewardWithdrawl {
    pub const LEN: usize =
        DISCRIMINATOR_LENGTH + BUMP_LENGTH + TIME_EPOCH_LENGTH + STAKING_AMOUNT_LENGTH;
}
