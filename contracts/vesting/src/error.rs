use soroban_sdk::contracterror;

/// Every failure mode of the contract. Codes are part of the public ABI:
/// never renumber an existing variant, only append new ones.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// `amount` must be strictly positive.
    InvalidAmount = 1,
    /// Timestamps must satisfy `start <= cliff <= end` and `start < end`.
    InvalidSchedule = 2,
    /// No schedule exists with the given id.
    ScheduleNotFound = 3,
    /// Nothing has vested beyond what was already claimed.
    NothingToClaim = 4,
    /// The schedule was created with `revocable = false`.
    NotRevocable = 5,
    /// The schedule has already been revoked.
    AlreadyRevoked = 6,
    /// An intermediate calculation overflowed `i128`.
    MathOverflow = 7,
}
