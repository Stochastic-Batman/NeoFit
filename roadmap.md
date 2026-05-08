# Devnet Deployment Roadmap

## Step 1 - Configure Solana CLI for devnet

I need to point my Solana CLI at devnet:

```bash
solana config set --url https://api.devnet.solana.com
```

I verify with:
```bash
solana config get
```

I should see `RPC URL: https://api.devnet.solana.com`.


## Step 2 - Fund the deployer keypair on devnet

My deployer keypair is at `~/.config/solana/id.json`. I need ~3 SOL for deployment fees.

```bash
solana airdrop 5 --url devnet
```

> Devnet airdrops are limited to 5 SOL per request and may rate-limit. I can run multiple times or use https://faucet.solana.com.

I verify with:
```bash
solana balance --url devnet
```


## Step 3 - Add devnet cluster to `Anchor.toml`

I add a `[programs.devnet]` section so Anchor knows my program ID on devnet (same keypair, same address):

```toml
[programs.devnet]
neofit = "BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5"
```

**File to modify:** `Anchor.toml`


## Step 4 - Deploy to devnet

```bash
anchor build
anchor deploy --provider.cluster devnet
```

I verify with:
```bash
solana program show BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5 --url devnet
```

I expect to see my program with my authority key, data length, and balance.


## Step 5 - Update `.env` for devnet

I change the repo-root `.env` to point at devnet:

```env
VITE_RPC_URL=https://api.devnet.solana.com
VITE_PROGRAM_ID=BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5
```

My frontend will now connect to devnet instead of localhost.


## Step 6 - Switch Phantom to devnet

1. I open Phantom -> Settings -> Developer Settings -> enable Testnet Mode
2. I select **Solana Devnet** as the network
3. I airdrop SOL to my Phantom address for transaction fees:

```bash
solana airdrop 2 <MY_PHANTOM_ADDRESS> --url devnet
```

Or I use the Phantom built-in faucet (Settings -> Developer Settings -> "Request Airdrop").


## Step 7 - Seed a challenge on devnet

I update the seed script to use devnet RPC, or pass it as an environment variable.

**File to modify:** `app/scripts/seed-challenge.ts`

I change `RPC_URL` from `http://127.0.0.1:8899` to `https://api.devnet.solana.com` (or read from env).

Then I run:
```bash
cd app && npx tsx scripts/seed-challenge.ts
```

I verify with:
```bash
solana account <CHALLENGE_PDA_ADDRESS> --url devnet
```


## Step 8 - Test the frontend on devnet

```bash
cd app && npm run dev
```

1. I open `http://localhost:5173`
2. I connect Phantom (on Devnet)
3. I navigate to Profile -> Create Profile -> verify on-chain
4. I navigate to Workout -> do reps -> Save to Chain -> verify `totalReps` on Profile
5. I navigate to Challenges -> see seeded challenge -> Join Pool -> verify entry fee deducted
6. I do reps for the challenge exercise -> Save to Chain -> verify progress
7. (After deadline) I claim Reward

**Key difference from localnet:** transactions take ~400ms instead of instant. My UI loading states become visible.


## Step 9 - (Optional) Deploy frontend to Vercel/Netlify

Since my frontend is a SvelteKit app with `adapter-auto`, I can deploy to Vercel or Netlify with zero config:

1. I push to GitHub
2. I connect the repo to Vercel
3. I set build directory to `app`
4. I set environment variables: `VITE_RPC_URL=https://api.devnet.solana.com`, `VITE_PROGRAM_ID=BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5`
5. I deploy

Users can then access my app at a public URL and interact with the devnet program using their Phantom wallet.