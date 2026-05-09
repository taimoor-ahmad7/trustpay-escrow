"use client";

import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useEffect, useMemo, useState } from "react";
import { DeadlineCountdown } from "@/components/DeadlineCountdown";
import { Spinner } from "@/components/Spinner";
import { StatusBadge } from "@/components/StatusBadge";
import { WalletButton } from "@/components/WalletButton";
import { getProgram, shortenAddress } from "@/lib/anchor";
import { EscrowJobView, fetchJobByClientAndId, fetchJobById, isDeadlinePassed } from "@/lib/jobs";
import styles from "./job-detail.module.css";

const TIMELINE = ["Open", "InProgress", "Submitted", "Completed"];

// JobDetailPage shows one escrow job and displays blockchain actions based on role and status.
export default function JobDetailPage() {
  const params = useParams<{ id: string }>();
  const wallet = useWallet();
  const { connection } = useConnection();
  const [job, setJob] = useState<EscrowJobView | null>(null);
  const [loading, setLoading] = useState(false);
  const [actionLoading, setActionLoading] = useState("");
  const [error, setError] = useState("");
  const [toast, setToast] = useState("");

  const viewerIsClient = useMemo(
    () => Boolean(wallet.publicKey && job?.client.equals(wallet.publicKey)),
    [job, wallet.publicKey]
  );

  const viewerIsFreelancer = useMemo(
    () => Boolean(wallet.publicKey && job?.freelancer.equals(wallet.publicKey)),
    [job, wallet.publicKey]
  );

  const refundAvailable = Boolean(job && viewerIsClient && isDeadlinePassed(job.deadline) && ["Open", "InProgress"].includes(job.status));

  // loadJob fetches this job from the program accounts on devnet.
  async function loadJob() {
    if (!wallet.connected) {
      return;
    }

    try {
      setLoading(true);
      setError("");
      const program = getProgram(connection, wallet);
      const directJob = wallet.publicKey ? await fetchJobByClientAndId(program, wallet.publicKey, params.id) : null;
      const foundJob = directJob ?? (await fetchJobById(program, params.id));
      setJob(foundJob);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not load job from devnet.");
    } finally {
      setLoading(false);
    }
  }

  // runAction sends an instruction transaction and refreshes the job after confirmation.
  async function runAction(label: string, action: () => Promise<string>) {
    try {
      setActionLoading(label);
      setError("");
      setToast("");
      const signature = await action();
      setToast(`${label} confirmed: ${signature.slice(0, 10)}...`);
      await loadJob();
    } catch (err) {
      setError(err instanceof Error ? err.message : `${label} failed.`);
    } finally {
      setActionLoading("");
    }
  }

  useEffect(() => {
    void loadJob();
  }, [wallet.connected, params.id]);

  if (!wallet.connected) {
    return (
      <main className="shell">
        <header className="topbar">
          <div className="brand">
            <h1>Job details</h1>
            <p>Connect Phantom to load escrow data from devnet.</p>
          </div>
          <WalletButton />
        </header>
        <p className={styles.notice}>Connect your wallet to view this escrow job.</p>
      </main>
    );
  }

  return (
    <main className="shell">
      <header className="topbar">
        <div className="brand">
          <h1>Job #{params.id}</h1>
          <p>Review status, deadline, wallet roles, and available actions.</p>
        </div>
        <WalletButton />
      </header>

      {error ? <p className="error">{error}</p> : null}

      {loading ? (
        <div className={styles.loading}>
          <Spinner />
          <span>Loading job from Solana devnet...</span>
        </div>
      ) : null}

      {!loading && !job ? (
        <section className={styles.panel}>
          <p className={styles.notice}>This job was not found. Older completed jobs created before the history update may already be closed.</p>
          <Link className="button secondary" href="/">
            Back home
          </Link>
        </section>
      ) : null}

      {job ? (
        <div className={styles.layout}>
          <section className={styles.panel}>
            <div className={styles.titleRow}>
              <p className={styles.description}>{job.description}</p>
              <StatusBadge status={job.status} />
            </div>

            <div className={styles.details}>
              <div className={styles.detailRow}>
                <span className={styles.label}>Payment amount</span>
                <strong>{job.amountSol} SOL</strong>
              </div>
              <div className={styles.detailRow}>
                <span className={styles.label}>Deadline</span>
                <span>
                  {new Date(job.deadline * 1000).toLocaleString()} - <DeadlineCountdown deadline={job.deadline} />
                </span>
              </div>
              <div className={styles.detailRow}>
                <span className={styles.label}>Client</span>
                <span className={styles.value}>{job.client.toBase58()}</span>
              </div>
              <div className={styles.detailRow}>
                <span className={styles.label}>Freelancer</span>
                <span className={styles.value}>{job.freelancer.toBase58()}</span>
              </div>
              <div className={styles.detailRow}>
                <span className={styles.label}>Escrow PDA</span>
                <span className={styles.value}>{job.address.toBase58()}</span>
              </div>
            </div>

            <div className={styles.timeline}>
              {TIMELINE.map((status) => {
                const active = status === job.status || TIMELINE.indexOf(status) <= TIMELINE.indexOf(job.status);
                return (
                  <div className={styles.timelineItem} key={status}>
                    <span className={`${styles.dot} ${active ? styles.dotActive : ""}`} />
                    <span>{status}</span>
                  </div>
                );
              })}
              {job.status === "Disputed" ? (
                <div className={styles.timelineItem}>
                  <span className={`${styles.dot} ${styles.dotActive}`} />
                  <span>Disputed</span>
                </div>
              ) : null}
              {job.status === "Refunded" ? (
                <div className={styles.timelineItem}>
                  <span className={`${styles.dot} ${styles.dotActive}`} />
                  <span>Refunded</span>
                </div>
              ) : null}
            </div>
          </section>

          <aside className={styles.actions}>
            <h2>Actions</h2>
            <p className={styles.notice}>
              Viewing as {wallet.publicKey ? shortenAddress(wallet.publicKey.toBase58()) : "unknown wallet"}
            </p>

            <div className={styles.actionList}>
              {viewerIsFreelancer && job.status === "Open" ? (
                <button
                  className="button"
                  disabled={Boolean(actionLoading)}
                  onClick={() =>
                    runAction("Accept job", async () => {
                      const program = getProgram(connection, wallet);
                      return program.methods
                        .acceptJob()
                        .accounts({ escrowJob: job.address, freelancer: wallet.publicKey! })
                        .rpc();
                    })
                  }
                  type="button"
                >
                  {actionLoading === "Accept job" ? <Spinner /> : null}
                  Accept Job
                </button>
              ) : null}

              {viewerIsFreelancer && job.status === "InProgress" ? (
                <button
                  className="button"
                  disabled={Boolean(actionLoading)}
                  onClick={() =>
                    runAction("Submit work", async () => {
                      const program = getProgram(connection, wallet);
                      return program.methods
                        .submitWork()
                        .accounts({ escrowJob: job.address, freelancer: wallet.publicKey! })
                        .rpc();
                    })
                  }
                  type="button"
                >
                  {actionLoading === "Submit work" ? <Spinner /> : null}
                  Submit Work
                </button>
              ) : null}

              {viewerIsClient && job.status === "Submitted" ? (
                <button
                  className="button"
                  disabled={Boolean(actionLoading)}
                  onClick={() =>
                    runAction("Approve release", async () => {
                      const program = getProgram(connection, wallet);
                      return program.methods
                        .approveRelease()
                        .accounts({ escrowJob: job.address, client: wallet.publicKey!, freelancer: job.freelancer })
                        .rpc();
                    })
                  }
                  type="button"
                >
                  {actionLoading === "Approve release" ? <Spinner /> : null}
                  Approve & Release Payment
                </button>
              ) : null}

              {refundAvailable ? (
                <button
                  className="button danger"
                  disabled={Boolean(actionLoading)}
                  onClick={() =>
                    runAction("Claim refund", async () => {
                      const program = getProgram(connection, wallet);
                      return program.methods
                        .claimRefund()
                        .accounts({ escrowJob: job.address, client: wallet.publicKey! })
                        .rpc();
                    })
                  }
                  type="button"
                >
                  {actionLoading === "Claim refund" ? <Spinner /> : null}
                  Claim Refund
                </button>
              ) : null}

              {(viewerIsClient || viewerIsFreelancer) && job.status === "Submitted" ? (
                <button
                  className="button secondary"
                  disabled={Boolean(actionLoading)}
                  onClick={() =>
                    runAction("Raise dispute", async () => {
                      const program = getProgram(connection, wallet);
                      return program.methods
                        .raiseDispute()
                        .accounts({ escrowJob: job.address, caller: wallet.publicKey! })
                        .rpc();
                    })
                  }
                  type="button"
                >
                  {actionLoading === "Raise dispute" ? <Spinner /> : null}
                  Raise Dispute
                </button>
              ) : null}

              {!viewerIsClient && !viewerIsFreelancer ? <p className={styles.notice}>This wallet is not part of this job.</p> : null}
              {viewerIsClient && !viewerIsFreelancer && job.status === "Open" ? <p className={styles.notice}>Waiting for the freelancer to accept.</p> : null}
              {viewerIsClient && !viewerIsFreelancer && job.status === "InProgress" ? <p className={styles.notice}>Waiting for the freelancer to submit work.</p> : null}
              {viewerIsFreelancer && !viewerIsClient && job.status === "Submitted" ? <p className={styles.notice}>Waiting for client approval.</p> : null}
              {job.status === "Completed" ? <p className={styles.notice}>Payment released. This job is saved as completed history.</p> : null}
              {job.status === "Refunded" ? <p className={styles.notice}>Payment refunded. This job is saved as refund history.</p> : null}
            </div>

            <Link className="button secondary" href="/">
              Back home
            </Link>
          </aside>
        </div>
      ) : null}

      {toast ? <div className="toast">{toast}</div> : null}
    </main>
  );
}
