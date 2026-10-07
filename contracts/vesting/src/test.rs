extern crate std;

use proptest::prelude::*;
use soroban_sdk::{
    testutils::{
        storage::Instance as _, storage::Persistent as _, Address as _, Events as _, Ledger,
    },
    token::{StellarAssetClient, TokenClient},
    Address, Env,
};

use crate::storage::{INSTANCE_BUMP, SCHEDULE_BUMP};
use crate::{Error, VestingContract, VestingContractClient};

const DAY: u64 = 86_400;
const MINT: i128 = 1_000_000_000;

struct Ctx {
    env: Env,
    contract: Address,
    token: Address,
    funder: Address,
    beneficiary: Address,
}

impl Ctx {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let token = env.register_stellar_asset_contract_v2(admin).address();
        let funder = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        StellarAssetClient::new(&env, &token).mint(&funder, &MINT);
        let contract = env.register(VestingContract, ());
        Ctx {
            env,
            contract,
            token,
            funder,
            beneficiary,
        }
    }

    fn client(&self) -> VestingContractClient<'_> {
        VestingContractClient::new(&self.env, &self.contract)
    }

    fn token(&self) -> TokenClient<'_> {
        TokenClient::new(&self.env, &self.token)
    }

    fn at(&self, timestamp: u64) {
        self.env.ledger().with_mut(|l| l.timestamp = timestamp);
    }

    /// 1_000 tokens over 100 days, 25-day cliff, revocable.
    fn standard(&self) -> u64 {
        self.client().create_schedule(
            &self.funder,
            &self.beneficiary,
            &self.token,
            &1_000,
            &0,
            &(25 * DAY),
            &(100 * DAY),
            &true,
        )
    }
}

#[test]
fn create_locks_funds_and_records_schedule() {
    let c = Ctx::new();
    let id = c.standard();

    assert_eq!(id, 0);
    assert_eq!(c.client().schedule_count(), 1);
    assert_eq!(c.token().balance(&c.funder), MINT - 1_000);
    assert_eq!(c.token().balance(&c.contract), 1_000);

    let s = c.client().get_schedule(&id);
    assert_eq!(s.total, 1_000);
    assert_eq!(s.claimed, 0);
    assert!(s.revocable && !s.revoked);
}

#[test]
fn nothing_vests_before_the_cliff() {
    let c = Ctx::new();
    let id = c.standard();
    c.at(25 * DAY - 1);

    assert_eq!(c.client().vested_amount(&id), 0);
    assert_eq!(c.client().try_claim(&id), Err(Ok(Error::NothingToClaim)));
}

#[test]
fn at_the_cliff_the_linear_amount_unlocks() {
    let c = Ctx::new();
    let id = c.standard();
    c.at(25 * DAY);

    assert_eq!(c.client().claimable(&id), 250);
    assert_eq!(c.client().claim(&id), 250);
    assert_eq!(c.token().balance(&c.beneficiary), 250);
}

#[test]
fn repeated_claims_never_pay_more_than_total() {
    let c = Ctx::new();
    let id = c.standard();

    c.at(50 * DAY);
    assert_eq!(c.client().claim(&id), 500);
    assert_eq!(c.client().try_claim(&id), Err(Ok(Error::NothingToClaim)));

    c.at(75 * DAY);
    assert_eq!(c.client().claim(&id), 250);

    c.at(1_000 * DAY);
    assert_eq!(c.client().claim(&id), 250);
    assert_eq!(c.token().balance(&c.beneficiary), 1_000);
    assert_eq!(c.token().balance(&c.contract), 0);
    assert_eq!(c.client().try_claim(&id), Err(Ok(Error::NothingToClaim)));
}

#[test]
fn revoke_refunds_unvested_and_keeps_vested_claimable() {
    let c = Ctx::new();
    let id = c.standard();
    c.at(40 * DAY);

    assert_eq!(c.client().revoke(&id), 600);
    assert_eq!(c.token().balance(&c.funder), MINT - 1_000 + 600);

    // Time passing after revocation does not vest anything more.
    c.at(1_000 * DAY);
    assert_eq!(c.client().vested_amount(&id), 400);
    assert_eq!(c.client().claim(&id), 400);
    assert_eq!(c.token().balance(&c.contract), 0);
}

#[test]
fn revoke_before_cliff_refunds_everything() {
    let c = Ctx::new();
    let id = c.standard();
    c.at(DAY);

    assert_eq!(c.client().revoke(&id), 1_000);
    assert_eq!(c.client().try_claim(&id), Err(Ok(Error::NothingToClaim)));
    assert_eq!(c.token().balance(&c.funder), MINT);
}

