use crate::error::*;
use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};


#[derive(Accounts)]
#[instruction(minimum_staking_amount: u64, maximum_staking_amount: u64, lock_duration: i64)]
pub struct InitializeStakingInfo<'info> {
    #[account(
        init,
        seeds = [
            b"staking",
            ruin_staking_token.key().as_ref(),
            ruin_staking_admin.key().as_ref(),
            [lock_duration as u8].as_ref()
        ],
        space = RuinStaking::LEN,
        payer = ruin_staking_admin, 
        bump
    )]
    pub ruin_staking: Box<Account<'info, RuinStaking>>,

    #[account(
        init,
        seeds = [
            b"staking_term",
            ruin_staking_token.key().as_ref(),
            ruin_staking_admin.key().as_ref(),
            [lock_duration as u8].as_ref()
        ],
        space = RuinStakingTerm::LEN,
        payer = ruin_staking_admin,
        bump
    )]
    pub ruin_staking_term: Box<Account<'info, RuinStakingTerm>>,

    #[account(mut)]
    pub ruin_staking_admin: Signer<'info>,

    pub ruin_staking_token: Account<'info, Mint>,

    #[account( 
        init,
        payer = ruin_staking_admin,
        token::mint = ruin_staking_token,
        token::authority = ruin_staking,
    )]
    pub ruin_staking_treasury: Account<'info, TokenAccount>,

    #[account( 
        init,
        seeds = [
            b"distributor",
            ruin_staking_token.key().as_ref(),
            ruin_staking_admin.key().as_ref(),
            [lock_duration as u8].as_ref()
        ],
        payer = ruin_staking_admin,
        bump,
        token::mint = ruin_staking_token,
        token::authority = ruin_staking,
    )]
    pub ruin_staking_distributor: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimPendingReward<'info> {
    #[account(
        seeds = [
            b"staking",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            [ruin_staking_term.lock_duration as u8].as_ref()
        ],
        bump = ruin_staking.ruin_staking_bump
    )]
    pub ruin_staking: Account<'info, RuinStaking>,

    #[account(
        seeds = [
            b"staking_term",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            [ruin_staking_term.lock_duration as u8].as_ref()
        ],
        bump = ruin_staking_term.ruin_staking_term_bump,
        has_one = ruin_staking @ProgramErrorCode::InvalidStakingTerms
    )]
    pub ruin_staking_term: Account<'info, RuinStakingTerm>,

    #[account(
        mut,
        seeds = [
            b"withdraw_reward",
            ruin_staking.key().as_ref(),
            ruin_staking_term.key().as_ref(),
            investor.key().as_ref()
        ],
        bump = user_pending_withdrawl.pending_reward_withdrawl_bump,  
    )]
    pub user_pending_withdrawl: Box<Account<'info, PendingRewardWithdrawl>>,

    #[account(
        mut,
        owner = Token::id(),
        seeds = [
            b"distributor",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            &[ruin_staking_term.lock_duration as u8]
        ],
        bump = ruin_staking.distributor_bump,  
    )]
    pub distributor_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        owner = Token::id(),
        constraint = investor_token_account.owner.key() == investor.key() @ProgramErrorCode::InvalidTokenAccountOwnership,
        constraint = investor_token_account.mint.key() == ruin_staking.staking_token.key() @ProgramErrorCode::UnsupportedStakingToken
    )]
    pub investor_token_account: Account<'info, TokenAccount>,
    pub investor: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [
            b"staking",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            [ruin_staking_term.lock_duration as u8].as_ref()
        ],
        bump = ruin_staking.ruin_staking_bump
    )]
    pub ruin_staking: Account<'info, RuinStaking>,

    #[account(
        seeds = [
            b"staking_term",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            [ruin_staking_term.lock_duration as u8].as_ref()
        ],
        bump = ruin_staking_term.ruin_staking_term_bump,
        has_one = ruin_staking @ProgramErrorCode::InvalidStakingTerms
    )]
    pub ruin_staking_term: Account<'info, RuinStakingTerm>,

    #[account(
        mut,
        seeds = [
            b"stake",
            ruin_staking.key().as_ref(),
            ruin_staking_term.key().as_ref(),
            investor.key().as_ref()
        ],
        bump = user_staked.user_staked_bump, 
    )]
    pub user_staked: Box<Account<'info, UserStaked>>,

    #[account(
        init,
        seeds = [
            b"withdraw",
            ruin_staking.key().as_ref(),
            ruin_staking_term.key().as_ref(),
            investor.key().as_ref()
        ],
        payer = investor,
        space = PendingRewardWithdrawl::LEN,
        bump, 
    )]
    pub user_pending_withdraw: Box<Account<'info, PendingWithdrawl>>,

    #[account(
        mut,
        constraint = treasury_token_account.key() == ruin_staking.treasury.key() @ProgramErrorCode::WrongTreasuryAddress,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        owner = Token::id(),
        constraint = investor_token_account.owner.key() == investor.key() @ProgramErrorCode::InvalidTokenAccountOwnership,
        constraint = investor_token_account.mint.key() == ruin_staking.staking_token.key() @ProgramErrorCode::UnsupportedStakingToken
    )]
    pub investor_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub investor: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Harvest<'info> {
    #[account(
        seeds = [
            b"staking",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            [ruin_staking_term.lock_duration as u8].as_ref()
        ],
        bump = ruin_staking.ruin_staking_bump
    )]
    pub ruin_staking: Account<'info, RuinStaking>,

    #[account(
        seeds = [
            b"staking_term",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            [ruin_staking_term.lock_duration as u8].as_ref()
        ],
        bump = ruin_staking_term.ruin_staking_term_bump,
        has_one = ruin_staking @ProgramErrorCode::InvalidStakingTerms
    )]
    pub ruin_staking_term: Account<'info, RuinStakingTerm>,

    #[account(
        mut,
        seeds = [
            b"stake",
            ruin_staking.key().as_ref(),
            ruin_staking_term.key().as_ref(),
            investor.key().as_ref()
        ],
        bump = user_staked.user_staked_bump, 
    )]
    pub user_staked: Box<Account<'info, UserStaked>>,

    #[account(
        mut,
        seeds = [
            b"withdraw_reward",
            ruin_staking.key().as_ref(),
            ruin_staking_term.key().as_ref(),
            investor.key().as_ref()
        ],
        bump = user_pending_withdrawl.pending_reward_withdrawl_bump,  
    )]
    pub user_pending_withdrawl: Box<Account<'info, PendingRewardWithdrawl>>,

    #[account(
        mut,
        constraint = treasury_token_account.key() == ruin_staking.treasury.key() @ProgramErrorCode::WrongTreasuryAddress,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,
    pub investor: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        seeds = [
            b"staking",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            [ruin_staking_term.lock_duration as u8].as_ref()
        ],
        bump = ruin_staking.ruin_staking_bump
    )]
    pub ruin_staking: Account<'info, RuinStaking>,

    #[account(
        seeds = [
            b"staking_term",
            ruin_staking.staking_token.key().as_ref(),
            ruin_staking.staking_admin.key().as_ref(),
            [ruin_staking_term.lock_duration as u8].as_ref()
        ],
        bump = ruin_staking_term.ruin_staking_term_bump,
        has_one = ruin_staking @ProgramErrorCode::InvalidStakingTerms
    )]
    pub ruin_staking_term: Account<'info, RuinStakingTerm>,

    #[account( 
        init,
        payer = investor,
        seeds = [
            b"stake",
            ruin_staking.key().as_ref(),
            ruin_staking_term.key().as_ref(),
            investor.key().as_ref()
        ],
        space = UserStaked::LEN, 
        bump, 
    )]
    pub user_staked: Box<Account<'info, UserStaked>>,

    #[account(
        init,
        payer = investor,
        seeds = [
            b"withdraw_reward",
            ruin_staking.key().as_ref(),
            ruin_staking_term.key().as_ref(),
            investor.key().as_ref()
        ],
        space = PendingWithdrawl::LEN,
        bump,
    )]
    pub user_pending_withdrawl: Box<Account<'info, PendingRewardWithdrawl>>,

    #[account(
        mut,
        owner = Token::id(),
        constraint = investor_token_account.owner.key() == investor.key() @ProgramErrorCode::InvalidTokenAccountOwnership,
        constraint = investor_token_account.mint.key() == ruin_staking.staking_token.key() @ProgramErrorCode::UnsupportedStakingToken
    )]
    pub investor_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        owner = Token::id(),
        constraint = treasury_token_account.owner == ruin_staking.key()
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub investor: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Stake<'info> {
    pub fn into_transfer_token_to_treasury(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.investor_token_account.to_account_info(),
            to: self.treasury_token_account.to_account_info(),
            authority: self.investor.to_account_info()
        };

        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

impl<'info> ClaimPendingReward<'info> {
    pub fn into_transfer_reward_to_investor(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.distributor_token_account.to_account_info(),
            to: self.investor_token_account.to_account_info(),
            authority: self.ruin_staking.to_account_info()
        };

        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

impl<'info> Withdraw<'info> {
}

