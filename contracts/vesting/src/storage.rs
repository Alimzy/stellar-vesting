use soroban_sdk::Env;

use crate::error::Error;
use crate::types::{DataKey, Schedule};

const DAY_IN_LEDGERS: u32 = 17_280;

/// Instance storage (the schedule counter) is bumped to ~30 days on every
/// state-changing call, whenever fewer than ~29 days remain.
pub const INSTANCE_BUMP: u32 = 30 * DAY_IN_LEDGERS;
pub const INSTANCE_THRESHOLD: u32 = INSTANCE_BUMP - DAY_IN_LEDGERS;

/// Each schedule is bumped to ~90 days on every touch, whenever fewer than
/// ~89 days remain. Anyone can also call `bump` to keep a long schedule alive.
pub const SCHEDULE_BUMP: u32 = 90 * DAY_IN_LEDGERS;
pub const SCHEDULE_THRESHOLD: u32 = SCHEDULE_BUMP - DAY_IN_LEDGERS;

pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

pub fn extend_schedule(env: &Env, id: u64) {
    env.storage().persistent().extend_ttl(
        &DataKey::Schedule(id),
        SCHEDULE_THRESHOLD,
        SCHEDULE_BUMP,
    );
}

pub fn count(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::Count).unwrap_or(0)
}

pub fn set_count(env: &Env, value: u64) {
    env.storage().instance().set(&DataKey::Count, &value);
}

pub fn load(env: &Env, id: u64) -> Result<Schedule, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Schedule(id))
        .ok_or(Error::ScheduleNotFound)
}

pub fn save(env: &Env, id: u64, schedule: &Schedule) {
    env.storage()
        .persistent()
        .set(&DataKey::Schedule(id), schedule);
}
