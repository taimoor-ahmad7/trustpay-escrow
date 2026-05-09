"use client";

import { BN } from "@coral-xyz/anchor";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { FormEvent, useMemo, useState } from "react";
import { Spinner } from "@/components/Spinner";
import { WalletButton } from "@/components/WalletButton";
import { getEscrowPda, getProgram } from "@/lib/anchor";
import styles from "./create.module.css";

// CreateJobPage lets the client create a new escrow account and lock SOL in it.
export default function CreateJobPage() {
  const wallet = useWallet();
  const { connection } = useConnection();
  const router = useRouter();
  const [freelancerAddress, setFreelancerAddress] = useState("");
  const [description, setDescription] = useState("");
  const [amountSol, setAmountSol] = useState("");
  const [deadline, setDeadline] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [toast, setToast] = useState("");

  const previewAmount = useMemo(() => Number(amountSol || "0"), [amountSol]);

  // handleSubmit validates the form, derives the escrow PDA, and sends createEscrow to devnet.
  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!wallet.publicKey) {
      setError("Connect Phantom before creating a job.");
      return;
    }

    try {
      setLoading(true);
      setError("");
      setToast("");

      const freelancer = new PublicKey(freelancerAddress.trim());
      const lamports = Math.round(Number(amountSol) * LAMPORTS_PER_SOL);
      const deadlineTimestamp = Math.floor(new Date(deadline).getTime() / 1000);
      const jobId = new BN(Date.now());
      const amount = new BN(lamports);
      const deadlineBn = new BN(deadlineTimestamp);
      const escrowJob = getEscrowPda(wallet.publicKey, jobId);
      const program = getProgram(connection, wallet);

      await program.methods
        .createEscrow(jobId, freelancer, amount, deadlineBn, description.trim())
        .accounts({
          escrowJob,
          client: wallet.publicKey,
          systemProgram: SystemProgram.programId
        })
        .rpc();

      setToast("Escrow created and SOL locked successfully.");
      router.push(`/job/${jobId.toString()}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Transaction failed while creating escrow.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="shell">
      <header className="topbar">
        <div className="brand">
          <h1>Create escrow job</h1>
          <p>Lock devnet SOL for a freelancer until the work is approved.</p>
        </div>
        <WalletButton />
      </header>

      <div className={styles.layout}>
        <form className={styles.form} onSubmit={handleSubmit}>
          <div className="field">
            <label htmlFor="freelancer">Freelancer wallet address</label>
            <input
              id="freelancer"
              onChange={(event) => setFreelancerAddress(event.target.value)}
              placeholder="Paste freelancer public key"
              required
              value={freelancerAddress}
            />
          </div>

          <div className="field">
            <label htmlFor="description">Job description</label>
            <textarea
              id="description"
              maxLength={200}
              onChange={(event) => setDescription(event.target.value)}
              placeholder="Describe the work needed"
              required
              value={description}
            />
            <span className={styles.counter}>{description.length}/200</span>
          </div>

          <div className="field">
            <label htmlFor="amount">SOL amount</label>
            <input
              id="amount"
              min="0.000000001"
              onChange={(event) => setAmountSol(event.target.value)}
              placeholder="0.10"
              required
              step="0.000000001"
              type="number"
              value={amountSol}
            />
          </div>

          <div className="field">
            <label htmlFor="deadline">Deadline</label>
            <input
              id="deadline"
              onChange={(event) => setDeadline(event.target.value)}
              required
              type="datetime-local"
              value={deadline}
            />
          </div>

          {error ? <p className="error">{error}</p> : null}

          <div className={styles.actions}>
            <button className="button" disabled={loading || !wallet.connected} type="submit">
              {loading ? <Spinner /> : null}
              {loading ? "Confirming..." : "Create escrow"}
            </button>
            <Link className="button secondary" href="/">
              Cancel
            </Link>
          </div>
        </form>

        <aside className={styles.preview}>
          <h2>Lock preview</h2>
          <div className={styles.previewRow}>
            <span className={styles.label}>Amount locked</span>
            <strong className={styles.value}>{previewAmount > 0 ? previewAmount.toFixed(4) : "0.0000"} SOL</strong>
          </div>
          <div className={styles.previewRow}>
            <span className={styles.label}>Freelancer</span>
            <span className={styles.value}>{freelancerAddress || "Not entered yet"}</span>
          </div>
          <div className={styles.previewRow}>
            <span className={styles.label}>Deadline</span>
            <span className={styles.value}>{deadline ? new Date(deadline).toLocaleString() : "Not selected yet"}</span>
          </div>
        </aside>
      </div>

      {toast ? <div className="toast">{toast}</div> : null}
    </main>
  );
}
