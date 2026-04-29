# NeoFit - Anchor Backend Roadmap

## 1. Target File Structure

Two small structs are used across multiple account types. Both derive the standard Anchor
and Borsh traits and live at the top of `state.rs`.

Given:

1. Pairs an exercise identifier with a rep total. Used in `UserProfile.rep_counts` and
`Enrollment.reps_logged`:

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ExerciseCount {
    pub exercise_id: u8,
    pub count: u32,
}
// serialized size: 5 bytes per entry
```

2. Pairs an exercise identifier with the number of reps needed to satisfy it. Used in
`Challenge.requirements`:

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExerciseRequirement {
    pub exercise_id: u8,
    pub rep_target: u16,
}
// serialized size: 3 bytes per entry
```


```
exercises.json              <- Single source of truth for exercise definitions.

programs/neofit/src/
├── lib.rs                  <- Adds pub fn entries for each new instruction.
├── constants.rs            <- SEED_USER_PROFILE, SEED_CHALLENGE, SEED_ENROLLMENT,
│                              MAX_USERNAME_LEN, MAX_EXERCISE_ID, MAX_REQUIREMENTS,
│                              MAX_EXERCISES_TRACKED, PROTOCOL_FEE_BPS.
├── error.rs                <- NotAuthorized, ChallengeExpired, AlreadyClaimed,
│                              ChallengeInactive, InsufficientFunds, InvalidExerciseId,
│                              TooManyRequirements, UsernameTooLong, Overflow.
├── state.rs                <- ExerciseCount, ExerciseRequirement, UserProfile,
│                              Challenge, Enrollment structs.
├── instructions.rs         <- pub mod + pub use for every instruction below.
└── instructions/
    ├── initialize.rs       <- Kept as-is placeholder; can be deleted post-MVP.
    ├── initialize_user.rs  <- Creates a UserProfile PDA with a derived default username.
    ├── update_username.rs  <- Lets the user set a custom username after initialisation.
    ├── log_reps.rs         <- Increments rep count, updates streak, checks completion.
    ├── create_challenge.rs <- Any user creates a Challenge PDA.
    ├── join_challenge.rs   <- User pays entry fee, creates Enrollment PDA.
    └── claim_reward.rs     <- Verified completer claims their SOL share.

programs/neofit/tests/
├── test_initialize.rs          <- Existing placeholder test.
├── test_initialize_user.rs
├── test_update_username.rs
├── test_log_reps.rs
├── test_create_challenge.rs
├── test_join_challenge.rs
└── test_claim_reward.rs
```

## 2. Account Structs (`src/state.rs`)

### `UserProfile`

PDA seeds: `[SEED_USER_PROFILE, authority_pubkey]`

| Field | Type | Bytes | Purpose |
| --- | --- | --- | --- |
| `authority` | `Pubkey` | 32 | Wallet that owns this profile. |
| `username` | `String` | 26 | `4 + MAX_USERNAME_LEN`. |
| `total_reps` | `u64` | 8 | Lifetime verified reps across all exercises. |
| `rep_counts` | `Vec<ExerciseCount>` | 504 | `4 + MAX_EXERCISES_TRACKED (100) * 5`. |
| `last_workout_ts` | `i64` | 8 | Unix timestamp of last session, for streak logic. |
| `streak_days` | `u32` | 4 | Current daily streak. |
| `bump` | `u8` | 1 | Stored PDA bump. |

**Total space:** `8 + 32 + 26 + 8 + 504 + 8 + 4 + 1 = 591 bytes`

**Note:** `rep_counts` is sparse. On `log_reps`, the instruction searches for an
existing entry matching `exercise_id`; if none is found and the vec length is below
`MAX_EXERCISES_TRACKED`, a new `ExerciseCount` is pushed. The profile page displays only
entries that exist.


### `Challenge`

PDA seeds: `[SEED_CHALLENGE, authority_pubkey, nonce.to_le_bytes()]`

The `nonce: u64` argument keeps seeds unique across multiple challenges from the same
authority. A `title`-based seed would work for a user's first challenge, but if they
create a second challenge with the same title - or update and redeploy one - the derived
address would collide with the existing account, causing `init` to fail. A monotonically
incrementing `u64` nonce, tracked off-chain by the creator's client, guarantees a fresh
address every time without any coordination with the program. The nonce is passed both as
a seed (for address derivation) and stored in the account so the TypeScript PDA helper can
reconstruct the address later.

