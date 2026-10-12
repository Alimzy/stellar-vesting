import {
  rpc,
  Contract,
  nativeToScVal,
  scValToNative,
  TransactionBuilder,
  Keypair,
  Networks,
  Address,
  Account,
} from "@stellar/stellar-sdk";

// Configuration defaults
const RPC_URL = process.env.SOROBAN_RPC_URL || "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE = Networks.TESTNET;
const CONTRACT_ID =
  process.env.VESTING_CONTRACT_ID || "CBF4ZT3QNPAWFMP5A434SS2JPLY3KWCRSP4PCDZNPD6HEOLAMODZSJVL";
const TOKEN_ID =
  process.env.TOKEN_ID || "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"; // Native XLM

const server = new rpc.Server(RPC_URL);
const contract = new Contract(CONTRACT_ID);

async function ensureFunded(name: string, secretKey?: string): Promise<Keypair> {
  if (secretKey) {
    const kp = Keypair.fromSecret(secretKey);
    console.log(`Using provided ${name}: ${kp.publicKey()}`);
    return kp;
  }

  const kp = Keypair.random();
  console.log(`Generated new ${name}: ${kp.publicKey()}`);
  console.log(`Funding ${name} via Testnet Friendbot...`);
  const res = await fetch(`https://friendbot.stellar.org?addr=${encodeURIComponent(kp.publicKey())}`);
  if (!res.ok) {
    throw new Error(`Friendbot HTTP error ${res.status}: ${await res.text()}`);
  }

  // Wait a few seconds for Friendbot ledger ingestion
  let funded = false;
  for (let i = 0; i < 10; i++) {
    try {
      await server.getAccount(kp.publicKey());
      funded = true;
      break;
    } catch {
      await new Promise((r) => setTimeout(r, 1500));
    }
  }

  if (!funded) {
    throw new Error(`Failed to fund ${name} (${kp.publicKey()}) via Friendbot`);
  }
  console.log(`Successfully funded ${name}.`);
  return kp;
}

async function invokeContract(signer: Keypair, op: ReturnType<Contract["call"]>): Promise<any> {
  const account = await server.getAccount(signer.publicKey());
  const tx = new TransactionBuilder(account, {
    fee: "100000",
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(op)
    .setTimeout(60)
    .build();

  const prepared = await server.prepareTransaction(tx);
  prepared.sign(signer);

  const sent = await server.sendTransaction(prepared);
  if (sent.status === "ERROR") {
    throw new Error(`Transaction submission error: ${JSON.stringify(sent.errorResult)}`);
  }

  console.log(`  Submitted tx hash: ${sent.hash}`);
  let txRes = await server.getTransaction(sent.hash);
  while (txRes.status === rpc.Api.GetTransactionStatus.NOT_FOUND) {
    await new Promise((r) => setTimeout(r, 1000));
    txRes = await server.getTransaction(sent.hash);
  }

  if (txRes.status === rpc.Api.GetTransactionStatus.SUCCESS) {
    return txRes.returnValue ? scValToNative(txRes.returnValue) : null;
  }

  throw new Error(`Transaction failed with status: ${txRes.status}`);
}

async function getClaimable(scheduleId: bigint, sourceAddress: string): Promise<bigint> {
  const dummyAccount = new Account(sourceAddress, "0");
  const tx = new TransactionBuilder(dummyAccount, {
    fee: "100",
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call("claimable", nativeToScVal(scheduleId, { type: "u64" })))
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) {
    throw new Error(`Simulation failed: ${sim.error}`);
  }
  if (!sim.result) {
    throw new Error("No simulation result returned for claimable");
  }
  return scValToNative(sim.result.retval) as bigint;
}

function stroopsToXlm(stroops: bigint): string {
  const whole = stroops / 10_000_000n;
  const fraction = (stroops % 10_000_000n).toString().padStart(7, "0");
  return `${whole}.${fraction} XLM`;
}

async function main() {
  console.log("=== Stellar Vesting TypeScript SDK Example ===");
  console.log(`Network:     Testnet`);
  console.log(`RPC URL:     ${RPC_URL}`);
  console.log(`Contract:    ${CONTRACT_ID}`);
  console.log(`Token:       ${TOKEN_ID}`);
  console.log("==============================================");

  const funderSecret = process.env.STELLAR_SECRET_KEY || process.env.FUNDER_SECRET_KEY;
  const beneficiarySecret = process.env.BENEFICIARY_SECRET_KEY;

  const funder = await ensureFunded("funder", funderSecret);
  const beneficiary = await ensureFunded("beneficiary", beneficiarySecret);

  const now = Math.floor(Date.now() / 1000);
  const start = BigInt(now - 60); // schedule started 60s ago
  const cliff = BigInt(now - 30); // cliff passed 30s ago
  const end = BigInt(now + 60); // schedule finishes in 60s
  const amount = 10_000_000n; // 1 XLM (10,000,000 stroops)
  const revocable = true;

  console.log("\n1. Creating vesting schedule...");
  console.log(`  Amount:    ${stroopsToXlm(amount)} (${amount} stroops)`);
  console.log(`  Duration:  start=${start}, cliff=${cliff}, end=${end}`);
  console.log(`  Revocable: ${revocable}`);

  const scheduleId = (await invokeContract(
    funder,
    contract.call(
      "create_schedule",
      new Address(funder.publicKey()).toScVal(),
      new Address(beneficiary.publicKey()).toScVal(),
      new Address(TOKEN_ID).toScVal(),
      nativeToScVal(amount, { type: "i128" }),
      nativeToScVal(start, { type: "u64" }),
      nativeToScVal(cliff, { type: "u64" }),
      nativeToScVal(end, { type: "u64" }),
      nativeToScVal(revocable, { type: "bool" })
    )
  )) as bigint;

  console.log(`  Created schedule ID: ${scheduleId}`);

  console.log("\n2. Checking claimable amount...");
  const claimableBefore = await getClaimable(scheduleId, funder.publicKey());
  console.log(`  Claimable: ${stroopsToXlm(claimableBefore)} (${claimableBefore} stroops)`);

  if (claimableBefore <= 0n) {
    console.log("  Nothing claimable yet. Waiting 2 seconds for vesting progression...");
    await new Promise((r) => setTimeout(r, 2000));
  }

  console.log("\n3. Beneficiary claiming vested tokens...");
  const claimedAmount = (await invokeContract(
    beneficiary,
    contract.call("claim", nativeToScVal(scheduleId, { type: "u64" }))
  )) as bigint;
  console.log(`  Successfully claimed: ${stroopsToXlm(claimedAmount)} (${claimedAmount} stroops)`);

  console.log("\n4. Checking remaining claimable amount...");
  const claimableAfter = await getClaimable(scheduleId, funder.publicKey());
  console.log(`  Remaining claimable: ${stroopsToXlm(claimableAfter)} (${claimableAfter} stroops)`);

  console.log("\n=== Demo Complete! ===");
}

main().catch((err) => {
  console.error("\nError running example:", err);
  process.exit(1);
});
