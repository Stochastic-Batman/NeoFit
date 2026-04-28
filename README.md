# NeoFit: Prove Your Reps. Own Your Progress.

A decentralized fitness app using **Solana** and **MoveNet** to reward real-world movement. The frontend uses your webcam to verify exercises via computer vision. Verified reps are logged on the Solana blockchain and will earn digital rewards.


## Tech Stack

| Layer | Technology |
|---|---|
| Smart Contracts | Rust 1.95.0 + Anchor 1.0.1 |
| Frontend | SvelteKit 2 + TypeScript + TailwindCSS 4 |
| Computer Vision | MoveNet via ml5.js 1.0.1 |
| On-Chain Testing | LiteSVM (in-process SVM, no validator needed) |
| Wallet | `@svelte-on-solana/wallet-adapter` (TBD: currently mocked) |
| TS <=> Program Bridge | `@anchor-lang/core` (Anchor 1.x TypeScript client) |


## Prerequisites

| Tool | Version |
|---|---|
| Rust | via `rust-toolchain.toml` (1.95.0) |
| Anchor CLI | 1.0.1 |
| Node.js | 24.15.0 |
| Solana CLI | latest stable |

> **Note on Node version:** ml5.js declares a peer requirement of Node `^20.15.1`. Node 24 works at runtime but will trigger engine warnings during `npm install`. Use `--engine-strict false` if you need to install on Node 24.


## Quick Start

### 1. Frontend (works today)

```bash
cd app
npm install
npm run dev
```

The app runs at `http://localhost:5173`. The wallet connect button is currently mocked - it reads a wallet address from the `VITE_PUBLIC_WALLET_ADDRESS` environment variable. Create `app/.env` to test wallet-gated pages:

```env
VITE_PUBLIC_WALLET_ADDRESS=YourBase58WalletAddressHere
```

### 2. Build the Smart Contract

```bash
anchor build
```

Outputs:
- `target/deploy/neofit.so` - compiled BPF bytecode (loaded by tests)
- `target/idl/neofit.json` - IDL used by the TypeScript client

You must rebuild after any change to the Rust program.

### 3. Run Smart Contract Tests

Tests use LiteSVM - an in-process Solana VM. No running validator needed.

```bash
cargo test -p neofit
```

Or from the workspace root:

```bash
cargo test
```

### 4. Local Validator (for manual end-to-end testing)

```bash
# Terminal 1
solana-test-validator

# Terminal 2 - deploy after building
anchor deploy
```

Alternatively, use [Surfpool](https://surfpool.dev) as a cloud-hosted validator. Change `cluster` in `Anchor.toml` to your Surfpool RPC URL while testing, then revert before committing.


## Supported Exercises

All detection runs entirely in-browser. No video is recorded or uploaded.

| Exercise | Camera Position | Detection Method |
|---|---|---|
| Squats | Side-on | Hip-knee-ankle angle < 90° |
| Push-Ups | Side-on | Shoulder-elbow-wrist angle < 90° |
| Pull-Ups | Facing | Shoulder-elbow-wrist angle + wrist above shoulder |
| Sit-Ups | Side-on | Shoulder-hip-knee angle < 75° |
| Jumping Jacks | Facing | Both wrists above shoulder line |

MoveNet tracks 17 skeletal keypoints at the confidence threshold of 0.25. Angles are computed from the raw keypoint coordinates (flip-invariant - mirroring the video feed for UX does not affect detection).


## On-Chain Architecture (TBD)

> The Anchor program currently contains only a placeholder `initialize` instruction. The sections below describe the target architecture; none of it is deployed yet.

### Account Types

**`UserProfile` PDA** - seeds: `["user_profile", wallet_pubkey]`

Stores per-user lifetime stats: username, total reps, per-exercise rep breakdown, streak days, and last workout timestamp.

**`Challenge` PDA** - seeds: `["challenge", authority_pubkey, nonce]`

Created by an admin wallet. Defines the exercise type, rep target, entry fee (in lamports), prize pool, and deadline. Acts as the SOL escrow for entry fees.

**`Enrollment` PDA** - seeds: `["enrollment", challenge_pubkey, user_pubkey]`

One per user per challenge. Tracks reps logged toward that specific challenge, completion status, and whether the reward has been claimed.

### Instructions (all TBD)

| Instruction | Who calls it | What it does |
|---|---|---|
| `initialize_user` | User (first login) | Creates `UserProfile` PDA, sets username |
| `log_reps` | User (end of session) | Adds reps to profile, updates streak |
| `create_challenge` | Admin only | Creates `Challenge` PDA with rules + deadline |
| `join_challenge` | User | Pays entry fee into challenge escrow, creates `Enrollment` |
| `claim_reward` | User (after completing) | Transfers 90% pool share to user; 10% stays for protocol |

### Trust Model

The `log_reps` instruction accepts a rep count from the frontend and cannot verify that it came from MoveNet. For the MVP this is acknowledged. The production path is an oracle: a trusted server runs MoveNet server-side, signs the count, and the instruction verifies the signature against a stored oracle pubkey.


## Recreating This Project

```sh
# Recreate the SvelteKit app with the same configuration
npx sv@0.15.1 create --template minimal --types ts \
  --add prettier eslint tailwindcss="plugins:typography" \
  vitest="usages:unit,component" \
  --install npm app

# Recreate the Anchor workspace
anchor init neofit --no-git
```
