import { BN, Program } from "@coral-xyz/anchor";
import { PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { getEscrowPda } from "@/lib/anchor";

export type EscrowStatus =
  | "Open"
  | "InProgress"
  | "Submitted"
  | "Completed"
  | "Disputed"
  | "Refunded";

export type EscrowJobView = {
  address: PublicKey;
  jobId: string;
  client: PublicKey;
  freelancer: PublicKey;
  amountLamports: BN;
  amountSol: string;
  deadline: number;
  status: EscrowStatus;
  description: string;
};

const STATUS_LABELS: Record<string, EscrowStatus> = {
  open: "Open",
  inProgress: "InProgress",
  submitted: "Submitted",
  completed: "Completed",
  disputed: "Disputed",
  refunded: "Refunded"
};

// statusFromAnchor converts Anchor's enum shape into a simple display string.
export function statusFromAnchor(rawStatus: unknown): EscrowStatus {
  if (typeof rawStatus === "string") {
    return STATUS_LABELS[rawStatus] ?? (rawStatus as EscrowStatus);
  }

  if (rawStatus && typeof rawStatus === "object") {
    const key = Object.keys(rawStatus as Record<string, unknown>)[0];
    return STATUS_LABELS[key] ?? "Open";
  }

  return "Open";
}

// toJobView converts a raw Anchor account into data that React components can display.
function toJobView(raw: { publicKey: PublicKey; account: any }): EscrowJobView {
  const amountLamports = raw.account.amount as BN;
  const jobId = raw.account.jobId ?? raw.account.job_id;

  return {
    address: raw.publicKey,
    jobId: jobId.toString(),
    client: raw.account.client,
    freelancer: raw.account.freelancer,
    amountLamports,
    amountSol: (amountLamports.toNumber() / LAMPORTS_PER_SOL).toFixed(4),
    deadline: Number(raw.account.deadline.toString()),
    status: statusFromAnchor(raw.account.status),
    description: raw.account.description
  };
}

// fetchAllJobs loads every EscrowJob account owned by this program.
export async function fetchAllJobs(program: Program) {
  const accounts: { publicKey: PublicKey; account: any }[] = await (program.account as any).escrowJob.all();
  return accounts.map(toJobView);
}

// fetchJobsForWallet separates jobs where the wallet is the client from jobs where it is the freelancer.
export async function fetchJobsForWallet(program: Program, wallet: PublicKey) {
  const jobs = await fetchAllJobs(program);

  return {
    posted: jobs.filter((job: EscrowJobView) => job.client.equals(wallet)),
    assigned: jobs.filter((job: EscrowJobView) => job.freelancer.equals(wallet))
  };
}

// fetchJobById finds one job using the human-readable job ID from the URL.
export async function fetchJobById(program: Program, jobId: string) {
  const jobs = await fetchAllJobs(program);
  return jobs.find((job: EscrowJobView) => job.jobId === jobId) ?? null;
}

// fetchJobByClientAndId derives the PDA directly when we know the client wallet and job ID.
export async function fetchJobByClientAndId(program: Program, client: PublicKey, jobId: string) {
  const jobIdBn = new BN(jobId);
  const escrowPda = getEscrowPda(client, jobIdBn);

  try {
    const account = await (program.account as any).escrowJob.fetch(escrowPda);
    return toJobView({ publicKey: escrowPda, account });
  } catch {
    return null;
  }
}

// isDeadlinePassed checks whether the current browser time is later than the job deadline.
export function isDeadlinePassed(deadline: number) {
  return Math.floor(Date.now() / 1000) > deadline;
}
