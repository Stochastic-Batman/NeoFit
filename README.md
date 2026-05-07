# NeoFit: Prove Your Reps. Own Your Progress.

A Solana-powered training dApp using body pose estimation to reward proper posture and training consistency. The frontend uses your webcam to verify exercises via computer vision. Verified reps are logged on the Solana blockchain and earn digital rewards.


## Tech Stack

| Layer                 | Technology |
|-----------------------|---|
| Smart Contracts       | Rust 1.95.0 + Anchor 1.0.1 |
| Frontend              | SvelteKit 2 + TypeScript + TailwindCSS 4 |
| Computer Vision       | MoveNet via ml5.js 1.0.1 |
| On-Chain Testing      | LiteSVM (in-process SVM, no validator needed) |
| Wallet                | Phantom via `@solana/wallet-adapter-phantom` |
| TS <-> Program Bridge | `@coral-xyz/anchor@0.32.x` (Anchor TypeScript SDK) |


## Prerequisites

| Tool | Version |
|---|---|
| Rust | via `rust-toolchain.toml` (1.95.0) |
| Anchor CLI | 1.0.1 |
| Node.js | 24.15.0 |
| Solana CLI | latest stable |
| Phantom | Browser extension (phantom.app) |


## Quick Start

### 1. Install dependencies

```bash
cd app
npm install --legacy-peer-deps
```

> `--legacy-peer-deps` is required because `@solana/kit` (v2) and `@solana/web3.js` (v1) have conflicting peer dependencies.

### 2. Set up environment variables

Create a `.env` file at the **repo root** (not inside `app/` - Vite is configured with `envDir: '../'`):

```env
VITE_RPC_URL=http://127.0.0.1:8899
VITE_PROGRAM_ID=BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5
```

### 3. Build the Smart Contract

```bash
anchor build
```

Outputs:
- `target/deploy/neofit.so` - compiled BPF bytecode
- `target/idl/neofit.json` - IDL used by the TypeScript client

### 4. Run Smart Contract Tests

Tests use LiteSVM - an in-process Solana VM. No running validator needed.

```bash
cargo test -p neofit
```

### 5. Local Development (end-to-end with Phantom)

**Terminal 1 - start the local validator:**
```bash
solana-test-validator
```

**Terminal 2 - deploy the program:**
```bash
# If you don't have a keypair at ~/.config/solana/id.json, copy yours:
cp ~/PATH_TO_YOUR_KEYPAIR/my-keypair.json ~/.config/solana/id.json

# Build and deploy
anchor build && anchor deploy --provider.cluster localnet

# Verify deployment
solana program show BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5 --url localhost

# Airdrop SOL to your Phantom wallet address
solana airdrop 5 <YOUR_PHANTOM_ADDRESS> --url localhost
```

**Terminal 3 - start the frontend:**
```bash
cd app && npm run dev
```

**In Phantom:**
1. Open Settings -> Developer Settings -> enable Testnet Mode
2. Select **Solana Localnet** as the network
3. Your airdropped SOL should appear in Phantom

**In the browser:**
1. Open `http://localhost:5173`
2. Click "Connect Wallet" - Phantom popup should appear
3. After approving, your truncated address appears in the nav

### 6. Copy IDL after program changes

Every time you modify the Rust program and run `anchor build`, re-copy the IDL:

```bash
cp target/idl/neofit.json app/src/lib/idl/neofit.json
```

Or use the convenience script from `app/`:
```bash
npm run copy-idl
```


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


## On-Chain Architecture

### Account Types

**`UserProfile` PDA** - seeds: `["user_profile", wallet_pubkey]`

Stores per-user lifetime stats: username, total reps, per-exercise rep breakdown, streak days, and last workout timestamp.

**`Challenge` PDA** - seeds: `["challenge", authority_pubkey, nonce]`

Created by an admin wallet. Defines exercise requirements, entry fee (in lamports), prize pool, and deadline. Acts as the SOL escrow for entry fees.

**`Enrollment` PDA** - seeds: `["enrollment", challenge_pubkey, user_pubkey]`

One per user per challenge. Tracks reps logged toward that specific challenge, completion status, and whether the reward has been claimed.

### Instructions

| Instruction | Who calls it | What it does |
|---|---|---|
| `initialize_user` | User (first login) | Creates `UserProfile` PDA |
| `update_username` | User | Updates the username field on `UserProfile` |
| `log_reps` | User (end of session) | Adds reps to profile + optionally to an enrollment |
| `create_challenge` | Admin only | Creates `Challenge` PDA with rules + deadline |
| `join_challenge` | User | Pays entry fee into challenge escrow, creates `Enrollment` |
| `claim_reward` | User (after completing) | Transfers 90% pool share to user; 10% protocol fee |

### Trust Model

The `log_reps` instruction accepts a rep count from the frontend and cannot verify that it came from MoveNet. For the MVP this is acknowledged. The production path is an oracle: a trusted server runs MoveNet server-side, signs the count, and the instruction verifies the signature against a stored oracle pubkey.
