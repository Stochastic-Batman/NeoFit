# Frontend <-> Backend Integration Roadmap

## Completed Steps

- **Step 1** - Installed `@coral-xyz/anchor@0.32.x`, `@solana/kit@6.x`, `@solana/wallet-adapter-phantom` via `--legacy-peer-deps`
- **Step 2** - Created `.env` and `.env.example` at repo root with `VITE_RPC_URL` and `VITE_PROGRAM_ID`
- **Step 3** - Copied IDL to `app/src/lib/idl/neofit.json`; added `copy-idl` script to `package.json`
- **Step 4** - Rewrote `app/src/lib/wallet.ts` with real Phantom adapter, `Connection`, `getProvider()`
- **Step 5** - Layout unchanged (store shape preserved: `$wallet.connected`, `wallet.connect()`/`disconnect()`)
- **Step 6** - Created `app/src/lib/pdas.ts` (PDA derivation for user_profile, challenge, enrollment)
- **Step 7** - Created `app/src/lib/program.ts` (instruction wrappers + account fetchers)
- **Vite fix** - Replaced `@solana/wallet-adapter-wallets` with `@solana/wallet-adapter-phantom` to avoid broken `@ledgerhq` ESM imports; updated `vite.config.ts` SSR/optimizeDeps


## Next: Step 8 - Start local validator, deploy, and airdrop

**Prerequisites:** Solana CLI installed, `anchor` CLI installed, Phantom extension in your browser set to **Solana Localnet**.

**Terminal commands (run from workspace root):**

1. Start a local validator (leave running in its own terminal):
   ```bash
   solana-test-validator
   ```

2. Build and deploy:
   ```bash
   anchor build && anchor deploy --provider.cluster localnet
   ```

3. Verify deployment:
   ```bash
   solana program show BWJXEiNyQv9h2f9Aq9HCw8NyvSbYitJ7ChyUhkR887o5 --url localhost
   ```

4. Airdrop to your Phantom address:
   ```bash
   solana airdrop 5 <YOUR_PHANTOM_ADDRESS> --url localhost
   ```

5. In Phantom: confirm network is set to **Solana Localnet** (Developer Settings -> Testnet Mode -> select Solana Localnet).

6. Copy the IDL (if not already done):
   ```bash
   cp target/idl/neofit.json app/src/lib/idl/neofit.json
   ```

7. Start the frontend:
   ```bash
   cd app && npm run dev
   ```

**How to test:** Open the app, connect Phantom, check that Phantom shows your airdropped SOL balance. The app should load without errors. No on-chain calls yet - that is the next step.



## Step 9 - Migrate Profile page

**File to modify:** `app/src/routes/profile/+page.svelte`

**What to change:**

1. Remove imports of `getUsername`, `setUsername` from `wallet.ts`.
2. Import `fetchUserProfile`, `initializeUser`, `updateUsername` from `$lib/program`.
3. Add an `onMount` (or `$effect`) block that runs when the wallet is connected:
   - Call `fetchUserProfile(walletPublicKey)`.
   - If it returns `null`, show a "Create Profile" button that calls `initializeUser()`. After the transaction confirms, re-fetch the profile.
   - If it returns data, populate `username`, `totalReps`, `streakDays` from the account fields.
4. Replace `saveName()`: instead of writing to `localStorage`, call `updateUsername(draft)`. Add a loading/spinner state while the transaction is in flight.
5. Remove the hardcoded `userStats` object.
6. Remove the yellow "placeholder data" warning banner.
7. The `editingName` / `draft` UI state variables remain unchanged.

**How to test:**
- Connect wallet, navigate to Profile.
- If first visit: "Create Profile" button should appear. Click it -> Phantom popup -> approve -> page shows default username and zero stats.
- Click username to edit -> type new name -> Save -> Phantom popup -> approve -> new name displayed.
- Refresh page -> data persists (on-chain, not localStorage).
- Disconnect wallet -> "Wallet Required" screen shows.


## Step 10 - Migrate Workout page

**File to modify:** `app/src/routes/workout/+page.svelte`

**What to change:**