| Field | Type | Bytes | Purpose |
| --- | --- | --- | --- |
| `authority` | `Pubkey` | 32 | Creator; only they can toggle `is_active`. |
| `title` | `String` | 36 | `4 + 32`. |
| `requirements` | `Vec<ExerciseRequirement>` | 49 | `4 + MAX_REQUIREMENTS (15) * 3`. Defines which exercises and how many reps of each are needed. |
| `entry_fee_lamports` | `u64` | 8 | SOL cost to join. Zero means free. |
| `pool_lamports` | `u64` | 8 | Total entry fees collected so far. |
| `completers` | `u32` | 4 | Count of users who have satisfied all requirements. |
| `deadline_ts` | `i64` | 8 | Unix timestamp; instructions reject calls after this. |
| `is_active` | `bool` | 1 | Creator kill-switch. |
| `nonce` | `u64` | 8 | Stored so the TypeScript PDA helper can reconstruct the address. |
| `bump` | `u8` | 1 | Stored PDA bump. |

**Total space:** `8 + 32 + 36 + 49 + 8 + 8 + 4 + 8 + 1 + 8 + 1 = 163 bytes`


### `Enrollment`

PDA seeds: `[SEED_ENROLLMENT, challenge_pubkey, user_pubkey]`

| Field | Type | Bytes | Purpose |
| --- | --- | --- | --- |
| `user` | `Pubkey` | 32 | |
| `challenge` | `Pubkey` | 32 | |
| `reps_logged` | `Vec<ExerciseCount>` | 79 | `4 + MAX_REQUIREMENTS (15) * 5`. Pre-populated at join time with one zeroed entry per requirement; positions correspond to `challenge.requirements` positions. |
| `completed` | `bool` | 1 | Set to `true` by `log_reps` when all requirements are met; never set by the frontend. |
| `reward_claimed` | `bool` | 1 | Double-claim guard. |
| `bump` | `u8` | 1 | |

**Total space:** `8 + 32 + 32 + 79 + 1 + 1 + 1 = 154 bytes`

**Design note:** `reps_logged` is pre-populated (not sparse) so that `log_reps` can
update by index rather than search. Entry at index `i` tracks progress toward
`challenge.requirements[i]`. `completed` is derived entirely on-chain; the frontend
cannot set it.


## 3. Instructions

### `initialize_user`

**Caller:** The user, once, on first wallet connection.

**Accounts context:**
```
user_profile:   Account<UserProfile>  [init, seeds = [SEED_USER_PROFILE, authority.key()], bump]
authority:      Signer
system_program: Program<System>
```

**Instruction arguments:** none.

The public key does not need to be passed as an argument - `authority.key()` is already
available in the accounts context because `authority` is a required `Signer`. The runtime
enforces that the signer's key matches what was used to derive the PDA, so no separate
input is needed.

**Logic:**
1. A default username is constructed on-chain from `authority.key()`:
   `format!("user_{}{}", &addr[..4], &addr[addr.len()-4..])` where `addr` is the
   base58-encoded pubkey. This produces a string like `user_7xKf...mQ3z` which is always
   unique per wallet and always fits within 22 characters.
2. `authority` is stored. `total_reps = 0`, `rep_counts = vec![]`, `streak_days = 0`,
   `last_workout_ts = 0`.
3. `bump` is stored as `ctx.bumps.user_profile`.


### `update_username`

**Caller:** The profile owner, any time after `initialize_user`.

**Accounts context:**
```
user_profile: Account<UserProfile>  [mut, seeds = [SEED_USER_PROFILE, authority.key()], has_one = authority]
authority:    Signer
```

**Instruction arguments:** `new_username: String`

**Logic:**
1. `new_username.len() >= 1 && new_username.len() <= MAX_USERNAME_LEN` is validated;
   returns `ErrorCode::UsernameTooLong` (or a new `UsernameTooShort`) otherwise.
2. `user_profile.username = new_username`.

The frontend Profile page's existing "Edit handle" UI calls this instruction instead of
writing to localStorage.


### `log_reps`

**Caller:** The user at the end of a workout session. Optionally also updates an
enrollment if the user is participating in a challenge.

**Accounts context:**
```
user_profile:  Account<UserProfile>  [mut, seeds = [...], has_one = authority]
authority:     Signer
clock:         Sysvar<Clock>
// Optional - only required when the user is in a challenge:
enrollment:    Option<Account<Enrollment>>  [mut, seeds = [...]]
challenge:     Option<Account<Challenge>>
```

