"use client";

import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-wallets";
import { useMemo } from "react";
import { Buffer } from "buffer";
import { DEVNET_RPC_ENDPOINT } from "@/lib/anchor";

declare global {
  interface Window {
    Buffer?: typeof Buffer;
  }
}

// Providers gives every page access to the Solana devnet connection and Phantom wallet adapter.
export function Providers({ children }: { children: React.ReactNode }) {
  if (typeof window !== "undefined") {
    window.Buffer = window.Buffer ?? Buffer;
  }

  const wallets = useMemo(() => [new PhantomWalletAdapter()], []);

  return (
    <ConnectionProvider endpoint={DEVNET_RPC_ENDPOINT}>
      <WalletProvider wallets={wallets} autoConnect>
        {children}
      </WalletProvider>
    </ConnectionProvider>
  );
}
