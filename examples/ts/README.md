# TypeScript Example (@stellar/stellar-sdk)

An end-to-end example demonstrating how to interact with the Stellar Vesting smart contract on Testnet using [`@stellar/stellar-sdk`](https://www.npmjs.com/package/@stellar/stellar-sdk).

## What it does

1. **Loads or funds testnet identities**: Uses your provided secret key(s) or automatically generates and funds new testnet keypairs using Stellar Testnet Friendbot.
2. **Creates a vesting schedule**: Calls `create_schedule` with linear unlocking, a cliff period, and revocability.
3. **Reads claimable tokens**: Queries `claimable(schedule_id)` via read-only transaction simulation.
4. **Claims vested tokens**: Signs and submits `claim(schedule_id)` from the beneficiary account.
5. **Verifies remaining claimable**: Queries `claimable(schedule_id)` again to confirm the updated claimable balance.

## Prerequisites

- [Node.js](https://nodejs.org/) v18 or later.
- [npm](https://www.npmjs.com/) or [pnpm](https://pnpm.io/).

## Setup & Running

```bash
cd examples/ts
npm install
npm start
```

That's it! If no environment variables are set, the script will automatically create fresh testnet keypairs, fund them via Friendbot, deploy the schedule, and execute the claim.

### Using your own funded Testnet account

To use an existing funded Testnet secret key:

```bash
STELLAR_SECRET_KEY="S..." npm start
```

### Environment Variables

| Variable | Description | Default |
|---|---|---|
| `STELLAR_SECRET_KEY` / `FUNDER_SECRET_KEY` | Secret key for the funder account | Auto-generated & funded via Friendbot |
| `BENEFICIARY_SECRET_KEY` | Secret key for the beneficiary account | Auto-generated & funded via Friendbot |
| `VESTING_CONTRACT_ID` | Deployed Vesting Contract ID | `CBF4ZT3QNPAWFMP5A434SS2JPLY3KWCRSP4PCDZNPD6HEOLAMODZSJVL` (Testnet) |
| `TOKEN_ID` | SEP-41 Token contract address | `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` (Native XLM) |
| `SOROBAN_RPC_URL` | Soroban RPC endpoint | `https://soroban-testnet.stellar.org` |
