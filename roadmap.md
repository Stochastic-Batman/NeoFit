# NeoFit - Client Generation & Frontend Integration Roadmap

## 1. Connecting to the Svelte Frontend

### Overview of Changes

The generated client does not replace all of `wallet.ts` at once. Migration happens
component by component, in this order: wallet adapter -> PDA helpers -> instruction
wrappers -> component updates. Each step is independently testable in the browser before
moving to the next.

### Real Wallet Adapter

The current `wallet.ts` reads a pubkey from `VITE_PUBLIC_WALLET_ADDRESS` and has no
signing capability. Replace it with `@svelte-on-solana/wallet-adapter-core` so that
actual keypair signing flows through the same `$wallet` store the components already use.

```bash
cd app
npm install @solana/web3.js @svelte-on-solana/wallet-adapter-core @svelte-on-solana/wallet-adapter-ui @solana/wallet-adapter-wallets
```

The replacement store keeps the same `{ connected, address }` shape so that wallet-gated
UI conditionals (`{#if $wallet.connected}`) require no changes. Add two new exports that
the instruction wrappers will need:

```typescript
// app/src/lib/wallet.ts  (additions)
export const connection = new Connection(
  import.meta.env.VITE_RPC_URL ?? 'http://127.0.0.1:8899',
  'confirmed'
)

// Returns an Anchor-compatible provider for the currently connected wallet.
export function getProvider(): AnchorProvider { ... }
```

Add `VITE_RPC_URL` to `app/.env.example`.

### PDA Helpers (`app/src/lib/pdas.ts`)

Create this file. It is the one place where seed strings must be kept manually in sync
with `constants.rs`. If the generated Codama client already emits PDA derivers for your
account types, import and re-export them from here rather than reimplementing.

```typescript
// app/src/lib/pdas.ts
import { PublicKey } from '@solana/web3.js'
import { PROGRAM_ID } from '$lib/generated'

export async function userProfilePda(authority: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('user_profile'), authority.toBuffer()],
    PROGRAM_ID
  )
}

export async function challengePda(authority: PublicKey, nonce: bigint) {
  const nonceBuf = Buffer.alloc(8)
  nonceBuf.writeBigUInt64LE(nonce)
  return PublicKey.findProgramAddressSync(
    [Buffer.from('challenge'), authority.toBuffer(), nonceBuf],
    PROGRAM_ID
  )
}

export async function enrollmentPda(challenge: PublicKey, user: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('enrollment'), challenge.toBuffer(), user.toBuffer()],
    PROGRAM_ID
  )
}
```

The seed strings `'user_profile'`, `'challenge'`, and `'enrollment'` must match the byte
literals in `constants.rs` exactly. A mismatch silently derives the wrong address and
produces an `AccountNotFound` error at runtime.

### Instruction Wrappers (`app/src/lib/program.ts`)

One async function per instruction. These are the only places in the frontend that touch
`sendAndConfirmTransaction`. Keeping blockchain logic here means Svelte components stay
clean and the wrappers can be unit-tested independently.

```typescript
// app/src/lib/program.ts
import { getInitializeUserInstruction }  from '$lib/generated'
import { getLogRepsInstruction }         from '$lib/generated'
import { getJoinChallengeInstruction }   from '$lib/generated'
import { getClaimRewardInstruction }     from '$lib/generated'
import { getUpdateUsernameInstruction }  from '$lib/generated'
import { userProfilePda, enrollmentPda } from '$lib/pdas'
import { getProvider }                   from '$lib/wallet'

export async function initializeUser() { ... }

export async function updateUsername(newUsername: string) { ... }

// exerciseId matches WorkoutDef.onChainId - already wired in workouts/index.ts
export async function logReps(
  exerciseId: number,
  count: number,
  challengePda?: PublicKey,
  enrollmentPda?: PublicKey
) { ... }

export async function joinChallenge(challengePda: PublicKey) { ... }

export async function claimReward(challengePda: PublicKey) { ... }
```

Each wrapper: derives necessary PDAs, builds the instruction via the generated helper,
signs and sends the transaction, and throws a typed error on failure. The Svelte component
catches that error and shows UI feedback - no raw Solana errors ever reach the template.

### Component Migration

Migrate one component at a time. Each component below lists what it currently uses and
what replaces it.

**`+layout.svelte`** - The wallet connect/disconnect button already calls `wallet.connect()`
and `wallet.disconnect()`. Replace the mock implementations in `wallet.ts` with the real
adapter. No template changes needed.

**`profile/+page.svelte`** - Currently reads from `localStorage` and writes via
`setUsername`. Replace with two calls: on mount, `fetchUserProfile(rpc, pda)` to read the
`UserProfile` account (populates `username`, `totalReps`, `streakDays`); on save,
`updateUsername(draft)` from `program.ts`. Remove the placeholder-data warning banner once
`userStats` is sourced from the PDA. The `editingName` / `draft` state variables are
unchanged.

**`workout/+page.svelte`** - Currently resets state and displays reps in-browser only.
After the user stops the session (or on a new "Save" button), call `logReps(workout.onChainId, count)`.
If the user is enrolled in a challenge, also pass `challengePda` and `enrollmentPda` from
the URL or local state. The `onChainId` field is already present on every `WorkoutDef`
object (wired in `workouts/index.ts`) so no changes are needed to the detection layer.

**`challenges/+page.svelte`** - The "Join Pool" button currently does nothing. Replace
with a call to `joinChallenge(challengePda)`. Sponsored challenge data is currently
hard-coded; once `Challenge` PDAs exist on-chain, replace with a paginated account fetch
using the generated `fetchAllChallenge(rpc)` or a filtered variant. After joining, persist
`enrollmentPda` in component state so that the workout page can pick it up.

### Exercise ID Mapping

`WorkoutDef.onChainId` (the `u8` sent in `log_reps`) is already sourced from
`exercises.json` via `workouts/index.ts`. No additional mapping is needed. The workout
page reads `cur.onChainId` when it calls `logReps`.

### `.env` Variables

| Variable | Purpose | Example |
|---|---|---|
| `VITE_RPC_URL` | Solana RPC endpoint used by `wallet.ts` | `http://127.0.0.1:8899` |
| `VITE_PUBLIC_WALLET_ADDRESS` | Remove once real adapter is wired | - |

Update `app/.env.example` to document `VITE_RPC_URL` and remove
`VITE_PUBLIC_WALLET_ADDRESS`.


## 2. Recommended Sequence

| Step | Command / Action |
|---|---|
| DONE: 1. Build program | `anchor build` |
| DONE: 2. Generate client | `npm run codama` (workspace root) |
| DONE: 3. Install wallet adapter | `cd app && npm install @svelte-on-solana/wallet-adapter-core ...` |
| 4. Create `pdas.ts` | Manual - keep seeds in sync with `constants.rs` |
| 5. Create `program.ts` | Manual - one wrapper per instruction |
| 6. Migrate `wallet.ts` | Replace mock `connect()` with real adapter |
| 7. Migrate Profile page | `fetchUserProfile` + `updateUsername` |
| 8. Migrate Workout page | `logReps` on session end |
| 9. Migrate Challenges page | `joinChallenge`, live account fetch |
| 10. Deploy to Surfpool | Update `Anchor.toml` cluster URL, `anchor deploy` |

Steps 1–2 must be repeated together after any change to the Rust program. Steps 3–9 are
independent of each other once the generated client and `program.ts` exist.