#[test]
fn revoke_after_partial_claim_does_not_double_pay() {
    let c = Ctx::new();
    let id = c.standard();
    c.at(50 * DAY);
    assert_eq!(c.client().claim(&id), 500);

    assert_eq!(c.client().revoke(&id), 500);
    assert_eq!(c.client().try_claim(&id), Err(Ok(Error::NothingToClaim)));
    assert_eq!(c.token().balance(&c.contract), 0);
}

#[test]
fn cannot_revoke_twice_or_a_non_revocable_schedule() {
    let c = Ctx::new();
    let id = c.standard();
    c.client().revoke(&id);
    assert_eq!(c.client().try_revoke(&id), Err(Ok(Error::AlreadyRevoked)));

    let locked = c.client().create_schedule(
        &c.funder,
        &c.beneficiary,
        &c.token,
        &500,
        &0,
        &0,
        &DAY,
        &false,
    );
    assert_eq!(c.client().try_revoke(&locked), Err(Ok(Error::NotRevocable)));
}

#[test]
fn invalid_parameters_are_rejected() {
    let c = Ctx::new();
    let try_create = |amount: i128, start: u64, cliff: u64, end: u64| {
        c.client().try_create_schedule(
            &c.funder,
            &c.beneficiary,
            &c.token,
            &amount,
            &start,
            &cliff,
            &end,
            &true,
        )
    };

    assert_eq!(try_create(0, 0, 0, 10), Err(Ok(Error::InvalidAmount)));
    assert_eq!(try_create(-5, 0, 0, 10), Err(Ok(Error::InvalidAmount)));
    assert_eq!(try_create(1, 10, 10, 10), Err(Ok(Error::InvalidSchedule)));
    assert_eq!(try_create(1, 10, 5, 20), Err(Ok(Error::InvalidSchedule)));
    assert_eq!(try_create(1, 0, 30, 20), Err(Ok(Error::InvalidSchedule)));
    // Nothing leaked: failed calls neither moved funds nor consumed ids.
    assert_eq!(c.client().schedule_count(), 0);
    assert_eq!(c.token().balance(&c.funder), MINT);
}

#[test]
fn unknown_schedule_ids_fail_cleanly() {
    let c = Ctx::new();
    assert_eq!(c.client().try_claim(&7), Err(Ok(Error::ScheduleNotFound)));
    assert_eq!(c.client().try_revoke(&7), Err(Ok(Error::ScheduleNotFound)));
    assert_eq!(c.client().try_bump(&7), Err(Ok(Error::ScheduleNotFound)));
    assert_eq!(
        c.client().try_get_schedule(&7),
        Err(Ok(Error::ScheduleNotFound))
    );
}

#[test]
fn schedules_in_one_contract_are_isolated() {
    let c = Ctx::new();
    let other = Address::generate(&c.env);
    let a = c.standard();
    let b = c.client().create_schedule(
        &c.funder,
        &other,
        &c.token,
        &4_000,
        &0,
        &0,
        &(40 * DAY),
        &false,
    );
    assert_eq!((a, b), (0, 1));

    c.at(30 * DAY);
    assert_eq!(c.client().claim(&b), 3_000);
    assert_eq!(c.client().claimable(&a), 300);
    assert_eq!(c.client().get_schedule(&a).claimed, 0);
    assert_eq!(c.token().balance(&other), 3_000);
    assert_eq!(c.token().balance(&c.contract), 1_000 + 4_000 - 3_000);
}

#[test]
fn required_signers_are_exactly_the_right_parties() {
    let c = Ctx::new();
    let id = c.standard();
    assert_eq!(c.env.auths()[0].0, c.funder);

    c.at(50 * DAY);
    c.client().claim(&id);
    let auths = c.env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, c.beneficiary);

    c.client().revoke(&id);
    let auths = c.env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, c.funder);
}

