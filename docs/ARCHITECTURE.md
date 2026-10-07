# Architecture

## Goals

1. Hold tokens for many beneficiaries in one contract without any schedule being able to
   affect another.
2. Make the money rules small enough to verify by reading: one pure function decides how much
   has vested, and everything else is bookkeeping around it.
3. Never lose a schedule to storage expiry through neglect.

## Data model

```
Instance storage                      Persistent storage
----------------                      ------------------
DataKey::Count  -> u64                DataKey::Schedule(id) -> Schedule
```

`Count` is the next schedule id. Ids are sequential, start at 0 and are never reused.

`Schedule` fields: `funder`, `beneficiary`, `token`, `total`, `claimed`, `start`, `cliff`,
`end`, `revocable`, `revoked`, `vested_at_revoke`.

Schedules live in **persistent** storage because each is independent state that must outlive
the shared instance entry; the counter is tiny and always needed, so it lives in **instance**
storage.

## Vesting math (`math.rs`)

```
vested(total, start, cliff, end, now) =
    0                                     if now < cliff
    total                                 if now >= end
    floor(total * (now - start) / (end - start))   otherwise
```

Properties (checked by unit and property tests):

- `0 <= vested <= total`
- non-decreasing in `now`
- jumps from 0 to the linear amount at the cliff, not to `total`
- multiplication uses `checked_mul`; overflow returns `Error::MathOverflow`
- rounding is always down, so the contract can never owe more than it holds

Worked example, 1,000 tokens, `start=0`, `cliff=25d`, `end=100d`:

| Time | Vested |
|---|---|
| day 24 | 0 |
| day 25 | 250 |
| day 50 | 500 |
| day 100 and later | 1,000 |

## State machine of a schedule

```mermaid
stateDiagram-v2
    [*] --> Active: create_schedule
    Active --> Active: claim (partial)
    Active --> Revoked: revoke (revocable only)
    Active --> Drained: claim when fully vested
    Revoked --> Drained: claim vested remainder
```

After revocation `vested_at_revoke` is frozen. `vested_amount` returns it forever, so time
passing no longer vests anything.

## Flows

**create_schedule**: auth funder, validate, write schedule, bump counter, extend TTL,
*then* transfer `amount` from funder to the contract, emit `ScheduleCreated`.

**claim**: load, auth beneficiary, compute `vested - claimed`, reject if zero, write
`claimed`, extend TTL, *then* transfer to beneficiary, emit `Claimed`.

**revoke**: load, auth funder, require revocable and not already revoked, compute vested,
freeze it, extend TTL, *then* refund `total - vested` to funder, emit `Revoked`.

State is always written before the token call (checks, effects, interactions). If the token
call fails, the whole invocation reverts, so there is no partial state.

## Storage lifetime (TTL) policy

| Entry | Extended by | Bumped to | When fewer than |
|---|---|---|---|
| Instance | `create_schedule`, `claim`, `revoke`, `bump` | ~30 days | ~29 days remain |
| `Schedule(id)` | `create_schedule` (own id), `claim`, `revoke`, `bump` | ~90 days | ~89 days remain |

Read-only views do not extend TTL, so they stay free of write cost in simulation.
Because every state-changing entrypoint and the permissionless `bump` extend **both** entries,
anyone interested in a schedule (beneficiary, funder, an indexer) can keep it alive.
Constants live in `storage.rs`; the test
`every_state_changing_entrypoint_extends_instance_and_schedule_ttl` pins the behaviour.

Soroban archives (does not delete) expired persistent entries, and they can be restored, but
restoring costs a transaction, so the policy above avoids needing it.

## Authorization

| Entrypoint | Required signer |
|---|---|
| `create_schedule` | funder (plus the token's own transfer auth, covered by the same signature) |
| `claim` | beneficiary |
| `revoke` | funder |
| `bump`, views | none |

There is no admin, owner, upgrade key or pause switch. The contract's behaviour cannot be
changed after deployment by anyone.

## Events

| Event | Topics | Data |
|---|---|---|
| `ScheduleCreated` | `id`, `beneficiary` | `funder`, `token`, `amount`, `start`, `cliff`, `end`, `revocable` |
| `Claimed` | `id`, `beneficiary` | `amount` |
| `Revoked` | `id`, `funder` | `refunded`, `vested` |

## Design decisions

- **Pull over push.** The beneficiary claims; no one needs to run a scheduler.
- **Per-schedule token.** Each schedule names its token, so one contract can vest several
  assets, and a misbehaving token can only affect schedules that use it.
- **No admin.** Removing privileged roles removes a whole class of risk. The funder's only
  power is over its own revocable schedules.
- **Pure math module.** Keeps the one piece of arithmetic that matters testable without a
  ledger environment.
