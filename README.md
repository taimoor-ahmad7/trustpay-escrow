# TrustPay Escrow

A Solana escrow dApp that lets clients lock SOL for freelancers and release payment only when work is approved.

## Problem Statement

Online freelance work depends on trust between people who may never meet. Freelancers risk completing work and not getting paid. Clients risk paying upfront and not receiving the promised work. This is especially difficult for international freelancers, students, and builders in regions where access to reliable payment platforms is limited.

Traditional platforms such as PayPal, Fiverr, and Upwork reduce some risk, but they also become middlemen. They charge high fees, delay withdrawals, reverse transactions, and restrict accounts. TrustPay Escrow replaces the payment middleman with transparent Solana smart contract logic.

## Live Demo

- **Frontend:** https://trustpay-escrow-4lp5.vercel.app
- **Program ID:** 9e8CP55VmG48GRvLQCq9ytrofocjWy4YtCsgSSonM96R
- **Network:** Solana Devnet
- **Explorer:** https://explorer.solana.com/address/9e8CP55VmG48GRvLQCq9ytrofocjWy4YtCsgSSonM96R?cluster=devnet

## How It Works

1. Client connects Phantom wallet
2. Client creates a job, enters the freelancer wallet address, adds a description, sets a deadline, and locks SOL
3. The SOL is held inside an escrow PDA account controlled by the Solana program
4. The freelancer accepts the job
5. The freelancer submits the work
6. The client approves the work
7. The smart contract releases the locked SOL directly to the freelancer
8. If the deadline passes before submission, the client can claim a full refund
9. If work is submitted but there is a disagreement, either side can raise a dispute

## Tech Stack

| Layer | Technology |
|---|---|
| Blockchain | Solana Devnet |
| Smart Contract | Anchor Framework |
| Contract Language | Rust |
| Frontend | Next.js 14, React, TypeScript |
| Wallet | Phantom |
| Solana Client | @coral-xyz/anchor, @solana/web3.js |
| Wallet Adapter | @solana/wallet-adapter |
| Deployment | Vercel |

## Security Features

- Safe CPI transfers using invoke_signed instead of raw lamport manipulation
- Self-assignment blocked — client cannot assign themselves as freelancer
- Dispute resolution instruction — SOL cannot be locked forever
- Minimum 0.001 SOL enforced to prevent spam jobs
- Maximum 1 year deadline enforced
- Freelancer cannot submit work after deadline passes
- Escrow accounts close automatically after completion or refund — rent is reclaimed

## Local Development Setup

### Prerequisites

- Node.js 18+
- Rust and Cargo
- Solana CLI
- Anchor CLI
- Phantom wallet browser extension

### Clone the repository

```bash
git clone https://github.com/taimoor-ahmad7/trustpay-escrow.git
cd trustpay-escrow
```

### Configure Solana CLI to devnet

```bash
solana config set --url https://api.devnet.solana.com
```

### Check your wallet balance and airdrop if needed

```bash
solana balance
solana airdrop 2
```

### Build the Anchor program

```bash
anchor build
```

### Deploy the Anchor program

```bash
anchor deploy --provider.cluster devnet
```

### Copy the generated IDL to the frontend

```bash
cp target/idl/escrow.json frontend/idl/escrow.json
```

### Set up the frontend environment

```bash
cp frontend/.env.local.example frontend/.env.local
```

Add your deployed program ID to `frontend/.env.local`:

```env
NEXT_PUBLIC_ESCROW_PROGRAM_ID=9e8CP55VmG48GRvLQCq9ytrofocjWy4YtCsgSSonM96R
```

### Install frontend dependencies

```bash
cd frontend
npm install
```

### Run the frontend locally

```bash
npm run dev
```

Open http://localhost:3000

## Smart Contract Instructions

| Instruction | Who Calls It | What It Does |
|---|---|---|
| `create_escrow` | Client | Creates a job, assigns a freelancer, sets a deadline, and locks SOL in a PDA |
| `accept_job` | Freelancer | Accepts the job and changes status to InProgress |
| `submit_work` | Freelancer | Marks the job as submitted for client review (must be before deadline) |
| `approve_release` | Client | Releases locked SOL to the freelancer and closes the escrow account |
| `claim_refund` | Client | Refunds the client if the deadline passes before submission |
| `raise_dispute` | Client or Freelancer | Flags the job as disputed after work has been submitted |
| `resolve_dispute` | Arbitrator | Resolves a disputed job by sending SOL to either party |

## Vulnerability Fixes Applied

The following security issues were identified and fixed during development:

1. **Raw lamport manipulation** — replaced with safe CPI transfers using invoke_signed
2. **Self-assignment exploit** — client can no longer assign themselves as freelancer
3. **Permanent SOL lock on dispute** — resolve_dispute instruction added
4. **No minimum amount** — enforced 0.001 SOL minimum
5. **No maximum deadline** — enforced 1 year maximum
6. **Submit after deadline** — deadline check added to submit_work
7. **Accounts never closed** — close constraint added to reclaim rent

## Screenshots

1. Home page with connected wallet
2. Create job form
3. Job detail page — Open status
4. Job detail page — after freelancer accepts
5. Job detail page — after work submission
6. Phantom transaction confirmation
7. Solana Explorer transaction

## License

MIT
