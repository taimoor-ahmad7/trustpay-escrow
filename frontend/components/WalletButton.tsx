"use client";

import { useWallet } from "@solana/wallet-adapter-react";
import { useCallback, useEffect, useState } from "react";
import { shortenAddress } from "@/lib/anchor";
import styles from "./WalletButton.module.css";

// WalletButton connects or disconnects Phantom and shows the connected wallet address.
export function WalletButton() {
  const { publicKey, connected, connecting, connect, disconnect, select, wallet, wallets } = useWallet();
  const [error, setError] = useState("");
  const [connectRequested, setConnectRequested] = useState(false);

  // handleConnect selects Phantom and asks the user to approve the wallet connection.
  const handleConnect = useCallback(() => {
    try {
      setError("");
      const phantom = wallets.find((wallet) => wallet.adapter.name === "Phantom");

      if (!phantom) {
        setError("Phantom wallet was not found. Please install Phantom first.");
        return;
      }

      select(phantom.adapter.name);
      setConnectRequested(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not connect wallet.");
    }
  }, [select, wallets]);

  useEffect(() => {
    if (!connectRequested || !wallet || connected || connecting) {
      return;
    }

    void connect()
      .catch((err) => {
        setError(err instanceof Error ? err.message : "Could not connect wallet.");
      })
      .finally(() => setConnectRequested(false));
  }, [connect, connectRequested, connected, connecting, wallet]);

  return (
    <div className={styles.walletArea}>
      {connected && publicKey ? (
        <>
          <span className={styles.address}>Connected: {shortenAddress(publicKey.toBase58())}</span>
          <button className={styles.walletButton} onClick={() => disconnect()} type="button">
            Disconnect
          </button>
        </>
      ) : (
        <button className={styles.walletButton} disabled={connecting} onClick={handleConnect} type="button">
          {connecting ? "Connecting..." : "Connect Phantom"}
        </button>
      )}
      {error ? <span className={styles.address}>{error}</span> : null}
    </div>
  );
}
