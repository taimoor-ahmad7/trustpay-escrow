import { AnchorProvider, BN, Idl, Program } from "@coral-xyz/anchor";
import { WalletContextState } from "@solana/wallet-adapter-react";
import { Connection, PublicKey } from "@solana/web3.js";
import { Buffer } from "buffer";
import escrowIdl from "@/idl/escrow.json";

export const DEVNET_RPC_ENDPOINT = "https://api.devnet.solana.com";

export const PROGRAM_ID = new PublicKey(
  process.env.NEXT_PUBLIC_ESCROW_PROGRAM_ID ?? "11111111111111111111111111111111"
);

// getProvider creates the Anchor object that knows which RPC endpoint and wallet to use.
export function getProvider(connection: Connection, wallet: WalletContextState) {
  return new AnchorProvider(connection, wallet as never, {
    commitment: "confirmed",
    preflightCommitment: "confirmed"
  });
}

// getProgram creates the JavaScript client used to call your Solana program instructions.
export function getProgram(connection: Connection, wallet: WalletContextState) {
  const provider = getProvider(connection, wallet);
  const idlWithAddress = { ...escrowIdl, address: PROGRAM_ID.toBase58() };
  return new Program(idlWithAddress as Idl, provider);
}

// getEscrowPda recreates the same PDA address used by the Rust smart contract.
export function getEscrowPda(client: PublicKey, jobId: BN) {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      client.toBuffer(),
      jobId.toArrayLike(Buffer, "le", 8)
    ],
    PROGRAM_ID
  )[0];
}

// shortenAddress displays a wallet address in a readable first-four plus last-four format.
export function shortenAddress(address: string) {
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}
