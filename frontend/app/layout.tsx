import type { Metadata } from "next";
import "./globals.css";
import { Providers } from "./providers";
export const metadata: Metadata = {
  title: "Trustless Freelance Escrow",
  description: "A Solana escrow dApp for client and freelancer payments."
};
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <div style={{
          background: "#1a1a2e",
          color: "#a78bfa",
          textAlign: "center",
          padding: "8px",
          fontSize: "13px",
          fontWeight: "500",
          letterSpacing: "0.5px"
        }}>
          University of Management and Technology
        </div>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