1. Import `logReps` from `$lib/program`.
2. Add a "Save to Chain" button (enabled only when wallet connected and `count > 0`).
3. When clicked: call `logReps(cur.onChainId, count)`.
4. Show loading/success/error states.
5. After successful save, reset the rep counter.
6. If wallet not connected, hide button or show "Connect Wallet to Save" tooltip.
7. Challenge-aware logging deferred to Step 13.

**How to test:**
- Connect wallet, ensure profile exists (from Step 9).
- Select exercise, perform reps on camera.
- Click "Save to Chain" -> Phantom popup -> approve -> transaction confirms.
- Navigate to Profile -> `totalReps` increased.



## Step 11 - Create a challenge seeding script

**File to create:** `scripts/seed-challenge.ts`

**What the script does:**
- Loads deployer keypair from `~/.config/solana/id.json`
- Connects to localhost
- Loads the IDL
- Calls `program.methods.createChallenge(…)` with sample parameters
- Prints created Challenge PDA address and transaction signature

**How to run:** `npx tsx scripts/seed-challenge.ts`

**How to test:** `solana account <CHALLENGE_PDA> --url localhost` shows account data.



## Step 12 - Migrate Challenges page (display)

**File to modify:** `app/src/routes/challenges/+page.svelte`

**What to change:**

1. Import `fetchAllChallenges` from `$lib/program`.
2. On mount: call `fetchAllChallenges()` and store in component state.
3. Replace hardcoded `sponsoredChallenges` with fetched on-chain data.
4. If no challenges exist, show empty state message.

**How to test:**
- Run seed script from Step 11 first.
- Connect wallet, navigate to Challenges.
- Seeded challenge appears in the grid.



## Step 13 - Migrate Challenges page (join + claim + challenge-aware workouts)

**File to modify:** `app/src/routes/challenges/+page.svelte`

1. Import `joinChallenge`, `claimReward`, `fetchEnrollment` from `$lib/program`.
2. Check enrollment status per challenge; show "Enrolled" badge or "Join Pool" button.
3. "Join Pool" -> calls `joinChallenge(challengeKey)`.
4. "Claim Reward" button when deadline passed + requirements met.
5. Show progress toward challenge requirements.

**File to modify:** `app/src/routes/workout/+page.svelte`

- Pass `challengeKey` to `logReps()` if enrolled in a challenge.

**How to test:**
- Seed a challenge -> Join Pool -> do reps -> Save to Chain -> verify progress -> Claim Reward.



## Step 14 - End-to-end smoke test

1. `solana-test-validator`
2. `anchor build && anchor deploy --provider.cluster localnet`
3. `cp target/idl/neofit.json app/src/lib/idl/neofit.json`
4. `solana airdrop 5 <PHANTOM_ADDR> --url localhost`
5. `npx tsx scripts/seed-challenge.ts`
6. `cd app && npm run dev`
7. Browser: connect Phantom (Localnet) -> Create Profile -> edit username -> workout -> save -> challenges -> join -> claim



## Step 15 - Cleanup

- `app/src/lib/wallet.ts` - remove compatibility shims (`getUsername`/`setUsername`)
- `app/src/routes/profile/+page.svelte` - remove placeholder banner
- `app/src/routes/challenges/+page.svelte` - remove hardcoded arrays
- `app/package.json` - remove `@anchor-lang/core`, `@solana/wallet-adapter-wallets`, `@svelte-on-solana/wallet-adapter-core`, `@svelte-on-solana/wallet-adapter-ui` (unused)
- `.env.example` - confirm `VITE_PUBLIC_WALLET_ADDRESS` is removed

**How to test:**
- `npm run check` - no type errors
- `npm run build` - production build succeeds



## File Reference

| File | Action | Step |
|---|---|---|
| `app/src/lib/wallet.ts` | Done | 4 |
| `app/src/lib/pdas.ts` | Done | 6 |
| `app/src/lib/program.ts` | Done | 7 |
| `app/src/lib/idl/neofit.json` | Done | 3 |
| `app/vite.config.ts` | Done | fix |
| `app/src/routes/profile/+page.svelte` | Modify | 9 |
| `app/src/routes/workout/+page.svelte` | Modify | 10, 13 |
| `scripts/seed-challenge.ts` | Create | 11 |
| `app/src/routes/challenges/+page.svelte` | Modify | 12, 13 |
