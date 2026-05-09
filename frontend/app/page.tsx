"use client";

import Link from "next/link";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { useEffect, useMemo, useState } from "react";
import { JobCard } from "@/components/JobCard";
import { Spinner } from "@/components/Spinner";
import { WalletButton } from "@/components/WalletButton";
import { getProgram } from "@/lib/anchor";
import { EscrowJobView, fetchJobsForWallet } from "@/lib/jobs";
import styles from "./page.module.css";

// Home shows jobs where the connected wallet is either the client or freelancer.
export default function HomePage() {
  const wallet = useWallet();
  const { connection } = useConnection();
  const [postedJobs, setPostedJobs] = useState<EscrowJobView[]>([]);
  const [assignedJobs, setAssignedJobs] = useState<EscrowJobView[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const canLoadJobs = useMemo(() => Boolean(wallet.connected && wallet.publicKey), [wallet.connected, wallet.publicKey]);
  const allJobs = useMemo(() => {
    const unique = new Map<string, EscrowJobView>();
    [...postedJobs, ...assignedJobs].forEach((job) => unique.set(job.address.toBase58(), job));
    return Array.from(unique.values());
  }, [assignedJobs, postedJobs]);
  const activeJobs = useMemo(() => allJobs.filter((job) => !["Completed", "Refunded"].includes(job.status)).length, [allJobs]);
  const completedJobs = useMemo(() => allJobs.filter((job) => job.status === "Completed").length, [allJobs]);
  const totalSol = useMemo(
    () => allJobs.reduce((sum, job) => sum + Number(job.amountSol), 0).toFixed(3),
    [allJobs]
  );

  // loadJobs fetches all escrow accounts from devnet and filters them for the connected wallet.
  async function loadJobs() {
    if (!wallet.publicKey) {
      setPostedJobs([]);
      setAssignedJobs([]);
      return;
    }

    try {
      setLoading(true);
      setError("");
      const program = getProgram(connection, wallet);
      const jobs = await fetchJobsForWallet(program, wallet.publicKey);
      setPostedJobs(jobs.posted);
      setAssignedJobs(jobs.assigned);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not load jobs from devnet.");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    if (canLoadJobs) {
      void loadJobs();
    } else {
      setPostedJobs([]);
      setAssignedJobs([]);
    }
  }, [canLoadJobs]);

  return (
    <main className="shell">
      <header className="topbar">
        <div className="brand">
          <h1>Trustless Freelance Escrow</h1>
          <p>Solana devnet escrow for client and freelancer payments.</p>
        </div>
        <WalletButton />
      </header>

      <section className={styles.hero}>
        <div className={styles.heroText}>
          <span className={styles.eyebrow}>Solana devnet escrow</span>
          <h2>Freelance payments that move only when the work does.</h2>
          <p>Lock SOL, track progress, submit work, and release funds from one focused dashboard.</p>
        </div>
        <div className={styles.heroVisual} aria-hidden="true">
          <span className={styles.flowNode}>Client</span>
          <span className={styles.flowRail}>
            <span className={styles.flowPulse} />
          </span>
          <span className={styles.flowNode}>Escrow</span>
          <span className={styles.flowRail}>
            <span className={styles.flowPulse} />
          </span>
          <span className={styles.flowNode}>Freelancer</span>
        </div>
      </section>

      <section className={styles.stats}>
        <div className={styles.stat}>
          <span className={styles.statLabel}>Active jobs</span>
          <strong>{wallet.connected ? activeJobs : "--"}</strong>
        </div>
        <div className={styles.stat}>
          <span className={styles.statLabel}>Completed</span>
          <strong>{wallet.connected ? completedJobs : "--"}</strong>
        </div>
        <div className={styles.stat}>
          <span className={styles.statLabel}>Tracked SOL</span>
          <strong>{wallet.connected ? totalSol : "--"}</strong>
        </div>
        <Link className={`${styles.createCard} button`} href="/create">
          Create new job
        </Link>
      </section>

      {error ? <p className="error">{error}</p> : null}

      {loading ? (
        <div className={styles.loading}>
          <Spinner />
          <span>Loading jobs from Solana devnet...</span>
        </div>
      ) : null}

      {!wallet.connected ? <p className={styles.empty}>Connect Phantom to load your escrow jobs.</p> : null}

      <div className={styles.grid}>
        <section className={styles.section}>
          <div className={styles.sectionHeader}>
            <h3>Jobs I posted (as client)</h3>
            <span>{postedJobs.length}</span>
          </div>
          <div className={styles.jobs}>
            {postedJobs.length > 0 ? postedJobs.map((job) => <JobCard job={job} key={job.address.toBase58()} />) : null}
            {wallet.connected && !loading && postedJobs.length === 0 ? <p className={styles.empty}>No client jobs found.</p> : null}
          </div>
        </section>

        <section className={styles.section}>
          <div className={styles.sectionHeader}>
            <h3>Jobs assigned to me (as freelancer)</h3>
            <span>{assignedJobs.length}</span>
          </div>
          <div className={styles.jobs}>
            {assignedJobs.length > 0 ? assignedJobs.map((job) => <JobCard job={job} key={job.address.toBase58()} />) : null}
            {wallet.connected && !loading && assignedJobs.length === 0 ? <p className={styles.empty}>No freelancer jobs found.</p> : null}
          </div>
        </section>
      </div>
    </main>
  );
}
