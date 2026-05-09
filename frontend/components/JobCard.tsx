import Link from "next/link";
import { DeadlineCountdown } from "@/components/DeadlineCountdown";
import { StatusBadge } from "@/components/StatusBadge";
import { EscrowJobView } from "@/lib/jobs";
import styles from "./JobCard.module.css";

// JobCard renders one escrow job summary for the home page lists.
export function JobCard({ job }: { job: EscrowJobView }) {
  return (
    <article className={styles.card}>
      <div className={styles.head}>
        <p className={styles.description}>{job.description}</p>
        <StatusBadge status={job.status} />
      </div>
      <p className={styles.meta}>
        <span>{job.amountSol} SOL</span>
        <span>Job #{job.jobId}</span>
        <DeadlineCountdown deadline={job.deadline} />
      </p>
      <div className={styles.footer}>
        <span className={styles.meta}>Escrow account: {job.address.toBase58().slice(0, 8)}...</span>
        <Link className="button secondary" href={`/job/${job.jobId}`}>
          View details
        </Link>
      </div>
    </article>
  );
}