#[test]
fn every_state_changing_entrypoint_extends_instance_and_schedule_ttl() {
    let c = Ctx::new();
    let id = c.standard();
    let ttl = |c: &Ctx| {
        c.env.as_contract(&c.contract, || {
            (
                c.env.storage().instance().get_ttl(),
                c.env
                    .storage()
                    .persistent()
                    .get_ttl(&crate::types::DataKey::Schedule(id)),
            )
        })
    };
    let age = |c: &Ctx, ledgers: u32| {
        c.env.ledger().with_mut(|l| l.sequence_number += ledgers);
    };
    let (i0, s0) = ttl(&c);
    assert_eq!((i0, s0), (INSTANCE_BUMP, SCHEDULE_BUMP));

    // claim
    c.at(50 * DAY);
    age(&c, 20 * 17_280);
    c.client().claim(&id);
    assert_eq!(ttl(&c), (INSTANCE_BUMP, SCHEDULE_BUMP));

    // bump (permissionless)
    age(&c, 20 * 17_280);
    c.client().bump(&id);
    assert_eq!(ttl(&c), (INSTANCE_BUMP, SCHEDULE_BUMP));

    // revoke
    age(&c, 20 * 17_280);
    c.client().revoke(&id);
    assert_eq!(ttl(&c), (INSTANCE_BUMP, SCHEDULE_BUMP));

    // create_schedule on a fresh id also refreshes the shared instance entry
    age(&c, 20 * 17_280);
    c.standard();
    assert_eq!(ttl(&c).0, INSTANCE_BUMP);
}

#[test]
fn lifecycle_emits_events() {
    let c = Ctx::new();
    let id = c.standard();
    let after_create = c.env.events().all().events().len();
    assert!(after_create >= 1);

    c.at(50 * DAY);
    c.client().claim(&id);
    let after_claim = c.env.events().all().events().len();
    assert!(after_claim >= 1);
}

// ---------------------------------------------------------------- properties

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// For any valid schedule and any increasing sequence of claim times:
    /// payouts are never negative, the sum never exceeds the total, the
    /// contract never holds less than what is still owed, and a claim after
    /// the end pays out exactly the remainder.
    #[test]
    fn claims_never_exceed_total_and_always_drain_at_end(
        total in 1i128..1_000_000_000_000,
        start in 0u64..1_000_000,
        len in 1u64..10_000_000,
        cliff_pct in 0u64..=100,
        mut times in proptest::collection::vec(0u64..30_000_000, 1..8),
    ) {
        let end = start + len;
        let cliff = start + len * cliff_pct / 100;
        times.sort_unstable();

        let c = Ctx::new();
        StellarAssetClient::new(&c.env, &c.token).mint(&c.funder, &total);
        let id = c.client().create_schedule(
            &c.funder, &c.beneficiary, &c.token, &total, &start, &cliff, &end, &false,
        );

        let mut paid: i128 = 0;
        let mut last_vested: i128 = 0;
        for t in times {
            c.at(t);
            let v = c.client().vested_amount(&id);
            prop_assert!(v >= last_vested, "vesting went backwards");
            prop_assert!(v <= total);
            last_vested = v;

            match c.client().try_claim(&id) {
                Ok(Ok(amount)) => {
                    prop_assert!(amount > 0);
                    paid += amount;
                }
                Err(Ok(Error::NothingToClaim)) => {}
                other => prop_assert!(false, "unexpected result: {:?}", other),
            }
            prop_assert!(paid <= total);
            prop_assert_eq!(c.token().balance(&c.contract), total - paid);
        }

        c.at(end);
        let rest = c.client().try_claim(&id);
        if paid < total {
            prop_assert_eq!(rest, Ok(Ok(total - paid)));
        }
        prop_assert_eq!(c.token().balance(&c.beneficiary) , total);
        prop_assert_eq!(c.token().balance(&c.contract), 0);
    }

    /// Revoking at any moment conserves tokens: refunded + vested == total,
    /// and the beneficiary can still collect exactly the vested part.
    #[test]
    fn revoke_conserves_tokens(
        total in 1i128..1_000_000_000_000,
        len in 1u64..10_000_000,
        cliff_pct in 0u64..=100,
        revoke_at in 0u64..20_000_000,
    ) {
        let cliff = len * cliff_pct / 100;
        let c = Ctx::new();
        StellarAssetClient::new(&c.env, &c.token).mint(&c.funder, &total);
        let before = c.token().balance(&c.funder);
        let id = c.client().create_schedule(
            &c.funder, &c.beneficiary, &c.token, &total, &0, &cliff, &len, &true,
        );

        c.at(revoke_at);
        let refund = c.client().revoke(&id);
        let vested = c.client().vested_amount(&id);
        prop_assert_eq!(refund + vested, total);
        prop_assert_eq!(c.token().balance(&c.funder), before - total + refund);

        c.at(revoke_at + 100_000_000);
        if vested > 0 {
            prop_assert_eq!(c.client().claim(&id), vested);
        }
        prop_assert_eq!(c.token().balance(&c.contract), 0);
    }
}
