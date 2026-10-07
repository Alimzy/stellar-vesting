# Security model

This contract has **not** been audited. Treat it as reference-quality code.

## Assets at risk

Tokens held by the contract on behalf of schedules.

## Trust assumptions

1. **The token is a well-behaved SEP-41 token.** `transfer(from, to, amount)` moves exactly
   `amount`. Fee-on-transfer, rebasing or deflationary tokens would leave the contract holding
   less than `total` and the last claimer could fail. Use standard tokens (including native XLM
   through its Stellar Asset Contract).
2. **Ledger time is honest within normal bounds.** Vesting reads `ledger().timestamp()`.
   Validators agree on close times within a small tolerance, so schedules should not depend on
   second-level precision.
3. **The funder and beneficiary keep their keys.** There is no recovery path or admin override.

## What the contract guarantees

| Guarantee | Enforced by |
|---|---|
| The beneficiary never receives more than `total` | `claimed` only grows by `vested - claimed`; property test |
| The funder can only reclaim unvested tokens | `revoke` refunds `total - vested` and freezes `vested` |
| Only the beneficiary can claim; only the funder can revoke | `require_auth` on the stored addresses; signer test |
| Schedules cannot affect each other | Separate storage keys; isolation test |
| No partial state on failure | State written before token call; a failing token call reverts the invocation |
| Rounding never over-pays | Integer division rounds down |
| Overflow cannot wrap | `checked_mul`; release profile has `overflow-checks = true` |

## Known limitations

- **Non-standard tokens**: see assumption 1. There is no balance-delta check on deposit yet.
- **No beneficiary transfer**: a lost beneficiary key means the tokens are stranded
  (unless the schedule is revocable and unvested tokens remain).
- **Revocation is a trust decision**: a revocable schedule means the beneficiary trusts the
  funder not to revoke. Use `revocable = false` when the beneficiary needs a guarantee.
- **Archival**: a schedule left untouched past its TTL is archived and must be restored before
  use. Call `bump` periodically for multi-year schedules.

## Reporting

See [SECURITY.md](../SECURITY.md).
