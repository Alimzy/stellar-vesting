use soroban_sdk::{contractevent, Address};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleCreated {
    #[topic]
    pub id: u64,
    #[topic]
    pub beneficiary: Address,
    pub funder: Address,
    pub token: Address,
    pub amount: i128,
    pub start: u64,
    pub cliff: u64,
    pub end: u64,
    pub revocable: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claimed {
    #[topic]
    pub id: u64,
    #[topic]
    pub beneficiary: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revoked {
    #[topic]
    pub id: u64,
    #[topic]
    pub funder: Address,
    /// Unvested tokens returned to the funder.
    pub refunded: i128,
    /// Tokens that had vested and stay claimable by the beneficiary.
    pub vested: i128,
}
