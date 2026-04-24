# NeoFit: Prove Your Reps. Own Your Progress.

A decentralized fitness app using **Solana** and **MoveNet** to reward real-world movement.

To recreate this project with the same configuration:

```sh
# recreate this project
npx sv@0.15.1 create --template minimal --types ts --add prettier eslint tailwindcss="plugins:typography" vitest="usages:unit,component" --install yarn app
```

## Tech Stack
- **On-Chain:** Rust + Anchor (Solana)
- **Frontend:** SvelteKit + TypeScript + TailwindCSS
- **Vision:** MoveNet (ml5.js)

## Project Structure
- `/programs`: Rust smart contracts.
- `/app`: SvelteKit frontend & ML logic.
- `/tests`: Integration tests (Anchor).

## Quick Start
1. **Frontend Dev:** `cd app && npm run dev`
2. **Rust Build:** `anchor build`
3. **Local Validator:** `solana-test-validator`
