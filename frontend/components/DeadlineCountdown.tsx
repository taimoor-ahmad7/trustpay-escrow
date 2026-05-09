"use client";

import { useEffect, useState } from "react";

// formatRemaining turns a Unix deadline into a compact countdown label.
function formatRemaining(deadline: number) {
  const secondsLeft = deadline - Math.floor(Date.now() / 1000);

  if (secondsLeft <= 0) {
    return "Deadline passed";
  }

  const days = Math.floor(secondsLeft / 86400);
  const hours = Math.floor((secondsLeft % 86400) / 3600);
  const minutes = Math.floor((secondsLeft % 3600) / 60);

  if (days > 0) {
    return `${days}d ${hours}h left`;
  }

  if (hours > 0) {
    return `${hours}h ${minutes}m left`;
  }

  return `${minutes}m left`;
}

// DeadlineCountdown refreshes the remaining time while the user is viewing the page.
export function DeadlineCountdown({ deadline }: { deadline: number }) {
  const [label, setLabel] = useState(() => formatRemaining(deadline));

  useEffect(() => {
    const interval = window.setInterval(() => setLabel(formatRemaining(deadline)), 30000);
    return () => window.clearInterval(interval);
  }, [deadline]);

  return <span>{label}</span>;
}
