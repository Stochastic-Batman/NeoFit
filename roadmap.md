# NeoFit - Client Generation & Frontend Integration Roadmap

## 1. Testing Reference

### The Test Stack

The project uses **LiteSVM**, an in-process SVM emulator, for unit tests. `Anchor.toml`
is configured to run `cargo test` via `anchor test`.

### Building

`anchor build` produces `target/deploy/neofit.so` (BPF bytecode) and
`target/idl/neofit.json` (the IDL). The IDL is the single source of truth for both the
Codama client generator and the TypeScript frontend. Rebuild after every program change.

### Running Tests

```bash
cargo test -p neofit
# or from workspace root:
cargo test
```

### Deploying to a Live Validator

```bash
# Terminal 1
solana-test-validator

# Terminal 2 - deploy after building
anchor deploy
```

Alternatively, point `cluster` in `Anchor.toml` at a Surfpool RPC URL for a
cloud-hosted validator. Revert before committing.


## 2. Codama Client Generation

### What Codama Is and Why You Need It

Codama is a code-generation pipeline. It reads `target/idl/neofit.json` and emits
fully-typed TypeScript: one file per instruction, one per account type, PDA derivation
helpers, and a barrel `index.ts`. The alternative - constructing raw instruction bytes
with `InstructionData` and manually deriving PDAs, exactly as the Rust tests do - works
but gives you no type checking, no autocomplete, and breaks silently when the IDL
changes.

The generated client is what the Svelte components will import. Wire the frontend before
running Codama and you are building on an untyped, hand-rolled foundation that you will
have to throw away.

### Where the Code Lives

Everything stays inside the existing repo. Codama packages are `devDependencies` of
`app/`. The generation script lives at the workspace root. Generated output is committed
into `app/src/lib/generated/` and re-run after every `anchor build`.

```
(workspace root)
├── codama.ts                        <- generation script (new)
├── target/idl/neofit.json           <- read by the script
└── app/
    ├── package.json                 <- add Codama devDependencies here
    └── src/lib/
        └── generated/               <- committed generated output (new)
            ├── index.ts
            ├── accounts/
            │   ├── userProfile.ts
            │   ├── challenge.ts
            │   └── enrollment.ts
            ├── instructions/
            │   ├── initializeUser.ts
            │   ├── updateUsername.ts
            │   ├── logReps.ts
            │   ├── createChallenge.ts
            │   ├── joinChallenge.ts
            │   └── claimReward.ts
            └── types/
                ├── exerciseCount.ts
                └── exerciseRequirement.ts
```

Add `app/src/lib/generated/` to version control. Treat it like a lock-file: machine-
generated, committed, re-generated when the IDL changes.

### Installing Codama

```bash
cd app
npm install --save-dev \
  codama \
  @codama/nodes-from-anchor \
  @codama/renderers-js \
  @codama/renderers-js-umi \
  tsx
```

`codama` is the core node-manipulation library. `@codama/nodes-from-anchor` parses Anchor
IDLs into Codama's intermediate representation. `@codama/renderers-js` emits plain
TypeScript (no framework dependency). `tsx` lets you run the generation script directly
without a separate compile step.

### The Generation Script

Create `codama.ts` at the workspace root (one level above `app/`):

```typescript
// codama.ts  - run with: npx tsx codama.ts
import { createFromRoot } from 'codama'
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor'
import { renderJavaScriptVisitor } from '@codama/renderers-js'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

const idl = JSON.parse(
  readFileSync(join(__dirname, 'target/idl/neofit.json'), 'utf8')
)

const codama = createFromRoot(rootNodeFromAnchor(idl))

codama.accept(
  renderJavaScriptVisitor(
    join(__dirname, 'app/src/lib/generated'),
    { prettierOptions: { useTabs: true, singleQuote: true } }
  )
)
```

Add a convenience script to the workspace-root `package.json`:

```json
{
  "scripts": {
    "codama": "tsx codama.ts"
  }
}
```

Run it once manually after every `anchor build`:

```bash
anchor build
npm run codama        # from workspace root
```

The correct order is always **build → generate → frontend**. If you change the Rust
program and forget to re-run Codama, TypeScript types will be stale and the compiler will
catch the mismatch.

### What Gets Generated

| File pattern | Contents |
|---|---|
| `accounts/userProfile.ts` | `fetchUserProfile(rpc, address)`, `deserializeUserProfile(data)`, size constants |
| `accounts/challenge.ts` | Same shape for `Challenge` |
| `accounts/enrollment.ts` | Same shape for `Enrollment` |
| `instructions/logReps.ts` | `getLogRepsInstruction({ userProfile, authority, exerciseId, count, ... })` |
| `instructions/joinChallenge.ts` | `getJoinChallengeInstruction(...)` with full account resolution |
| `types/exerciseRequirement.ts` | Borsh codec for `ExerciseRequirement` |
| `index.ts` | Re-exports everything; the only import path Svelte components need |

Every instruction builder accepts a plain object matching the Anchor accounts context. The
compiler will error if a required account is omitted or if `exerciseId` is passed as a
`string` instead of `number`.


## 3. Connecting to the Svelte Frontend

### Overview of Changes

The generated client does not replace all of `wallet.ts` at once. Migration happens
component by component, in this order: wallet adapter → PDA helpers → instruction
wrappers → component updates. Each step is independently testable in the browser before
moving to the next.

### Real Wallet Adapter

The current `wallet.ts` reads a pubkey from `VITE_PUBLIC_WALLET_ADDRESS` and has no
signing capability. Replace it with `@svelte-on-solana/wallet-adapter-core` so that
actual keypair signing flows through the same `$wallet` store the components already use.

```bash
cd app
npm install \
  @solana/web3.js \
  @svelte-on-solana/wallet-adapter-core \
  @svelte-on-solana/wallet-adapter-ui \
  @solana/wallet-adapter-wallets
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


## 4. Recommended Sequence

| Step | Command / Action |
|---|---|
| 1. Build program | `anchor build` |
| 2. Generate client | `npm run codama` (workspace root) |
| 3. Install wallet adapter | `cd app && npm install @svelte-on-solana/wallet-adapter-core ...` |
| 4. Create `pdas.ts` | Manual - keep seeds in sync with `constants.rs` |
| 5. Create `program.ts` | Manual - one wrapper per instruction |
| 6. Migrate `wallet.ts` | Replace mock `connect()` with real adapter |
| 7. Migrate Profile page | `fetchUserProfile` + `updateUsername` |
| 8. Migrate Workout page | `logReps` on session end |
| 9. Migrate Challenges page | `joinChallenge`, live account fetch |
| 10. Deploy to Surfpool | Update `Anchor.toml` cluster URL, `anchor deploy` |

Steps 1–2 must be repeated together after any change to the Rust program. Steps 3–9 are
independent of each other once the generated client and `program.ts` exist.