**Instruction arguments:** `exercise_id: u8`, `count: u32`

**Logic:**
1. `exercise_id < MAX_EXERCISE_ID` is validated (`MAX_EXERCISE_ID = 5` means valid IDs
   are 0–4); returns `ErrorCode::InvalidExerciseId`.
2. `count` is added to `user_profile.total_reps` using `checked_add`.
3. `user_profile.rep_counts` is searched for an entry with matching `exercise_id`. If
   found, its `count` is incremented with `checked_add`. If not found and
   `rep_counts.len() < MAX_EXERCISES_TRACKED`, a new `ExerciseCount` is pushed.
4. **Streak logic** (using `clock.unix_timestamp`):
   - `now_day = ts / 86_400`, `last_day = last_workout_ts / 86_400`.
   - If `last_workout_ts == 0` -> `streak_days = 1`.
   - If `now_day == last_day` -> streak unchanged.
   - If `now_day == last_day + 1` -> `streak_days` incremented.
   - If `now_day > last_day + 1` -> `streak_days = 1` (streak broken).
5. `last_workout_ts` is updated to `now`.
6. **If `enrollment` and `challenge` accounts are present:**
   - The position `i` in `enrollment.reps_logged` whose `exercise_id` matches is found.
   - `enrollment.reps_logged[i].count` is incremented with `checked_add`.
   - If `enrollment.completed` is already `true`, step 6 is skipped entirely.
   - After the update, all entries in `enrollment.reps_logged` are checked against
     `challenge.requirements[i].rep_target`. If every entry satisfies its target,
     `enrollment.completed = true` and `challenge.completers` is incremented.


### `create_challenge`

**Caller:** Any wallet. Challenges are open to create; the frontend only promotes
sponsored ones. The `authority` field on the stored account identifies the creator and is
the only wallet that can later toggle `is_active`.

**Accounts context:**
```
challenge:      Account<Challenge>  [init, seeds = [SEED_CHALLENGE, authority.key(), nonce.to_le_bytes()], bump]
authority:      Signer
system_program: Program<System>
clock:          Sysvar<Clock>
```

**Instruction arguments:**
`title: String`, `requirements: Vec<ExerciseRequirement>`, `entry_fee_lamports: u64`,
`deadline_ts: i64`, `nonce: u64`

**Logic:**
1. `requirements.len() >= 1 && requirements.len() <= MAX_REQUIREMENTS` is validated;
   returns `ErrorCode::TooManyRequirements`.
2. Each entry in `requirements` is validated: `exercise_id < MAX_EXERCISE_ID` and
   `rep_target > 0`.
3. `deadline_ts > clock.unix_timestamp` is validated; returns `ErrorCode::ChallengeExpired`.
4. All fields are stored; `pool_lamports = 0`, `completers = 0`, `is_active = true`,
   `nonce` is stored for later PDA reconstruction.


### `join_challenge`

**Caller:** A user entering an active challenge.

**Accounts context:**
```
enrollment:     Account<Enrollment>   [init, seeds = [SEED_ENROLLMENT, challenge.key(), authority.key()], bump]
challenge:      Account<Challenge>    [mut]
user_profile:   Account<UserProfile>  [seeds = [...], has_one = authority]
authority:      Signer
system_program: Program<System>
clock:          Sysvar<Clock>
```

**Logic:**
1. `challenge.is_active == true` is checked; returns `ErrorCode::ChallengeInactive`.
2. `clock.unix_timestamp < challenge.deadline_ts` is checked; returns
   `ErrorCode::ChallengeExpired`.
3. If `entry_fee_lamports > 0`, lamports are transferred from `authority` to the
   `challenge` PDA via a system program CPI.
4. `challenge.pool_lamports` is incremented with `checked_add`.
5. `enrollment` is initialised. `reps_logged` is pre-populated with one `ExerciseCount`
   per entry in `challenge.requirements`, each with `count = 0` and the matching
   `exercise_id`. This keeps indices aligned for `log_reps` updates.
6. `enrollment.completed = false`, `enrollment.reward_claimed = false`.


### `claim_reward`

**Caller:** A user where `enrollment.completed == true` and `reward_claimed == false`.

**Accounts context:**
```
enrollment: Account<Enrollment>  [mut, seeds = [SEED_ENROLLMENT, challenge.key(), authority.key()], has_one = user, has_one = challenge]
challenge:  Account<Challenge>   [mut]
authority:  Signer
```

