# Changelog

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- `transfer_beneficiary(id, new_beneficiary)` entrypoint requiring current beneficiary authorization and emitting `BeneficiaryTransferred` event (#6).
- Initial release: `create_schedule`, `claim`, `revoke`, `bump`, and read-only views.
- Events: `ScheduleCreated`, `Claimed`, `Revoked`, `BeneficiaryTransferred`.
- 23 tests including two property tests.
