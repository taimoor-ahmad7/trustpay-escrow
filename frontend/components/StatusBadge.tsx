import { EscrowStatus } from "@/lib/jobs";
import styles from "./StatusBadge.module.css";

const STATUS_CLASS: Record<EscrowStatus, string> = {
  Open: styles.open,
  InProgress: styles.inProgress,
  Submitted: styles.submitted,
  Completed: styles.completed,
  Disputed: styles.disputed,
  Refunded: styles.refunded
};

// StatusBadge displays the escrow status with a clear color for quick scanning.
export function StatusBadge({ status }: { status: EscrowStatus }) {
  return <span className={`${styles.badge} ${STATUS_CLASS[status]}`}>{status}</span>;
}
