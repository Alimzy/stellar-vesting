#!/usr/bin/env bash
# Creates the starter issue backlog with the gh CLI.
#   DRY_RUN=1 bash scripts/create-issues.sh   # list titles only
#   bash scripts/create-issues.sh             # create labels + issues
# Works from any directory: it moves to the repository root itself.
set -euo pipefail
cd "$(dirname "$0")/.."

DRY_RUN="${DRY_RUN:-0}"

if [ "$DRY_RUN" != "1" ]; then
  command -v gh >/dev/null || { echo "gh CLI not found" >&2; exit 1; }
  gh auth status >/dev/null 2>&1 || { echo "run: gh auth login" >&2; exit 1; }

  label() { gh label create "$1" --color "$2" --description "$3" --force >/dev/null; }
  label trivial          "c2e0c6" "A few lines or docs; under an hour"
  label medium           "fbca04" "Self-contained change with tests; a few hours"
  label high             "d93f0b" "Touches contract logic or security; discuss design first"
  label ci               "0e8a16" "Continuous integration"
  label security         "b60205" "Security-relevant"
  label testing          "1d76db" "Tests and verification"
  label documentation    "0075ca" "Docs"
  label enhancement      "a2eeef" "New feature or request"
  label "good first issue" "7057ff" "Good for newcomers"
  label "help wanted"    "008672" "Extra attention is welcome"
fi

issue() { # issue <title> <labels> <body>
  if [ "$DRY_RUN" = "1" ]; then
    echo "[dry run] $1  [$2]"
  else
    gh issue create --title "$1" --label "$2" --body "$3"
  fi
}

issue "Verify the declared MSRV (1.84) and test it in CI" "medium,ci" \
"The workspace declares \`rust-version = \"1.84.0\"\` but CI only builds on 1.85.

**Do**
- Add a CI job that runs \`cargo check --locked --all-targets\` on Rust 1.84.0.
- Fix the lockfile or dependency versions if it fails, or raise the declared MSRV.

**Done when** the job is green and README/Cargo.toml agree on the MSRV."

issue "Enforce a dependency license allow-list with cargo-deny" "medium,ci,security" \
"\`deny.toml\` checks bans and sources but not licenses.

**Do**
- Run \`cargo deny list\` to see licenses in use, add a \`[licenses]\` allow-list, and add \`licenses\` to the CI \`deny\` job.

**Done when** CI fails on a dependency with a license outside the list."

issue "Snapshot-test event payloads" "medium,testing" \
"\`lifecycle_emits_events\` only checks that events exist. Indexers depend on exact topics and data.

**Do**
- Assert topics and data for \`ScheduleCreated\`, \`Claimed\`, \`Revoked\` (use \`env.events().all()\`).

**Done when** changing a field name or topic order breaks a test."

issue "Test that a failing token transfer leaves no partial state" "medium,testing,security" \
"State is written before the token call, relying on invocation rollback.

**Do**
- Add a test token whose \`transfer\` panics; call \`claim\`, \`revoke\` and \`create_schedule\` with it.
- Assert \`claimed\`, \`revoked\`, and \`schedule_count\` are unchanged afterwards.

**Done when** all three entrypoints are covered."

issue "Test restoring an archived schedule" "medium,testing" \
"Schedules in persistent storage can be archived after their TTL passes.

**Do**
- Advance the ledger past \`SCHEDULE_BUMP\`, assert access fails, restore the entry, then claim successfully.

**Done when** the test documents the restore flow."

issue "Add transfer_beneficiary(id, new_beneficiary)" "medium,enhancement" \
"A lost beneficiary key currently strands the schedule.

**Do**
- Add an entrypoint requiring the *current* beneficiary's auth, updating \`beneficiary\`, extending TTL, and emitting an event.
- Tests: auth, old beneficiary loses claim rights, claimed amount is preserved.
- Update docs, \`FEATURE-STATUS.md\`, \`CHANGELOG.md\`.

**Done when** tests and docs land together."

issue "Guard against fee-on-transfer tokens with a balance-delta check" "high,security,testing" \
"\`docs/SECURITY-MODEL.md\` assumes standard tokens. A fee-on-transfer token would leave the contract under-collateralised.

**Do**
- Measure the contract balance before and after the deposit in \`create_schedule\` and reject if the delta differs from \`amount\`.
- Add a test with a fee-charging mock token. Discuss the new error code in the issue before coding.

**Done when** the mock token is rejected and normal tokens still pass."

issue "Fuzz the full contract with arbitrary call sequences" "high,testing,security,help wanted" \
"Property tests cover claims and revoke separately.

**Do**
- Add a model-based test (proptest state machine or cargo-fuzz) interleaving create, claim, revoke and time advances across several schedules, checking conservation: contract balance == sum of unclaimed owed amounts.

**Done when** it runs in CI within a reasonable time budget."

issue "Add a batch create_schedules entrypoint" "high,enhancement" \
"Funding many beneficiaries today costs one transaction each.

**Do**
- Propose the signature in the issue first (shared funder and token, vec of per-beneficiary params).
- Implement with a single token transfer, tests, events, and docs.

**Done when** gas/footprint is measurably better than N single calls."

issue "Add a TypeScript example using @stellar/stellar-sdk" "medium,documentation,good first issue" \
"**Do**
- Add \`examples/ts/\` that creates a schedule, reads \`claimable\`, and claims on testnet.
- Document usage in the README.

**Done when** a newcomer can run it with only a funded testnet key."

issue "Fail CI when the optimized WASM exceeds a size budget" "trivial,ci,good first issue" \
"The \`build-wasm\` job prints size but never fails.

**Do**
- Pick a budget (suggest current size + 20%) and fail the job if exceeded. Note the budget in CONTRIBUTING.md.

**Done when** an oversized build turns CI red."

issue "Add worked rounding examples to docs/ARCHITECTURE.md" "trivial,documentation,good first issue" \
"Add two or three numeric examples showing floor rounding with awkward totals and durations (for example 10 tokens over 3 seconds) and why the contract never over-pays.

**Done when** the examples match \`math::tests\`."