**Logic:**
1. `enrollment.completed == true` is checked.
2. `!enrollment.reward_claimed` is checked; returns `ErrorCode::AlreadyClaimed`.
3. `user_share` is calculated as
   `challenge.pool_lamports * (10_000 - PROTOCOL_FEE_BPS) / 10_000 / challenge.completers as u64`.
4. `user_share` lamports are transferred from the `challenge` PDA to `authority` using a
   PDA signer (`challenge` signs via its bump).
5. `enrollment.reward_claimed = true`.


## 4. Running and Testing

### The Test Stack

The project uses **LiteSVM**, an in-process SVM emulator, for unit tests. `Anchor.toml`
is configured to run `cargo test` via `anchor test`.

### Building

`anchor build` produces `target/deploy/neofit.so` (BPF bytecode) and
`target/idl/neofit.json` (the IDL). Rebuilds are required after every program change.

### LiteSVM Unit Tests

Each test file follows the same skeleton:
1. A `LiteSVM` instance is created.
2. The compiled `.so` is loaded via `include_bytes!`.
3. A keypair is created and airdropped SOL.
4. The instruction is constructed with Anchor-encoded data and account metas.
5. A `VersionedTransaction` is built and sent via `svm.send_transaction(tx)`.

### Deploying to a Live Validator

Deployments can be made to a local validator using `solana-test-validator` and
`anchor deploy`, or to a cloud-hosted validator like Surfpool by updating the cluster URL
in `Anchor.toml`.


## 5. Connecting to the Svelte Frontend

1. **IDL:** After `anchor build`, `target/idl/neofit.json` is copied to
   `app/src/lib/idl/neofit.json`. The Anchor TypeScript client uses this to encode and
   decode instructions automatically.
2. **Real Wallet Adapter:** The mocked `wallet.ts` store is replaced with
   `@svelte-on-solana/wallet-adapter` to provide a real `PublicKey`, signing capability,
   and an RPC `Connection`.
3. **PDA Helpers (`app/src/lib/pdas.ts`):** All PDA derivations are centralised here,
   using the same seed strings as `constants.rs`. This is the one place where the seeds
   must be kept manually in sync between Rust and TypeScript.
4. **Exercise ID mapping:** The `workouts` array in `index.ts` imports `exercises.json`
   and exposes `onChainId: number` on each workout definition. The `log_reps` call reads
   `workout.onChainId` rather than an array index.
5. **Instruction Wrappers (`app/src/lib/program.ts`):** One async function per
   instruction keeps blockchain logic out of Svelte components.
6. **Replacing Mocks:** The Profile page reads `UserProfile` from the PDA and calls
   `update_username` instead of writing to localStorage. The Workout page calls `log_reps`
   on session end. The Challenges page calls `join_challenge` and `claim_reward`.


## 6. Security Notes (NeoFit-specific)

* **The MoveNet trust problem:** `log_reps` accepts a rep count from the frontend that
  the program cannot independently verify. For production, a trusted oracle signs the
  count server-side and the instruction verifies the signature against a stored oracle
  pubkey.
* **Anchor safeguards:** `overflow-checks = true` is set in `Cargo.toml`. `checked_add`
  is used explicitly for clarity. `init` (not `init_if_needed`) prevents re-initialisation
  attacks. `has_one` constraints enforce signer checks. Timestamps are read from
  `Clock::get()`, never from instruction arguments.
* **Completion flag integrity:** `enrollment.completed` is set exclusively by `log_reps`
  on-chain after verifying all requirements. The frontend has no instruction path to set
  it directly.


## 7. Build Order

1. **`initialize_user`** - Creates the on-chain `UserProfile` PDA; unblocks the Profile
   page.
2. **`update_username`** - Lets the user set a custom handle; replaces the localStorage
   mock in the Profile page.
3. **`log_reps`** - Connects MoveNet output to the blockchain; unblocks the Workout page.
4. **`create_challenge`** - Any user (or an admin script) populates the chain with
   challenges.
5. **`join_challenge`** - Connects the "Join Pool" UI to fee escrow.
6. **`claim_reward`** - Implements payout logic and the double-claim guard.
7. **`withdraw_fees` (post-MVP)** - Allows the protocol to collect its share.
8. **Token rewards (post-MVP)** - Replaces SOL payouts with an SPL token mint.
