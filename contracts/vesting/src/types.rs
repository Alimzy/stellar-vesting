use soroban_sdk::{contracttype, Address};

/// A single vesting schedule. One contract instance holds many of these,
/// each fully isolated from the others.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Schedule {
    /// Account that funded the schedule and may revoke it (if revocable).
    pub funder: Address,
    /// Account that receives vested tokens.
    pub beneficiary: Address,
    /// SEP-41 token being vested.
    pub token: Address,
    /// Total amount locked at creation.
    pub total: i128,
    /// Amount the beneficiary has already withdrawn.
    pub claimed: i128,
    /// Unix timestamp (seconds) at which linear vesting starts.
    pub start: u64,
    /// Nothing is claimable before this timestamp.
    pub cliff: u64,
    /// Timestamp at which everything is vested.
    pub end: u64,
    /// Whether the funder may revoke the unvested remainder.
    pub revocable: bool,
    /// Set once by `revoke`.
    pub revoked: bool,
    /// Amount that had vested at the moment of revocation (0 if not revoked).
    pub vested_at_revoke: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Number of schedules ever created; also the next schedule id.
    Count,
    /// A schedule, keyed by id.
    Schedule(u64),
}
