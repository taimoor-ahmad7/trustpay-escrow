import type { Metadata } from "next";
import "./globals.css";
import { Providers } from "./providers";

export const metadata: Metadata = {
  title: "Trustless Freelance Escrow",
  description: "A Solana escrow dApp for client and freelancer payments."
};

// RootLayout wraps the whole Next.js app with global CSS and Solana wallet providers.
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
