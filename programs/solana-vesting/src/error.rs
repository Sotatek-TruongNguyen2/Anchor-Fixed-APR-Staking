use anchor_lang::prelude::*;

#[error_code]
pub enum ProgramErrorCode {
    #[msg("Staking not started or ended")]
    ExceedsCurrentStakingAmount,
    #[msg("Staking not started or ended")]
    StakingNotStartedOrEnded,
    #[msg("Maximum amount exceeds")]
    MaximumAmountExceeds,
    #[msg("Minimum amount not reached")]
    MinimumAmountNotReached,
    #[msg("Exceeds your current balance!")]
    ExceedsCurrentBalance,
    #[msg("Wrong treasury address!")]
    WrongTreasuryAddress,
    #[msg("Investor do not acquire this token account!")]
    InvalidTokenAccountOwnership,
    #[msg("Staking system did not support this token!")]
    UnsupportedStakingToken,
    #[msg("Token account with staking token not matched!")]
    TokenAccountNotMatched,
    #[msg("Apr must be greater than 0!")]
    InvalidStakingAPR,
    #[msg("End join time must greater than start join time!")]
    InvalidJoiningTime,
    #[msg("Staking term did not matched with Staking info!")]
    InvalidStakingTerms,
}
