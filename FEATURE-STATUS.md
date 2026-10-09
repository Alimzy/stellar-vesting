# Feature status

"Verified" means a test or a command output demonstrates it. Anything not verified is listed as such.

| Feature | Implemented | Verified by |
|---|---|---|
| Create schedule, lock funds | Yes | `create_locks_funds_and_records_schedule` |
| Parameter validation | Yes | `invalid_parameters_are_rejected` |
| Cliff behaviour | Yes | `nothing_vests_before_the_cliff`, `at_the_cliff_the_linear_amount_unlocks` |
| Linear vesting and rounding | Yes | `math::tests::*` |
| Repeated claims bounded by total | Yes | `repeated_claims_never_pay_more_than_total`, property test |
| Revoke (refund unvested, keep vested) | Yes | `revoke_*` tests, `revoke_conserves_tokens` property test |
| Non-revocable and double-revoke guards | Yes | `cannot_revoke_twice_or_a_non_revocable_schedule` |
| Schedule isolation | Yes | `schedules_in_one_contract_are_isolated` |
| Correct signer per entrypoint | Yes | `required_signers_are_exactly_the_right_parties` |
| TTL extension on every state-changing entrypoint | Yes | `every_state_changing_entrypoint_extends_instance_and_schedule_ttl` |
| Overflow reported, not wrapped | Yes | `math::tests::overflow_is_reported_not_wrapped` |
| Events emitted | Yes | `lifecycle_emits_events` (presence only; payload snapshots are an open issue) |
| `cargo fmt`, `clippy -D warnings`, tests | Yes | Run locally on Rust 1.85 |
| Optimized WASM build | Yes | `stellar contract build` (16,590 bytes); the `build-wasm` CI job repeats it |
| Testnet deployment and demo | Yes | Deploy, create, claim, revoke, claim run on testnet; see README proof table and `docs/testnet-proof.md` |
| Declared MSRV (1.84) builds | Yes | Verified in CI (`msrv` job) on Rust 1.84.0 |
| Dependency license allow-list | Not enforced | Open issue |
| Third-party audit | No | Not audited |

## Known limits

- Tokens must behave like standard SEP-41 tokens. Fee-on-transfer or rebasing tokens break per-schedule accounting; see `docs/SECURITY-MODEL.md`.
- Schedules cannot be transferred to a new beneficiary yet (open issue).
