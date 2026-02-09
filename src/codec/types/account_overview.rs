use super::compute::delegation::Delegation;
use super::compute::commitment::Commitment;
use super::account_info::AccountInfo;
use super::token_conversion::token_conversion::TokenConversion;
use super::vesting::vesting::Vesting;

#[derive(uniffi::Record, Debug)]
pub struct AccountOverview {
    pub account_info: AccountInfo,
    pub commitment: Option<Commitment>,
    pub delegations: Vec<DelegationWithCommitment>,
    pub vesting: Vec<Vesting>,
    pub conversion: Option<TokenConversion>,
}

#[derive(uniffi::Record, Debug)]
pub struct DelegationWithCommitment {
    pub delegation: Delegation,
    pub commitment: Commitment,
}
